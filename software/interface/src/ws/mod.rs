use std::format;
use std::string::{String, ToString};
use std::vec::Vec;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod compute_node;
pub mod parameters;
pub mod logs;

#[derive(TS)]
#[ts(export)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketMessage {
    UpdateParameters(parameters::Parameters),
    Logs(Vec<logs::LogEntry>),
    RegisterComputeNode(compute_node::ComputeNodeCapabilities),
    AnnounceComputeNode(compute_node::ComputeNodeAnnouncement),
}
