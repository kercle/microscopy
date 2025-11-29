use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::ws::{compute_node::TaskUiDescription, value::Value};

#[cfg(feature = "ts")]
use {std::format, std::string::ToString, ts_rs::TS};

pub mod compute_node;
pub mod logs;
pub mod parameters;
pub mod value;

use compute_node::{ComputeNode, ComputeNodeCapabilities};
use logs::LogEntry;
use parameters::Parameters;

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde_with::skip_serializing_none]
#[serde(rename_all = "snake_case")]
pub enum WebSocketMessage {
    RegisterUserClient,
    RegisterComputeNode(ComputeNodeCapabilities),

    UpdateParameters(Parameters),
    ComputeNodes(Vec<ComputeNode>),
    WithTaskParams {
        task_name: String,
        source_uuid: Option<String>,
        destination_uuid: String,
        params: HashMap<String, Value>,
    },
    TaskDescription {
        task_name: String,
        source_uuid: Option<String>,
        destination_uuid: String,
        ui_description: TaskUiDescription,
    },
    StartTask {
        compute_node_uuid: String,
        task_name: String,
        params: HashMap<String, Value>,
    },

    Logs(Vec<LogEntry>),
}
