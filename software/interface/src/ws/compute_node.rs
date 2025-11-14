use std::string::String;

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use {std::borrow::ToOwned, std::format, std::string::ToString, ts_rs::TS};

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Clone, Serialize, Deserialize)]
pub struct ComputeNodeCapabilities {}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Clone, Serialize, Deserialize)]
pub struct ComputeNodeAnnouncement {
    pub node_id: String,
    pub capabilities: ComputeNodeCapabilities,
}
