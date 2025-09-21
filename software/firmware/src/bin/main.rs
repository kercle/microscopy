#![no_std]
#![no_main]

// MS1 GPIO32
// MS2 GPIO33
// DIR GPIO25
// STEP GPIO26

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    main,
};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    println!("Hello world!");

    // Set GPIO7 as an output, and set its state high initially.
    let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

    let mut step = Output::new(peripherals.GPIO26, Level::Low, OutputConfig::default());
    let mut dir = Output::new(peripherals.GPIO25, Level::High, OutputConfig::default());
    let _ms1 = Output::new(peripherals.GPIO32, Level::Low, OutputConfig::default());
    let _ms2 = Output::new(peripherals.GPIO33, Level::Low, OutputConfig::default());

    dir.set_high();

    for _ in 0..100 {
        step.set_high();
        Delay::new().delay_millis(10);
        step.set_low();
        Delay::new().delay_millis(10);
    }

    led.set_high();

    // Initialize the Delay peripheral, and use it to toggle the LED state in a
    // loop.
    let delay = Delay::new();

    loop {
        led.toggle();
        delay.delay_millis(2000);
    }
}