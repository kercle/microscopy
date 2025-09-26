#![no_std]

use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
mod parse;
#[cfg(feature = "host")]
pub mod driver;

mod bytes_repr;

const COM_INIT_STR: &str = "MICROSCOPE_COM_v0.1";

#[derive(Debug)]
pub enum Error {
    BufferTooSmall,
    InvalidCommand,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LogMessageLevel {
    Info,
    Warning,
    Error,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DeviceEvent<StrType> {
    InitSignature,
    LogMessage { level: LogMessageLevel, message: StrType },
    StageMotorPosition { position_steps: i32 },
}

impl<StrType: Serialize + DeserializeOwned> DeviceEvent<StrType> {
    pub fn encode_bytes(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        match self {
            DeviceEvent::InitSignature => {
                let msg = COM_INIT_STR.as_bytes();
                if buffer.len() >= msg.len() {
                    buffer[..msg.len()].copy_from_slice(msg);
                    Ok(msg.len())
                } else {
                    Err(Error::BufferTooSmall)
                }
            }
            _ => bytes_repr::encode_bytes(self, buffer),
        }
    }

    pub fn decode_bytes(data: &[u8]) -> Result<Self, Error> {
        bytes_repr::decode_bytes(data)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StageMotorCmd {
    Enable,
    Disable,
    Home,
    MoveSteps { steps: i32, step_delay_us: u32 },
    Stop,
    SetLowerLimitToCurrent,
    SetUpperLimitToCurrent,
    SetLowerLimit(i32),
    SetUpperLimit(i32),
    ReleaseLimits,
    GoToLowerLimit { step_delay_us: u32 },
    GoToUpperLimit { step_delay_us: u32 },
    ReportPosition,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HostCommand {
    StageMotor(StageMotorCmd),
}

impl HostCommand {
    pub fn from_bytes(_data: &[u8]) -> Option<Self> {
        // Placeholder for command parsing logic
        None
    }

    pub fn encode_bytes(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        bytes_repr::encode_bytes(self, buffer)
    }

    pub fn decode_bytes(data: &[u8]) -> Result<Self, Error> {
        bytes_repr::decode_bytes(data)
    }
}

pub fn test() {}
