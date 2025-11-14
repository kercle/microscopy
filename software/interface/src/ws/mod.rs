use std::vec::Vec;

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use {std::format, std::string::String, std::string::ToString, ts_rs::TS};

pub mod compute_node;
pub mod logs;
pub mod parameters;

use compute_node::{ComputeNode, ComputeNodeCapabilities};
use logs::LogEntry;
use parameters::Parameters;

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketMessage {
    RegisterUserClient,
    RegisterComputeNode(ComputeNodeCapabilities),

    UpdateParameters(Parameters),
    ComputeNodes(Vec<ComputeNode>),

    Logs(Vec<LogEntry>),
}
