use std::string::String;
use chrono;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ZScanMetadata {
    pub relative_start_pos: i32,
    pub relative_stop_pos: i32,
    pub steps_between_layers: u32,
    pub frame_count: usize,
    pub uuid: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
}
