#[cfg(feature = "std")]
extern crate std;

use std::string::String;
use std::vec::Vec;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ComputeNodeCapabilities {}

#[derive(Serialize, Deserialize)]
pub struct ComputeNodeAnnouncement {
    pub node_id: String,
    pub capabilities: ComputeNodeCapabilities,
}

#[derive(Serialize, Deserialize)]
pub enum WebSocketMessage<Params> {
    UpdateParameters(Params),
    Logs(Vec<String>),
    RegisterComputeNode(ComputeNodeCapabilities),
    AnnounceComputeNode(ComputeNodeAnnouncement),
}
