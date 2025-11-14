#[cfg(feature = "std")]
extern crate std;

use std::fmt;
use std::str::FromStr;
use std::string::{String, ToString};

use regex::Regex;

use crate::uart::{HostCommand, StageMotorCmd};

impl StageMotorCmd {
    fn parse_enable(s: &str) -> Option<Self> {
        if s.trim() == "enable" {
            Some(StageMotorCmd::Enable)
        } else {
            None
        }
    }

    fn parse_disable(s: &str) -> Option<Self> {
        if s.trim() == "disable" {
            Some(StageMotorCmd::Disable)
        } else {
            None
        }
    }

    fn parse_home(s: &str) -> Option<Self> {
        if s.trim() == "home" {
            Some(StageMotorCmd::Home)
        } else {
            None
        }
    }

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

    fn parse_stop(s: &str) -> Option<Self> {
        if s.trim() == "stop" {
            Some(StageMotorCmd::Stop)
        } else {
            None
        }
    }

    fn parse_set_lower_limit_to_current(s: &str) -> Option<Self> {
        if s.trim() == "set_lower_limit" {
            Some(StageMotorCmd::SetLowerLimitToCurrent)
        } else {
            None
        }
    }

    fn parse_set_upper_limit_to_current(s: &str) -> Option<Self> {
        if s.trim() == "set_upper_limit" {
            Some(StageMotorCmd::SetUpperLimitToCurrent)
        } else {
            None
        }
    }

    fn parse_release_limits_to_current(s: &str) -> Option<Self> {
        if s.trim() == "release_limits" {
            Some(StageMotorCmd::ReleaseLimits)
        } else {
            None
        }
    }

    fn parse_set_upper_limit(s: &str) -> Option<Self> {
        let r = Regex::new(r"^set_upper_limit:(-?\d+)$").unwrap();

        if let Some(caps) = r.captures(s) {
            let limit = if let Some(limit_str) = caps.get(1) {
                limit_str.as_str().parse::<i32>().ok()
            } else {
                return None;
            };

            limit.map(StageMotorCmd::SetUpperLimit)
        } else {
            None
        }
    }

    fn parse_set_lower_limit(s: &str) -> Option<Self> {
        let r = Regex::new(r"^set_lower_limit:(-?\d+)$").unwrap();

        if let Some(caps) = r.captures(s) {
            let limit = if let Some(limit_str) = caps.get(1) {
                limit_str.as_str().parse::<i32>().ok()
            } else {
                return None;
            };

            limit.map(StageMotorCmd::SetLowerLimit)
        } else {
            None
        }
    }

    fn parse_go_to_lower_limit(s: &str) -> Option<Self> {
        let r = Regex::new(r"^goto_lower_limit:(\d+)$").unwrap();

        if let Some(caps) = r.captures(s) {
            let step_delay_us = if let Some(delay_str) = caps.get(1) {
                delay_str.as_str().parse::<u32>().ok()
            } else {
                return None;
            };

            step_delay_us.map(|step_delay_us| StageMotorCmd::GoToLowerLimit { step_delay_us })
        } else {
            None
        }
    }

    fn parse_go_to_upper_limit(s: &str) -> Option<Self> {
        let r = Regex::new(r"^goto_upper_limit:(\d+)$").unwrap();

        if let Some(caps) = r.captures(s) {
            let step_delay_us = if let Some(delay_str) = caps.get(1) {
                delay_str.as_str().parse::<u32>().ok()
            } else {
                return None;
            };

            step_delay_us.map(|step_delay_us| StageMotorCmd::GoToUpperLimit { step_delay_us })
        } else {
            None
        }
    }

    fn parse_report_position(s: &str) -> Option<Self> {
        if s.trim() == "get_pos" {
            Some(StageMotorCmd::ReportPosition)
        } else {
            None
        }
    }
}

impl fmt::Display for StageMotorCmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StageMotorCmd::Enable => {
                write!(f, "enable")
            }
            StageMotorCmd::Disable => {
                write!(f, "disable")
            }
            StageMotorCmd::Home => {
                write!(f, "home")
            }
            StageMotorCmd::MoveSteps {
                steps,
                step_delay_us,
            } => {
                write!(f, "steps:{},{}", steps, step_delay_us)
            }
            StageMotorCmd::Stop => {
                write!(f, "stop")
            }
            StageMotorCmd::SetLowerLimitToCurrent => {
                write!(f, "set_lower_limit_to_current")
            }
            StageMotorCmd::SetUpperLimitToCurrent => {
                write!(f, "set_upper_limit_to_current")
            }
            StageMotorCmd::SetLowerLimit(limit) => {
                write!(f, "set_lower_limit:{}", limit)
            }
            StageMotorCmd::SetUpperLimit(limit) => {
                write!(f, "set_upper_limit:{}", limit)
            }
            StageMotorCmd::ReleaseLimits => {
                write!(f, "release_limits")
            }
            StageMotorCmd::GoToLowerLimit { step_delay_us } => {
                write!(f, "goto_lower_limit:{}", step_delay_us)
            }
            StageMotorCmd::GoToUpperLimit { step_delay_us } => {
                write!(f, "goto_upper_limit:{}", step_delay_us)
            }
            StageMotorCmd::ReportPosition => {
                write!(f, "report_position")
            }
        }
    }
}

impl FromStr for StageMotorCmd {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(cmd) = StageMotorCmd::parse_move_steps(s) {
            Ok(cmd)
        } else if let Some(cmd) = StageMotorCmd::parse_stop(s) {
            Ok(cmd)
        } else if let Some(cmd) = StageMotorCmd::parse_release_limits_to_current(s) {
            Ok(cmd)
        } else if let Some(cmd) = StageMotorCmd::parse_set_lower_limit_to_current(s) {
            Ok(cmd)
        } else if let Some(cmd) = StageMotorCmd::parse_set_upper_limit_to_current(s) {
            Ok(cmd)
        } else if let Some(cmd) = StageMotorCmd::parse_set_upper_limit(s) {
            Ok(cmd)
        } else if let Some(cmd) = StageMotorCmd::parse_set_lower_limit(s) {
            Ok(cmd)
        } else if let Some(cmd) = StageMotorCmd::parse_go_to_lower_limit(s) {
            Ok(cmd)
        } else if let Some(cmd) = StageMotorCmd::parse_go_to_upper_limit(s) {
            Ok(cmd)
        } else if let Some(cmd) = StageMotorCmd::parse_enable(s) {
            Ok(cmd)
        } else if let Some(cmd) = StageMotorCmd::parse_disable(s) {
            Ok(cmd)
        } else if let Some(cmd) = StageMotorCmd::parse_home(s) {
            Ok(cmd)
        } else if let Some(cmd) = StageMotorCmd::parse_report_position(s) {
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
