use std::borrow::ToOwned;
use std::format;
use std::string::{String, ToString};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS)]
#[ts(export)]
#[derive(Serialize, Deserialize)]
pub struct ComputeNodeCapabilities {}

#[derive(TS)]
#[ts(export)]
#[derive(Serialize, Deserialize)]
pub struct ComputeNodeAnnouncement {
    pub node_id: String,
    pub capabilities: ComputeNodeCapabilities,
}
