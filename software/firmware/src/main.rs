#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_hal::timer::timg::TimerGroup;
use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output, OutputConfig},
    uart::{Config as UartConfig, Uart, UartRx, UartTx},
};

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};

use communication::StageMotorCmd;
use heapless::{String, Vec};

type DeviceEvent = communication::DeviceEvent<String<128>>;

struct Channels {
    stage_motor: Channel<CriticalSectionRawMutex, StageMotorCmd, 8>,
    _device_events: Channel<CriticalSectionRawMutex, DeviceEvent, 16>,
}

impl Channels {
    const fn new() -> Self {
        Self {
            stage_motor: Channel::new(),
            _device_events: Channel::new(),
        }
    }

    fn send_device_event(&self, event: DeviceEvent) {
        let _ = self._device_events.try_send(event);
    }
}

static CHANNELS: Channels = Channels::new();

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[embassy_executor::task]
async fn uart_tx_task(mut tx: UartTx<'static, esp_hal::Async>) {
    let mut buffer = [0u8; 512];

    CHANNELS.send_device_event(DeviceEvent::LogMessage {
        level: communication::LogMessageLevel::Info,
        message: String::try_from("Firmware started.").unwrap(),
    });

    loop {
        match CHANNELS._device_events.try_receive() {
            Ok(event) => {
                let len = event
                    .encode_bytes(&mut buffer)
                    .expect("Buffer too small for event");
                let _ = tx.write_async(&buffer[..len]).await;
                let _ = tx.write_async(&[0u8]).await; // Null-terminate
                let _ = tx.flush_async().await;
            }
            Err(_) => {
                Timer::after(Duration::from_millis(20)).await;
            }
        }
    }
}

#[embassy_executor::task]
async fn uart_rx_task(mut rx: UartRx<'static, esp_hal::Async>) {
    CHANNELS
        ._device_events
        .try_send(DeviceEvent::InitSignature)
        .ok();

    let mut buf = [0u8; 64];
    let mut packet = Vec::<u8, 256>::new();
    loop {
        let n = rx.read_async(&mut buf).await.unwrap();

        if n == 0 {
            continue;
        }

        packet.extend_from_slice(&buf[..n]).unwrap_or(());
        let end_pos = if let Some(end_pos) = packet.iter().position(|&b| b == 0) {
            end_pos
        } else {
            // No complete packet yet
            continue;
        };

        let cmd_slice = &packet[..end_pos];
        if let Ok(cmd) = communication::HostCommand::decode_bytes(cmd_slice) {
            match cmd {
                communication::HostCommand::StageMotor(motor_cmd) => {
                    let _ = CHANNELS.stage_motor.try_send(motor_cmd);
                }
            }
        }

        // Remove the processed packet from the buffer
        packet.drain(..=end_pos);
    }
}

#[embassy_executor::task]
async fn motor_task(
    mut step: Output<'static>,
    mut dir: Output<'static>,
    _ms1: Output<'static>,
    _ms2: Output<'static>,
) {
    loop {
        match CHANNELS.stage_motor.try_receive() {
            Ok(StageMotorCmd::MoveSteps {
                steps,
                step_delay_us,
            }) => {
                let _ = CHANNELS.send_device_event(DeviceEvent::LogMessage {
                    level: communication::LogMessageLevel::Info,
                    message: String::try_from("Moving stage along Z axis.").unwrap(),
                });

                if steps > 0 {
                    dir.set_high();
                } else {
                    dir.set_low();
                }
                for _ in 0..steps.abs() {
                    step.set_high();
                    Timer::after(Duration::from_micros(step_delay_us as u64)).await;
                    step.set_low();
                    Timer::after(Duration::from_micros(step_delay_us as u64)).await;
                }

                let _ = CHANNELS.send_device_event(DeviceEvent::LogMessage {
                    level: communication::LogMessageLevel::Info,
                    message: String::try_from("Stage moved to new position.").unwrap(),
                });
            }
            _ => {}
        }

        Timer::after(Duration::from_millis(20)).await;
    }
}

#[embassy_executor::task]
async fn _debug_msg_task() {
    loop {
        Timer::after(Duration::from_secs(2)).await;
        CHANNELS.send_device_event(DeviceEvent::LogMessage {
            level: communication::LogMessageLevel::Info,
            message: String::try_from("Debug message from firmware.").unwrap(),
        });
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let uart_cfg = UartConfig::default().with_baudrate(115_200);
    let uart0 = Uart::new(peripherals.UART0, uart_cfg)
        .unwrap()
        .with_tx(peripherals.GPIO1)
        .with_rx(peripherals.GPIO3)
        .into_async();
    let (rx0, tx0) = uart0.split();

    let timer0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timer0.timer0);

    spawner.must_spawn(uart_tx_task(tx0));
    spawner.must_spawn(uart_rx_task(rx0));
    // spawner.must_spawn(_debug_msg_task());
    spawner.must_spawn(motor_task(
        Output::new(peripherals.GPIO26, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO25, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO32, Level::Low, OutputConfig::default()),
        Output::new(peripherals.GPIO33, Level::Low, OutputConfig::default()),
    ));

    esp_alloc::heap_allocator!(size: 64 * 1024);

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
