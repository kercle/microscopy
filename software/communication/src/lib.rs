#![no_std]

const COM_INIT_STR: &str = "MICROSCOPE_COM_v0.1";

pub enum Axes {
    X,
    Y,
    Z,
}

pub enum Error {
    BufferTooSmall,
    InvalidCommand,
}

pub enum Commands {
    Init,
    MoveSteps {
        axis: Axes,
        steps: i32,
        step_delay_us: u32,
    },
}

impl Commands {
    pub fn from_bytes(_data: &[u8]) -> Option<Self> {
        // Placeholder for command parsing logic
        None
    }

    pub fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        match self {
            Commands::Init => {
                let msg = COM_INIT_STR.as_bytes();
                if buffer.len() >= msg.len() {
                    buffer[..msg.len()].copy_from_slice(msg);
                    Ok(msg.len())
                } else {
                    Err(Error::BufferTooSmall)
                }
            }
            Commands::MoveSteps {
                axis: _,
                steps: _,
                step_delay_us: _,
            } => {
                buffer[..1].copy_from_slice(b"S");
                Ok(1)
            }
        }
    }
}

pub fn test() {}
