use std::fmt;
use std::str::FromStr;
use std::string::{String, ToString};

use regex::Regex;

use crate::{HostCommand, StageMotorCmd};

impl StageMotorCmd {
    fn parse_move_steps(s: &str) -> Option<Self> {
        let r = Regex::new(r"^steps:(-?\d+),(\d+)$").unwrap();

        if let Some(caps) = r.captures(s) {
            let steps = if let Some(step_str) = caps.get(1) {
                step_str.as_str().parse::<i32>().ok()
            } else {
                return None;
            };

            let step_delay_us = if let Some(delay_str) = caps.get(2) {
                delay_str.as_str().parse::<u32>().ok()
            } else {
                return None;
            };

            if let (Some(steps), Some(step_delay_us)) = (steps, step_delay_us) {
                Some(StageMotorCmd::MoveSteps {
                    steps,
                    step_delay_us,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

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
        if let Some(cmd) = StageMotorCmd::parse_move_steps(s) {
            Ok(cmd)
        } else {
            Err("Invalid StageMotorCmd format".to_string())
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
