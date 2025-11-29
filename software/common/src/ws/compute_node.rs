use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use {std::format, std::string::ToString, ts_rs::TS};

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub row: u32,
    pub column: u32,
    pub row_span: u32,
    pub column_span: u32,
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Widget {
    Image {
        display_name: String,
        href: String,
        positioning: WidgetPosition,
    },
    Select {
        display_name: String,
        options: Vec<String>,
        value: String,
        positioning: WidgetPosition,
    },
    Button {
        display_name: String,
        positioning: WidgetPosition,
    },
    Slider {
        display_name: String,
        min: f64,
        max: f64,
        step: f64,
        value: f64,
        positioning: WidgetPosition,
    },
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUiDescription {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub columns: u32,
    pub progress: Option<f32>,
    pub locked: bool,
    pub elements: HashMap<String, Widget>,
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeNodeCapabilities {
    pub tasks: HashMap<String, TaskUiDescription>,
}

#[cfg_attr(feature = "ts", derive(TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeNode {
    pub node_id: String,
    pub capabilities: ComputeNodeCapabilities,
}
