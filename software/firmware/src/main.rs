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
};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut step = Output::new(peripherals.GPIO26, Level::Low, OutputConfig::default());
    let _dir = Output::new(peripherals.GPIO25, Level::Low, OutputConfig::default());
    let _ms1 = Output::new(peripherals.GPIO32, Level::Low, OutputConfig::default());
    let _ms2 = Output::new(peripherals.GPIO33, Level::Low, OutputConfig::default());

    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timer0 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    for _ in 0..100 {
        step.set_high();
        Timer::after(Duration::from_millis(1)).await;
        step.set_low();
        Timer::after(Duration::from_millis(1)).await;
    }

    let _ = spawner;

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
