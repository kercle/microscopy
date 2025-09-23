use communication::StageMotorCmd;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, TryReceiveError, TrySendError},
    signal::Signal,
};
use embassy_time::{Duration, Timer};
use esp_hal::gpio::Output;

use crate::com::{send_error, send_info, send_warning};

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
    SoftLowerLimitReached,
    SoftUpperLimitReached,
}

pub struct StageMotor {
    position: i32,
    lower_limit: Option<i32>,
    upper_limit: Option<i32>,

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

    fn request_stop() {
        STAGE_MOTOR.stop.signal(());
        STAGE_MOTOR.commands.clear();
    }

    fn stop() -> bool {
        let stopped = STAGE_MOTOR.stop.signaled();
        STAGE_MOTOR.stop.reset();
        stopped
    }

    fn set_lower_limit(&mut self) {
        self.lower_limit = Some(self.position);
    }

    fn set_upper_limit(&mut self) {
        self.upper_limit = Some(self.position);
    }

    fn release_limits(&mut self) {
        self.lower_limit = None;
        self.upper_limit = None;
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

            if let Some(limit) = self.lower_limit {
                if self.position <= limit && delta < 0 {
                    return Err(StageMotorError::SoftLowerLimitReached);
                }
            }

            if let Some(limit) = self.upper_limit {
                if self.position >= limit && delta > 0 {
                    return Err(StageMotorError::SoftUpperLimitReached);
                }
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
    _ms1: Output<'static>,
    _ms2: Output<'static>,
) {
    let mut state = StageMotor {
        position: 0,
        lower_limit: None,
        upper_limit: None,
        step_pin: step,
        dir_pin: dir,
    };

    loop {
        match StageMotor::recv_command() {
            Ok(StageMotorCmd::MoveSteps {
                steps,
                step_delay_us,
            }) => {
                send_info("Moving stage along Z axis.");
                match state.steps(steps, step_delay_us as u64).await {
                    Err(StageMotorError::CommandAborted) => {
                        send_warning("Stage movement aborted.");
                    }
                    Err(StageMotorError::SoftLowerLimitReached) => {
                        send_error("Stage movement aborted: reached lower limit.");
                    }
                    Err(StageMotorError::SoftUpperLimitReached) => {
                        send_error("Stage movement aborted: reached upper limit.");
                    }
                    Ok(()) => {
                        send_info("Stage moved to requested position.");
                    }
                }
            }
            Ok(StageMotorCmd::SetLowerLimit) => {
                state.set_lower_limit();
                send_info("Set current position as lower limit.");
            }
            Ok(StageMotorCmd::SetUpperLimit) => {
                state.set_upper_limit();
                send_info("Set current position as upper limit.");
            }
            Ok(StageMotorCmd::ReleaseLimits) => {
                state.release_limits();
                send_info("Released stage limits.");
            }
            Ok(StageMotorCmd::GoToLowerLimit { step_delay_us }) => {
                if let Some(limit) = state.lower_limit {
                    let steps = limit - state.position;
                    send_info("Moving stage to lower limit.");
                    match state.steps(steps, step_delay_us as u64).await {
                        Err(StageMotorError::CommandAborted) => {
                            send_warning("Stage movement aborted.");
                        }
                        Ok(()) => {
                            send_info("Stage moved to lower limit.");
                        }
                        _ => {}
                    }
                } else {
                    send_warning("Lower limit not set, cannot move to lower limit.");
                }
            }
            Ok(StageMotorCmd::GoToUpperLimit { step_delay_us }) => {
                if let Some(limit) = state.upper_limit {
                    let steps = limit - state.position;
                    send_info("Moving stage to upper limit.");
                    match state.steps(steps, step_delay_us as u64).await {
                        Err(StageMotorError::CommandAborted) => {
                            send_warning("Stage movement aborted.");
                        }
                        Ok(()) => {
                            send_info("Stage moved to upper limit.");
                        }
                        _ => {}
                    }
                } else {
                    send_warning("Upper limit not set, cannot move to upper limit.");
                }
            }
            _ => {}
        }

        Timer::after(Duration::from_millis(20)).await;
    }
}
