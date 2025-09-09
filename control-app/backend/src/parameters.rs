use crate::camera::CameraProperties;
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

#[derive(Clone, Serialize, Deserialize)]
pub struct Parameters {
    pub camera_properties: CameraProperties,
}

impl Parameters {
    pub fn new() -> Self {
        Parameters {
            camera_properties: CameraProperties {
                exposure_time: Some(4000),
                gain: Some(1.0),
                brightness: Some(0.0),
                contrast: Some(1.0),
                saturation: Some(1.0),
                sharpness: Some(0),
                auto_white_balance: Some(true),
                white_balance_mode: Some(crate::camera::WhiteBalanceMode::Auto),
                color_gain_red: Some(1.0),
                color_gain_blue: Some(1.0),
                test_pattern: None,
            },
        }
    }

    fn patch(&mut self, other: &Self) -> usize {
        self.camera_properties.patch(&other.camera_properties)
    }
}

pub struct ParametersController {
    pub parameters: Parameters,
    notify_channel: watch::Sender<Parameters>,
}

impl ParametersController {
    pub fn new() -> Self {
        let (notify_channel, _) = watch::channel(Parameters::new());

        ParametersController {
            parameters: Parameters::new(),
            notify_channel,
        }
    }

    pub fn subscribe_changes(&self) -> watch::Receiver<Parameters> {
        self.notify_channel.subscribe()
    }

    pub fn patch(&mut self, other: &Parameters) {
        let changes = self.parameters.patch(&other);

        if changes > 0 {
            let _ = self.notify_channel.send(self.parameters.clone());
        }
    }
}
