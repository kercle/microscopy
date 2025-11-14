use std::format;
use std::string::{String, ToString};
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS)]
#[ts(export)]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Parameters {
    pub camera_properties: CameraProperties,
}

#[derive(TS)]
#[ts(export)]
#[ts(optional_fields)]
#[serde_with::skip_serializing_none]
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct CameraProperties {
    pub exposure_time: Option<u32>,
    pub gain: Option<f64>,
    pub brightness: Option<f32>,
    pub contrast: Option<f32>,
    pub saturation: Option<f32>,
    pub sharpness: Option<i32>,
    pub auto_white_balance: Option<bool>,
    pub white_balance_mode: Option<WhiteBalanceMode>,
    pub color_gain_red: Option<f32>,
    pub color_gain_blue: Option<f32>,
    pub test_pattern: Option<TestPattern>,
}

#[derive(TS)]
#[ts(export)]
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum WhiteBalanceMode {
    Auto = 0,
    Incandescent = 1,
    Tungsten = 2,
    Fluorescent = 3,
    Indoor = 4,
    Daylight = 5,
    Cloudy = 6,
    Custom = 7,
}

impl Display for WhiteBalanceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Incandescent => write!(f, "incandescent"),
            Self::Tungsten => write!(f, "tungsten"),
            Self::Fluorescent => write!(f, "fluorescent"),
            Self::Indoor => write!(f, "indoor"),
            Self::Daylight => write!(f, "daylight"),
            Self::Cloudy => write!(f, "cloudy"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

#[derive(TS)]
#[ts(export)]
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TestPattern {
    Smpte = 0,
    Snow = 1,
    Ball = 18,
}

impl Display for TestPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Smpte => write!(f, "smpte"),
            Self::Snow => write!(f, "snow"),
            Self::Ball => write!(f, "ball"),
        }
    }
}
