mod task;

use std::collections::HashMap;

use anyhow::{Result, bail};
use common::ws::value::Value;
use reqwest;
use tracing::{error, warn};

use common::rest::z_scan::ZScanMetadata;
use common::ws::compute_node::{TaskUiDescription, Widget, WidgetPosition};
use tokio::sync::watch;

use crate::gpu::GpuImageProcessor;
use crate::tasks::Task;

const THUMBNAIL_SIZE: u32 = 250;

#[derive(Clone)]
pub struct FocusStacking {
    host_name: String,

    progress_rx: watch::Receiver<Option<f32>>,
    progress_tx: watch::Sender<Option<f32>>,
}

impl FocusStacking {
    pub fn new(host_name: String) -> Self {
        let (progress_tx, progress_rx) = watch::channel(None);
        FocusStacking {
            host_name,
            progress_rx,
            progress_tx,
        }
    }

    pub async fn get_progress(&self) -> Option<f32> {
        *self.progress_rx.borrow()
    }

    pub async fn set_progress(&self, progress: Option<f32>) {
        let _ = self.progress_tx.send(progress);
    }

    async fn fetch_image_stacks(host_name: &str) -> Result<Vec<String>> {
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

    async fn list_image_stacks(host_name: &str) -> Vec<String> {
        let image_stacks = Self::fetch_image_stacks(host_name).await;

        if image_stacks.is_err() {
            error!(
                "Failed to fetch image stacks: {}",
                image_stacks.err().unwrap()
            );
            vec![]
        } else {
            image_stacks.unwrap()
        }
    }

    async fn get_selected_image_stack(
        params: &HashMap<String, Value>,
        image_stacks: &Vec<String>,
    ) -> Option<String> {
        if let Some(Value::String(selected_stack)) = params.get("image_stack") {
            if !image_stacks.contains(&selected_stack) {
                error!(
                    "Selected image stack {} not found. Defaulting to first available stack.",
                    selected_stack
                );
            }

            Some(selected_stack.clone())
        } else if !image_stacks.is_empty() {
            warn!(
                "No image stack selected. Defaulting to first available stack {}.",
                image_stacks[0]
            );

            Some(image_stacks[0].clone())
        } else {
            None
        }
    }
}

#[async_trait::async_trait]
impl Task for FocusStacking {
    async fn describe(
        &self,
        task_name: String,
        params: HashMap<String, Value>,
    ) -> TaskUiDescription {
        let host_name = &self.host_name;

        let image_stacks = Self::list_image_stacks(host_name).await;
        let selected_stack = Self::get_selected_image_stack(&params, &image_stacks).await;

        let href = if let Some(ref stack_id) = selected_stack {
            format!("http://{host_name}/api/z-scan/thumbnail/{stack_id}/0/{THUMBNAIL_SIZE}")
        } else {
            format!("")
        };

        TaskUiDescription {
            name: task_name,
            display_name: "Focus Stacking".to_string(),
            description: "Generates an image with extended depth of field by combining multiple images taken at different focus distances.".to_string(),
            columns: 2,
            progress: self.get_progress().await,
            locked: false,
            elements: HashMap::from([
                ("stack_preview".to_string(), Widget::Image {
                    display_name: "".to_string(),
                    href,
                    positioning: WidgetPosition {
                        row: 1,
                        column: 1,
                        row_span: 1,
                        column_span: 1,
                    },
                }),
                ("image_stack".to_string(), Widget::Select {
                    display_name: "".to_string(),
                    options: image_stacks,
                    value: selected_stack.unwrap_or_default(),
                    positioning: WidgetPosition {
                        row: 1,
                        column: 2,
                        row_span: 1,
                        column_span: 1,
                    },
                }),
            ]),
        }
    }

    async fn execute(
        &self,
        gpu_image_processor: &GpuImageProcessor,
        params: HashMap<String, Value>,
    ) {
        let stack_id = if let Some(Value::String(stack_id)) = params.get("image_stack") {
            stack_id.clone()
        } else {
            error!("No image stack selected for focus stacking.");
            return;
        };

        if let Err(e) = self.run_task(gpu_image_processor, &stack_id).await {
            error!("Error executing focus stacking task: {}", e);
        }
    }

    fn get_progress_receiver(&self) -> watch::Receiver<Option<f32>> {
        self.progress_rx.clone()
    }
}
