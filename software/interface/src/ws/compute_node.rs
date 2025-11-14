use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use {std::borrow::ToOwned, std::format, std::string::ToString, ts_rs::TS};

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Clone, Serialize, Deserialize)]
enum Input {
    Selection(Vec<String>),
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Clone, Serialize, Deserialize)]
enum Output {
    Image,
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Clone, Serialize, Deserialize)]
pub struct ComputeNodeCapabilities {
    inputs: HashMap<String, Input>,
    outputs: HashMap<String, Output>,
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Clone, Serialize, Deserialize)]
pub struct ComputeNode {
    pub node_id: String,
    pub capabilities: ComputeNodeCapabilities,
}
