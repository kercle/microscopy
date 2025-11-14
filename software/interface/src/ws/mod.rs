use std::vec::Vec;

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use {
    std::format,
    std::string::{String, ToString},
    ts_rs::TS,
};

pub mod compute_node;
pub mod parameters;
pub mod logs;

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketMessage {
    UpdateParameters(parameters::Parameters),
    Logs(Vec<logs::LogEntry>),
    RegisterComputeNode(compute_node::ComputeNodeCapabilities),
    AnnounceComputeNode(compute_node::ComputeNodeAnnouncement),
}
