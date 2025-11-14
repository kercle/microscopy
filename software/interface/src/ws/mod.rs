use std::vec::Vec;

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use {
    std::format,
    std::string::{String, ToString},
    ts_rs::TS,
};

pub mod compute_node;
pub mod logs;
pub mod parameters;

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketMessage {
    RegisterUserClient,
    RegisterComputeNode(compute_node::ComputeNodeCapabilities),

    UpdateParameters(parameters::Parameters),

    Logs(Vec<logs::LogEntry>),
}
