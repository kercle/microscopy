use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use {std::format, std::string::ToString, ts_rs::TS};

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Clone, Serialize, Deserialize)]
pub enum Input {
    Selection {
        display_name: String,
        options: Vec<String>,
    },
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Clone, Serialize, Deserialize)]
pub enum Output {
    Image { display_name: String },
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Clone, Serialize, Deserialize)]
pub struct Procedure {
    pub display_name: String,
    pub description: String,
    pub inputs: HashMap<String, Input>,
    pub outputs: HashMap<String, Output>,
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Clone, Serialize, Deserialize)]
pub struct ComputeNodeCapabilities {
    pub procedures: HashMap<String, Procedure>,
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Clone, Serialize, Deserialize)]
pub struct ComputeNode {
    pub node_id: String,
    pub capabilities: ComputeNodeCapabilities,
}
