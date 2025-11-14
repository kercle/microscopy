use std::vec::Vec;

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use {std::format, std::string::ToString, ts_rs::TS};

pub mod compute_node;
pub mod logs;
pub mod parameters;

use compute_node::{ComputeNode, ComputeNodeCapabilities};
use parameters::Parameters;
use logs::LogEntry;

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketMessage {
    RegisterUserClient,
    RegisterComputeNode(ComputeNodeCapabilities),

    UpdateParameters(Parameters),
    ComputeNodeAnnouncement(Vec<ComputeNode>),

    Logs(Vec<LogEntry>),
}
