use std::format;
use std::string::{String, ToString};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS)]
#[ts(export)]
#[derive(Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}
