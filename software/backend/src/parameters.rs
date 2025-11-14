use tokio::sync::watch;

use interface::ws as com_ws;

use crate::camera::CameraPropertiesExt;

trait ParametersExt {
    fn new() -> Self;
    fn patch(&mut self, other: &Self) -> usize;
}

impl ParametersExt for com_ws::parameters::Parameters {
    fn new() -> Self {
        Self {
            camera_properties: com_ws::parameters::CameraProperties {
                exposure_time: Some(4000),
                gain: Some(1.0),
                brightness: Some(0.0),
                contrast: Some(1.0),
                saturation: Some(1.0),
                sharpness: Some(0),
                auto_white_balance: Some(true),
                white_balance_mode: Some(com_ws::parameters::WhiteBalanceMode::Auto),
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
    pub parameters: com_ws::parameters::Parameters,
    notify_channel: watch::Sender<com_ws::parameters::Parameters>,
}

impl ParametersController {
    pub fn new() -> Self {
        let (notify_channel, _) = watch::channel(com_ws::parameters::Parameters::new());

        ParametersController {
            parameters: com_ws::parameters::Parameters::new(),
            notify_channel,
        }
    }

    pub fn subscribe_changes(&self) -> watch::Receiver<com_ws::parameters::Parameters> {
        self.notify_channel.subscribe()
    }

    pub fn patch(&mut self, other: &com_ws::parameters::Parameters) {
        let changes = self.parameters.patch(other);

        if changes > 0 {
            let _ = self.notify_channel.send(self.parameters.clone());
        }
    }
}
