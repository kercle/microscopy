use std::fmt;
use std::str::FromStr;
use std::string::{String, ToString};

use regex::Regex;

use crate::{HostCommand, StageMotorCmd};

impl fmt::Display for StageMotorCmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StageMotorCmd::MoveSteps {
                steps,
                step_delay_us,
            } => {
                write!(f, "steps:{},{}", steps, step_delay_us)
            }
        }
    }
}

impl FromStr for StageMotorCmd {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r = Regex::new(r"^steps:(-?\d+),(\d+)$").map_err(|e| e.to_string())?;

        if let Some(caps) = r.captures(s) {
            let steps = caps
                .get(1)
                .ok_or("Missing steps")?
                .as_str()
                .parse::<i32>()
                .map_err(|e| e.to_string())?;
            let step_delay_us = caps
                .get(2)
                .ok_or("Missing step_delay_us")?
                .as_str()
                .parse::<u32>()
                .map_err(|e| e.to_string())?;
            Ok(StageMotorCmd::MoveSteps {
                steps,
                step_delay_us,
            })
        } else {
            Err("Invalid command format".to_string())
        }
    }
}

impl fmt::Display for HostCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HostCommand::StageMotor(cmd) => write!(f, "Z[{}]", cmd),
        }
    }
}

impl FromStr for HostCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r = Regex::new(r"^Z\[(.+)\]$").map_err(|e| e.to_string())?;

        if let Some(caps) = r.captures(s) {
            let cmd_str = caps.get(1).ok_or("Missing command")?.as_str();
            let cmd = StageMotorCmd::from_str(cmd_str)?;
            Ok(HostCommand::StageMotor(cmd))
        } else {
            Err("Invalid command format".to_string())
        }
    }
}
