use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel},
};

use heapless::String;

pub type DeviceEvent = communication::DeviceEvent<String<128>>;

pub struct Channels {
    pub device_events: Channel<CriticalSectionRawMutex, DeviceEvent, 16>,
}

impl Channels {
    const fn new() -> Self {
        Self {
            device_events: Channel::new(),
        }
    }

    pub fn send_device_event(&self, event: DeviceEvent) {
        let _ = self.device_events.try_send(event);
    }
}

// TODO: Make this private
pub static CHANNELS: Channels = Channels::new();

pub fn send_device_event(event: DeviceEvent) {
    let _ = CHANNELS.send_device_event(event);
}

pub fn send_info(message: &str) {
    send_device_event(DeviceEvent::LogMessage {
        level: communication::LogMessageLevel::Info,
        message: String::try_from(message).unwrap(),
    });
}

pub fn send_warning(message: &str) {
    send_device_event(DeviceEvent::LogMessage {
        level: communication::LogMessageLevel::Warning,
        message: String::try_from(message).unwrap(),
    });
}

pub fn send_error(message: &str) {
    send_device_event(DeviceEvent::LogMessage {
        level: communication::LogMessageLevel::Error,
        message: String::try_from(message).unwrap(),
    });
}
