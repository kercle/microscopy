use serde::{Deserialize, Serialize};
use std::string::String;

#[cfg(feature = "ts")]
use {std::format, std::string::ToString, ts_rs::TS};

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}
