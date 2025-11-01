use anyhow::Result;
use clap::Parser;
use communication::DeviceEvent;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::cli::{CliCommand, serve};

mod camera;
mod cli;
mod control_app;
mod handlers;
mod parameters;

async fn device_monitor(app_state: control_app::AppState) -> Result<()> {
    let driver = if let Some(driver) = &app_state.device_driver {
        driver.clone()
    } else {
        warn!("No device driver available, skipping device monitor.");
        return Ok(());
    };

    {
        let mut driver = driver.lock().await;

        driver.reset().await?; // This also runs the homing procedure
        driver.set_upper_limit::<String>(5450).await?; // dummy value for now
    }

    while !driver.lock().await.connection_established::<String>() {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    info!("Connection to device established.");

    loop {
        if let Some(event) = driver.lock().await.recv_event::<String>()? {
            match event {
                DeviceEvent::LogMessage { level, message } => match level {
                    communication::LogMessageLevel::Info => {
                        info!("Controller board log: {message}");
                    }
                    communication::LogMessageLevel::Warning => {
                        warn!("Controller board log: {message}");
                    }
                    communication::LogMessageLevel::Error => {
                        error!("Controller board log: {message}");
                    }
                },
                _ => {}
            }
        } else {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    let cli_command = CliCommand::parse();

    match cli_command {
        CliCommand::Serve(options) => {
            serve(options).await;
        }
    }
}
