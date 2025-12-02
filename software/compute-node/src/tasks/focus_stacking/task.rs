use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use common::rest::z_scan::ZScanMetadata;
use image::RgbaImage;
use reqwest::Client;
use tempfile::{TempDir, tempdir};
use tokio::fs;
use tokio::task::spawn_blocking;
use tracing::info;

use crate::helpers::gpu::{GpuFilter, GpuImageProcessor};
use crate::helpers::progress::ProgressIter;
use crate::tasks::focus_stacking::FocusStacking;

struct TaskContext {
    temp_dir: TempDir,

    raw_dir_path: PathBuf,

    input_images: Vec<PathBuf>,
}

impl TaskContext {
    pub async fn new() -> Result<Self> {
        let temp_dir = tempdir()?;
        let raw_dir_path = temp_dir.path().join("raw");
        let sobel_dir_path = temp_dir.path().join("sobel");

        fs::create_dir_all(&raw_dir_path).await?;
        fs::create_dir_all(&sobel_dir_path).await?;

        Ok(TaskContext {
            temp_dir,
            raw_dir_path,
            input_images: Vec::new(),
        })
    }

    pub async fn write_raw_image(&mut self, index: usize, data: &[u8]) -> Result<()> {
        let file_path = self.raw_dir_path.join(format!("{:05}.png", index));
        fs::write(&file_path, data).await?;
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

    pub async fn run_task(&self, gpu_processor: &GpuImageProcessor, stack_id: &str) -> Result<()> {
        self.set_progress(Some(0.0)).await;
        info!("Executing focus stacking on stack: {}", stack_id);

        let stack_metadata = self.describe_stack(stack_id).await?;

        let mut task_ctx = TaskContext::new().await?;

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

            let data = resp.bytes().await?;
            task_ctx.write_raw_image(i as usize, &data).await?;

            let img: RgbaImage = spawn_blocking(move || -> Result<RgbaImage> {
                Ok(turbojpeg::decompress_image(&data)?)
            })
            .await??;

            let sobel_image = gpu_processor
                .apply_filters(
                    &img,
                    GpuFilter::Sobel + (GpuFilter::BoxHBlur + GpuFilter::BoxVBlur) * 10,
                )
                .await?;

            let temp_dir = task_ctx.temp_dir.path().to_path_buf();
            spawn_blocking(move || -> Result<()> {
                let jpeg_data =
                    turbojpeg::compress_image(&sobel_image, 95, turbojpeg::Subsamp::Sub2x2)?;

                std::fs::write(
                    temp_dir.join("sobel").join(format!("{:05}.jpg", i)),
                    &jpeg_data,
                )?;

                Ok(())
            });
        }

        // just for testing purposes
        task_ctx.make_persistent(Path::new("/tmp/compute_node_test_output"))?;

        Ok(())
    }
}
