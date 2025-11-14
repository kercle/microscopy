#[cfg(feature = "std")]
extern crate std;

use std::boxed::Box;
use std::path::Path;
use std::vec::Vec;

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use serialport::SerialPort;

use crate::uart::{DeviceEvent, HostCommand};

type EventBuffer = [u8; 4096];

pub struct DeviceDriver {
    pub port: Box<dyn SerialPort>,
    pub buffer: Vec<u8>,
    pub signature_received: bool,
}

impl DeviceDriver {
    pub fn new(serial_port: &Path, baud_rate: u32) -> Result<Self> {
        Ok(DeviceDriver {
            port: serialport::new(serial_port.to_str().unwrap(), baud_rate)
                .timeout(std::time::Duration::from_millis(10))
                .open()?,
            buffer: Vec::new(),
            signature_received: false,
        })
    }

    pub async fn reset(&mut self) -> serialport::Result<()> {
        self.port.write_data_terminal_ready(false)?;

        self.port.write_request_to_send(true)?;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        self.port.write_request_to_send(false)?;
        tokio::time::sleep(std::time::Duration::from_millis(200)).await; // wait for device to reboot
        Ok(())
    }

    fn verify_signature<StrType: Serialize + DeserializeOwned>(&mut self) -> bool {
        let mut init_packet: EventBuffer = [0; 4096];
        let init_package_size = DeviceEvent::<StrType>::InitSignature
            .encode_bytes(&mut init_packet)
            .unwrap();

        if self.buffer.len() < init_package_size + 1 {
            return false;
        }

        for i in 0..self.buffer.len() - init_package_size + 1 {
            if self.buffer[i..].starts_with(&init_packet[..init_package_size]) {
                self.buffer.drain(..i + init_package_size + 1); // +1 to also remove the null terminator
                self.signature_received = true;
                return true;
            }
        }

        false
    }

    pub fn extend_buffer(&mut self) -> Result<usize> {
        let bytes_available = self.port.bytes_to_read()?;
        if bytes_available == 0 {
            return Ok(0);
        }

        let mut buf = std::vec![0u8; bytes_available as usize];
        let n = self
            .port
            .read(&mut buf)
            .inspect(|&n| self.buffer.extend_from_slice(&buf[..n]))?;

        Ok(n)
    }

    pub fn connection_established<StrType: Serialize + DeserializeOwned>(&mut self) -> bool {
        if self.signature_received {
            return true;
        }

        if self.extend_buffer().is_ok() {
            self.verify_signature::<StrType>()
        } else {
            false
        }
    }

    fn decode_next_event<StrType: Serialize + DeserializeOwned>(
        &mut self,
    ) -> Result<Option<DeviceEvent<StrType>>> {
        if !self.connection_established::<StrType>() {
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
        self.extend_buffer()?;
        self.decode_next_event()
    }

    pub async fn send_command<StrType: Serialize + DeserializeOwned>(
        &mut self,
        cmd: HostCommand,
    ) -> Result<()> {
        if !self.connection_established::<StrType>() {
            return Err(anyhow::anyhow!(
                "Connection not established. Cannot send command."
            ));
        }

        let mut buffer: EventBuffer = [0; 4096];
        let packet_size = cmd
            .encode_bytes(&mut buffer)
            .map_err(|e| anyhow::anyhow!("Failed to encode command: {:?}", e))?;
        self.port.write_all(&buffer[..packet_size])?;
        self.port.write_all(&[0u8])?; // Null-terminate
        self.port.flush()?;

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        Ok(())
    }

    pub async fn home<StrType: Serialize + DeserializeOwned>(&mut self) -> Result<()> {
        self.send_command::<StrType>(HostCommand::StageMotor(crate::uart::StageMotorCmd::Home))
            .await
    }

    pub async fn set_upper_limit<StrType: Serialize + DeserializeOwned>(
        &mut self,
        position: i32,
    ) -> Result<()> {
        self.send_command::<StrType>(HostCommand::StageMotor(
            crate::uart::StageMotorCmd::SetUpperLimit(position),
        ))
        .await
    }

    pub async fn stage_move_steps<StrType: Serialize + DeserializeOwned>(
        &mut self,
        steps: i32,
        step_delay_us: u32,
    ) -> Result<()> {
        self.send_command::<StrType>(HostCommand::StageMotor(
            crate::uart::StageMotorCmd::MoveSteps {
                steps,
                step_delay_us,
            },
        ))
        .await
    }
}
