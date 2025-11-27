use std::collections::HashMap;

use anyhow::{Result, bail};
use reqwest;

use common::rest::z_scan::ZScanMetadata;
use common::ws::compute_node::{Element, ElementPositioning, ProcedureUi};

pub struct FocusStacking {}

impl FocusStacking {
    pub fn _new() -> Self {
        FocusStacking {}
    }

    async fn list_image_stacks(host_name: &str) -> Result<Vec<String>> {
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

    pub async fn describe(host_name: &str) -> ProcedureUi {
        let image_stacks = Self::list_image_stacks(host_name).await;

        let image_stacks = if image_stacks.is_err() {
            println!(
                "Failed to fetch image stacks: {}",
                image_stacks.err().unwrap()
            );
            vec![]
        } else {
            image_stacks.unwrap()
        };

        ProcedureUi {
            display_name: "Focus Stacking".to_string(),
            description: "Generates an image with extended depth of field by combining multiple images taken at different focus distances.".to_string(),
            columns: 4,
            elements: HashMap::from([
                ("stack_preview".to_string(), Element::Image {
                    display_name: "Stack Preview".to_string(),
                    href: format!("http://{host_name}/api/z-scan/thumbnail/e4a5f501-0865-4b0b-9840-98744fce5d4e/0/150"),
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
                    options: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                    value: "a".to_string(),
                    positioning: ElementPositioning {
                        row: 1,
                        column: 3,
                        row_span: 1,
                        column_span: 2,
                    },
                }),
            ]),
        }
    }
}
