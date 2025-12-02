pub mod focus_stacking;

use std::collections::HashMap;

use async_trait::async_trait;

use common::ws::{compute_node::TaskUiDescription, value::Value};
use tokio::sync::watch;

use crate::gpu::GpuImageProcessor;

#[async_trait]
pub trait Task {
    async fn describe(&self, task_name: String, params: HashMap<String, Value>) -> TaskUiDescription;
    async fn execute(&self, gpu_image_processor: &GpuImageProcessor, params: HashMap<String, Value>);

    fn get_progress_receiver(&self) -> watch::Receiver<Option<f32>>;
}
