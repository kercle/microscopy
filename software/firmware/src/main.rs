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
use embassy_time::{Duration, Timer};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[embassy_executor::task]
async fn uart_echo_task(
    mut rx: UartRx<'static, esp_hal::Async>,
    mut tx: UartTx<'static, esp_hal::Async>,
) {
    let _ = tx.write(b"BOOT\r\n");
    let _ = tx.flush();

    let mut buf = [0u8; 64];
    loop {
        let n = rx.read_async(&mut buf).await.unwrap();
        if n > 0 {
            // simple echo
            let _ = tx.write_async(b"Received: ").await;
            let _ = tx.write_async(&buf[..n]).await;
            let _ = tx.write_async(b"\r\n").await;
            let _ = tx.flush_async().await;
        }
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
    spawner.must_spawn(uart_echo_task(rx0, tx0));

    let timer0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timer0.timer0);

    let _step = Output::new(peripherals.GPIO26, Level::Low, OutputConfig::default());
    let _dir = Output::new(peripherals.GPIO25, Level::Low, OutputConfig::default());
    let _ms1 = Output::new(peripherals.GPIO32, Level::Low, OutputConfig::default());
    let _ms2 = Output::new(peripherals.GPIO33, Level::Low, OutputConfig::default());

    esp_alloc::heap_allocator!(size: 64 * 1024);

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
