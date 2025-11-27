use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use {std::format, std::string::ToString, ts_rs::TS};

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input {
    Selection {
        display_name: String,
        options: Vec<String>,
        value: String,
    },
    ImagePreview {
        display_name: String,
        href: String,
    },
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Output {
    Image { display_name: String },
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementPositioning {
    pub row: u32,
    pub column: u32,
    pub row_span: u32,
    pub column_span: u32,
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Element {
    Image {
        display_name: String,
        href: String,
        positioning: ElementPositioning,
    },
    Select {
        display_name: String,
        options: Vec<String>,
        value: String,
        positioning: ElementPositioning,
    },
    Button {
        display_name: String,
        positioning: ElementPositioning,
    },
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureUi {
    pub display_name: String,
    pub description: String,
    pub columns: u32,
    pub elements: HashMap<String, Element>,
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeNodeCapabilities {
    pub procedures: HashMap<String, ProcedureUi>,
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeNode {
    pub node_id: String,
    pub capabilities: ComputeNodeCapabilities,
}
