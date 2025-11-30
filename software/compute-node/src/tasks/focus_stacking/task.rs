use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use common::rest::z_scan::ZScanMetadata;
use reqwest::Client;
use tempfile::{TempDir, tempdir};

use crate::tasks::focus_stacking::FocusStacking;
use crate::helpers::progress::ProgressIter;

struct TaskContext {
    temp_dir: TempDir,

    raw_dir_path: PathBuf,

    input_images: Vec<PathBuf>,
}

impl TaskContext {
    pub fn new() -> Result<Self> {
        let temp_dir = tempdir()?;
        let raw_dir_path = temp_dir.path().join("raw");
        let sobel_dir_path = temp_dir.path().join("sobel");

        fs::create_dir_all(&raw_dir_path)?;
        fs::create_dir_all(&sobel_dir_path)?;

        Ok(TaskContext {
            temp_dir,
            raw_dir_path,
            input_images: Vec::new(),
        })
    }

    pub fn write_raw_image(&mut self, index: usize, data: &[u8]) -> Result<()> {
        let file_path = self.raw_dir_path.join(format!("{:05}.png", index));
        fs::write(&file_path, data)?;
        self.input_images.push(file_path);
        Ok(())
    }

    pub fn make_persistent(self, target_dir: &Path) -> Result<()> {
        crate::helpers::fs::sync_folders(&self.temp_dir.path(), target_dir)?;
        Ok(())
    }
}

impl FocusStacking {
    async fn describe_stack(&self, stack_id: &str) -> Result<ZScanMetadata> {
        let url = format!("http://{}/api/z-scan/list", self.host_name);

        let client = Client::new();
        let resp = client.get(&url).send().await?;
        let stacks: Vec<ZScanMetadata> = resp.json().await?;

        for stack in stacks {
            if stack.uuid == stack_id {
                return Ok(stack);
            }
        }

        bail!("Stack with ID {} not found", stack_id);
    }

    pub async fn run_task(&self, stack_id: &str) -> Result<()> {
        self.set_progress(Some(0.0)).await;
        println!("Executing focus stacking on stack: {}", stack_id);

        let stack_metadata = self.describe_stack(stack_id).await?;

        let mut task_ctx = TaskContext::new()?;

        let client = Client::new();
        for i in ProgressIter::new(0..stack_metadata.frame_count, self.progress_tx.clone()) {
            let url = format!(
                "http://{}/api/z-scan/frame/{}/{}",
                self.host_name, stack_id, i
            );

            let resp = client.get(&url).send().await?;

            if !resp.status().is_success() {
                bail!(
                    "Failed to fetch frame {i} of stack {stack_id}: {}",
                    resp.status()
                );
            }

            task_ctx.write_raw_image(i as usize, &resp.bytes().await?)?;
        }

        // just for testing purposes
        task_ctx.make_persistent(Path::new("/tmp/compute_node_test_output"))?;

        Ok(())
    }
}
