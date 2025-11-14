use interface::uart::StageMotorCmd;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, TryReceiveError, TrySendError},
    signal::Signal,
};
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Input, Output};
use heapless::format;

use crate::com::{send_device_event, send_error, send_info, send_warning};

struct StageMotorChannels {
    commands: Channel<CriticalSectionRawMutex, StageMotorCmd, 8>,
    stop: Signal<CriticalSectionRawMutex, ()>,
}

static STAGE_MOTOR: StageMotorChannels = StageMotorChannels {
    commands: Channel::new(),
    stop: Signal::new(),
};

enum StageMotorError {
    CommandAborted,
    LowerLimitReached,
    UpperLimitReached,
    LowerEndStopNotAvailable,
}

impl StageMotorError {
    fn as_str(&self) -> &'static str {
        match self {
            StageMotorError::CommandAborted => "Command aborted",
            StageMotorError::LowerLimitReached => "Lower limit reached",
            StageMotorError::UpperLimitReached => "Upper limit reached",
            StageMotorError::LowerEndStopNotAvailable => "Lower end stop not available",
        }
    }
}

impl core::fmt::Display for StageMotorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub struct StageMotor {
    position: i32,
    lower_limit: Option<i32>,
    upper_limit: Option<i32>,

    end_stop_lower: Option<Input<'static>>,
    en_pin: Option<Output<'static>>,
    step_pin: Output<'static>,
    dir_pin: Output<'static>,
}

impl StageMotor {
    pub fn send_command(cmd: StageMotorCmd) -> Result<(), TrySendError<StageMotorCmd>> {
        if let StageMotorCmd::Stop = cmd {
            Self::request_stop();
            return Ok(());
        }

        STAGE_MOTOR.commands.try_send(cmd)
    }

    fn recv_command() -> Result<StageMotorCmd, TryReceiveError> {
        if STAGE_MOTOR.stop.signaled() {
            STAGE_MOTOR.stop.reset();
            return Err(TryReceiveError::Empty);
        }

        STAGE_MOTOR.commands.try_receive()
    }

    fn is_at_lower_limit(&self) -> bool {
        if let Some(limit) = self.lower_limit {
            return self.position <= limit;
        }

        self.end_stop_lower_triggered()
    }

    fn is_at_upper_limit(&self) -> bool {
        if let Some(limit) = self.upper_limit {
            return self.position >= limit;
        }
        false
    }

    fn end_stop_lower_triggered(&self) -> bool {
        if let Some(pin) = &self.end_stop_lower {
            pin.is_high()
        } else {
            false
        }
    }

    fn enable(&mut self) {
        if let Some(en) = &mut self.en_pin {
            en.set_low();
        }
    }

    fn disable(&mut self) {
        if let Some(en) = &mut self.en_pin {
            en.set_high();
        }
    }

    fn request_stop() {
        STAGE_MOTOR.stop.signal(());
        STAGE_MOTOR.commands.clear();
    }

    fn stop() -> bool {
        STAGE_MOTOR.stop.signaled()
    }

    fn reset_stop() {
        STAGE_MOTOR.stop.reset();
    }

    fn set_lower_limit(&mut self) {
        self.lower_limit = Some(self.position);
    }

    fn set_upper_limit(&mut self) {
        self.upper_limit = Some(self.position);
    }

    fn release_limits(&mut self) {
        self.relase_lower_limit();
        self.relase_upper_limit();
    }

    fn relase_lower_limit(&mut self) {
        self.lower_limit = None;
    }

    fn relase_upper_limit(&mut self) {
        self.upper_limit = None;
    }

    async fn home(&mut self) -> Result<(), StageMotorError> {
        if self.end_stop_lower.is_none() {
            return Err(StageMotorError::LowerEndStopNotAvailable);
        }

        self.relase_lower_limit();

        loop {
            let err = self.steps(-20, 800).await;

            if let Err(StageMotorError::LowerLimitReached) = err {
                break;
            } else if err.is_err() {
                return err;
            }
        }

        self.steps(300, 1000).await?;

        while !self.end_stop_lower_triggered() {
            let err = self.steps(-1, 4000).await;

            if let Err(StageMotorError::LowerLimitReached) = err {
                break;
            } else if err.is_err() {
                return err;
            }
        }

        self.position = 0;
        self.set_lower_limit();

        Ok(())
    }

