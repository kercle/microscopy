#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
mod parse;

const COM_INIT_STR: &str = "MICROSCOPE_COM_v0.1";

pub enum Error {
    BufferTooSmall,
    InvalidCommand,
}

pub enum DeviceEvent {
    InitSignature,
    StageMotorPosition { position_steps: i32 },
}

impl DeviceEvent {
    pub fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, Error> {
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
            DeviceEvent::StageMotorPosition { position_steps: _ } => {
                buffer[..1].copy_from_slice(b"P");
                Ok(1)
            }
        }
    }
}

pub enum StageMotorCmd {
    MoveSteps { steps: i32, step_delay_us: u32 },
}

pub enum HostCommand {
    StageMotor(StageMotorCmd),
}

impl HostCommand {
    pub fn from_bytes(_data: &[u8]) -> Option<Self> {
        // Placeholder for command parsing logic
        None
    }

    pub fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        match self {
            HostCommand::StageMotor(StageMotorCmd::MoveSteps {
                steps: _,
                step_delay_us: _,
            }) => {
                buffer[..1].copy_from_slice(b"S");
                Ok(1)
            }
        }
    }
}

pub fn test() {}
