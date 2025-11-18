use std::collections::HashMap;

use anyhow::{Result, bail};
use reqwest;

use interface::rest::z_scan::ZScanMetadata;
use interface::ws::compute_node::{Input, Output, Procedure};

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

    pub async fn describe(host_name: &str) -> Procedure {
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

        Procedure {
            display_name: "Focus Stacking".to_string(),
            description: "Generates an image with extended depth of field by combining multiple images taken at different focus distances.".to_string(),
            inputs: HashMap::from([
                ("image_stack".to_string(), Input::Selection {
                    display_name: "Image stack".to_string(),
                    options: image_stacks,
                    value: "a".to_string(),
                }),
                ("preview".to_string(), Input::ImagePreview {
                    display_name: "Preview".to_string(),
                    href: format!("http://{host_name}/api/z-scan/thumbnail/e4a5f501-0865-4b0b-9840-98744fce5d4e/0/150"),
                }),
            ]),
            outputs: HashMap::from([
                ("image".to_string(), Output::Image {
                    display_name: "Stacked Image".to_string(),
                }),
            ]),
        }
    }
}
