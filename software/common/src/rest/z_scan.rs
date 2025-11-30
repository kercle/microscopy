use std::string::String;
use chrono;

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use {std::format, std::string::ToString, ts_rs::TS};

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Serialize, Deserialize)]
pub struct ZScanMetadata {
    pub relative_start_pos: i32,
    pub relative_stop_pos: i32,
    pub steps_between_layers: u32,
    pub frame_count: usize,
    pub uuid: String,

    #[cfg_attr(feature = "ts", ts(as = "String"))]
    pub timestamp: chrono::DateTime<chrono::Local>,
}