    async fn steps(&mut self, n: i32, delay_us: u64) -> Result<(), StageMotorError> {
        let delta = if n > 0 {
            self.dir_pin.set_high();
            1
        } else {
            self.dir_pin.set_low();
            -1
        };

        for _ in 0..n.abs() {
            if StageMotor::stop() {
                return Err(StageMotorError::CommandAborted);
            }

            if self.is_at_lower_limit() && delta < 0 {
                return Err(StageMotorError::LowerLimitReached);
            }

            if self.is_at_upper_limit() && delta > 0 {
                return Err(StageMotorError::UpperLimitReached);
            }

            self.step_pin.set_high();
            Timer::after(Duration::from_micros(delay_us)).await;
            self.step_pin.set_low();
            Timer::after(Duration::from_micros(delay_us)).await;

            self.position += delta;
        }

        Ok(())
    }
}

#[embassy_executor::task]
pub async fn motor_task(
    step: Output<'static>,
    dir: Output<'static>,
    en: Option<Output<'static>>,
    end_stop_lower: Option<Input<'static>>,
    _ms1: Output<'static>,
    _ms2: Output<'static>,
) {
    let mut state = StageMotor {
        position: 0,
        lower_limit: None,
        upper_limit: None,
        end_stop_lower: end_stop_lower,
        en_pin: en,
        step_pin: step,
        dir_pin: dir,
    };

    state.enable();
    if let Err(err) = state.home().await {
        send_error(&format!(256; "Homing error: {}", err).unwrap());
    }

    loop {
        StageMotor::reset_stop();

        match StageMotor::recv_command() {
            Ok(StageMotorCmd::Enable) => {
                state.enable();
                send_info("Stage motor enabled.");
            }
            Ok(StageMotorCmd::Disable) => {
                state.disable();
                send_info("Stage motor disabled.");
            }
            Ok(StageMotorCmd::Home) => {
                send_info("Homing stage.");
                if let Err(err) = state.home().await {
                    send_error(&format!(256; "Homing error: {}", err).unwrap());
                } else {
                    send_info("Stage homed to lower limit.");
                }
            }
            Ok(StageMotorCmd::MoveSteps {
                steps,
                step_delay_us,
            }) => {
                send_info("Moving stage along Z axis.");
                if let Err(err) = state.steps(steps, step_delay_us as u64).await {
                    send_error(&format!(256; "Stage movement error: {}", err).unwrap());
                } else {
                    send_info("Stage moved to requested position.");
                }
            }
            Ok(StageMotorCmd::SetLowerLimitToCurrent) => {
                state.set_lower_limit();
                send_info("Set current position as lower limit.");
            }
            Ok(StageMotorCmd::SetUpperLimitToCurrent) => {
                state.set_upper_limit();
                send_info("Set current position as upper limit.");
            }
            Ok(StageMotorCmd::SetLowerLimit(limit)) => {
                state.lower_limit = Some(limit);
                send_info("Set lower limit.");
            }
            Ok(StageMotorCmd::SetUpperLimit(limit)) => {
                state.upper_limit = Some(limit);
                send_info("Set upper limit.");
            }
            Ok(StageMotorCmd::ReleaseLimits) => {
                state.release_limits();
                send_info("Released stage limits.");
            }
            Ok(StageMotorCmd::GoToLowerLimit { step_delay_us }) => {
                if let Some(limit) = state.lower_limit {
                    let steps = limit - state.position;
                    send_info("Moving stage to lower limit.");
                    if let Err(err) = state.steps(steps, step_delay_us as u64).await {
                        send_error(&format!(256; "Stage movement error: {}", err).unwrap());
                    } else {
                        send_info("Stage moved to lower limit.");
                    }
                } else {
                    send_warning("Lower limit not set, cannot move to lower limit.");
                }
            }
            Ok(StageMotorCmd::GoToUpperLimit { step_delay_us }) => {
                if let Some(limit) = state.upper_limit {
                    let steps = limit - state.position;
                    send_info("Moving stage to upper limit.");
                    if let Err(err) = state.steps(steps, step_delay_us as u64).await {
                        send_error(&format!(256; "Stage movement error: {}", err).unwrap());
                    } else {
                        send_info("Stage moved to upper limit.");
                    }
                } else {
                    send_warning("Upper limit not set, cannot move to upper limit.");
                }
            }
            Ok(StageMotorCmd::ReportPosition) => {
                send_device_event(interface::uart::DeviceEvent::StageMotorPosition {
                    position_steps: state.position,
                });
            }
            Ok(StageMotorCmd::Stop) => { /* Handled in send_command */ }
            Err(_) => { /* No command received */ }
        }

        Timer::after(Duration::from_millis(10)).await;
    }
}
