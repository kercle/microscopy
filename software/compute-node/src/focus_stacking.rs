use std::collections::HashMap;

use anyhow::{Result, bail};
use common::ws::value::Value;
use reqwest;

use common::rest::z_scan::ZScanMetadata;
use common::ws::compute_node::{Element, ElementPositioning, ProcedureUi};

pub struct FocusStacking {}

impl FocusStacking {
    pub fn _new() -> Self {
        FocusStacking {}
    }

    async fn _list_image_stacks(host_name: &str) -> Result<Vec<String>> {
        // request to microscope_url to get actual image stacks would go here
        let url = format!("http://{host_name}/api/z-scan/list");
        let response = reqwest::get(&url).await?;

        if !response.status().is_success() {
            bail!("Failed to fetch image stacks: {}", response.status());
        }

        let response_text = response.text().await?;
        let response: Vec<ZScanMetadata> = serde_json::from_str(&response_text)?;
        Ok(response.into_iter().map(|metadata| metadata.uuid).collect())
    }

    pub async fn describe(host_name: &str, params: HashMap<String, Value>) -> ProcedureUi {
        // let image_stacks = Self::list_image_stacks(host_name).await;
        let image_stacks: std::result::Result<Vec<String>, String> =
            Ok(vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        let image_stacks = if image_stacks.is_err() {
            println!(
                "Failed to fetch image stacks: {}",
                image_stacks.err().unwrap()
            );
            vec![]
        } else {
            image_stacks.unwrap()
        };

        let selected_stack = if let Some(Value::String(selected_stack)) = params.get("image_stack") {
            if !image_stacks.contains(&selected_stack) {
                println!(
                    "Selected image stack {} not found. Defaulting to first available stack.",
                    selected_stack
                );
            }

            Some(selected_stack.clone())
        } else if !image_stacks.is_empty() {
            println!(
                "No image stack selected. Defaulting to first available stack {}.",
                image_stacks[0]
            );

            Some(image_stacks[0].clone())
        } else {
            None
        };

        let slider_value = if let Some(slider_value) = params.get("test_slider") {
            slider_value.as_f64().unwrap_or(50.0)
        } else {
            50.0
        };

        let href = if let Some(ref stack_id) = selected_stack {
            format!("http://{host_name}/api/z-scan/thumbnail/{stack_id}/0/150")
        } else {
            format!("")
        };

        ProcedureUi {
            name: "focus_stacking".to_string(),
            display_name: "Focus Stacking".to_string(),
            description: "Generates an image with extended depth of field by combining multiple images taken at different focus distances.".to_string(),
            columns: 4,
            elements: HashMap::from([
                ("stack_preview".to_string(), Element::Image {
                    display_name: "Stack Preview".to_string(),
                    href,
                    positioning: ElementPositioning {
                        row: 1,
                        column: 1,
                        row_span: 1,
                        column_span: 1,
                    },
                }),
                ("output_preview".to_string(), Element::Image {
                    display_name: "Output Preview".to_string(),
                    href: format!("http://{host_name}/api/z-scan/thumbnail/e4a5f501-0865-4b0b-9840-98744fce5d4e/0/150"),
                    positioning: ElementPositioning {
                        row: 1,
                        column: 2,
                        row_span: 1,
                        column_span: 1,
                    },
                }),
                ("image_stack".to_string(), Element::Select {
                    display_name: "Image stack".to_string(),
                    options: image_stacks,
                    value: selected_stack.unwrap_or_default(),
                    positioning: ElementPositioning {
                        row: 1,
                        column: 3,
                        row_span: 1,
                        column_span: 2,
                    },
                }),
                ("test_slider".to_string(), Element::Slider {
                    display_name: "Test Slider".to_string(),
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                    value: slider_value,
                    positioning: ElementPositioning {
                        row: 2,
                        column: 1,
                        row_span: 1,
                        column_span: 4,
                    },
                }),
            ]),
        }
    }
}
