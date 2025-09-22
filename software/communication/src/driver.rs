use std::boxed::Box;
use std::path::Path;
use std::string::String;
use std::vec::Vec;

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use serialport::SerialPort;

use crate::DeviceEvent;

type EventBuffer = [u8; 4096];

pub struct DeviceDriver {
    port: Box<dyn SerialPort>,
    buffer: Vec<u8>,
    signature_received: bool,
}

impl DeviceDriver {
    pub fn new(serial_port: &Path, baud_rate: u32) -> Result<Self> {
        Ok(DeviceDriver {
            port: serialport::new(serial_port.to_str().unwrap(), baud_rate)
                .timeout(std::time::Duration::from_millis(100))
                .open()?,
            buffer: Vec::new(),
            signature_received: false,
        })
    }

    fn verify_signature<StrType: Serialize + DeserializeOwned>(&mut self) -> bool {
        if self.signature_received {
            return true;
        }

        let mut init_packet: EventBuffer = [0; 4096];
        let init_package_size = DeviceEvent::<StrType>::InitSignature
            .encode_bytes(&mut init_packet)
            .unwrap();

        for i in 0..self.buffer.len() - init_package_size + 1 {
            if self.buffer[i..].starts_with(&init_packet[..init_package_size]) {
                self.buffer.drain(..i + init_package_size + 1); // +1 to also remove the null terminator
                self.signature_received = true;
                return true;
            }
        }

        false
    }

    fn decode_next_event<StrType: Serialize + DeserializeOwned>(
        &mut self,
    ) -> Result<Option<DeviceEvent<StrType>>> {
        if !self.verify_signature::<StrType>() {
            return Ok(None);
        }

        let res = if let Some(pos) = self.buffer.iter().position(|&x| x == 0) {
            let packet = self.buffer.drain(..=pos).collect::<Vec<u8>>();
            Some(DeviceEvent::<StrType>::decode_bytes(&packet).map_err(|e| {
                anyhow::anyhow!("Failed to decode packet: {:?}, error: {:?}", packet, e)
            })?)
        } else {
            None
        };

        Ok(res)
    }

    pub fn recv_event<StrType: Serialize + DeserializeOwned>(
        &mut self,
    ) -> Result<Option<DeviceEvent<StrType>>> {
        let bytes_available = self.port.bytes_to_read()?;

        if bytes_available > 0 {
            let mut buf = Vec::with_capacity(bytes_available as usize);
            let n = self.port.read(&mut buf)?;
            self.buffer.extend_from_slice(&buf[..n]);
        }

        self.decode_next_event()
    }
}
