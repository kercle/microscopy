use std::collections::HashMap;

use anyhow::Result;
use image::RgbaImage;
use wgpu::util::DeviceExt;

pub struct GpuImageProcessor {
    device: wgpu::Device,
    queue: wgpu::Queue,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,

    pipelines: HashMap<&'static str, wgpu::ComputePipeline>,
}

pub enum GpuFilter {
    Sobel,
    GaussianHBlur,
    GaussianVBlur,
}

impl std::fmt::Display for GpuFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuFilter::Sobel => write!(f, "sobel"),
            GpuFilter::GaussianHBlur => write!(f, "gaussian_hblur"),
            GpuFilter::GaussianVBlur => write!(f, "gaussian_vblur"),
        }
    }
}

impl GpuImageProcessor {
    pub async fn new() -> Result<Self> {
        let adapter = wgpu::Instance::default()
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await?;

        let (device, queue) = adapter.request_device(&Default::default()).await?;

        let bind_group_layout = Self::create_bind_group_layout(&device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let mut ret = Self {
            device,
            queue,
            bind_group_layout,
            pipeline_layout,
            pipelines: HashMap::new(),
        };

        ret.load_pipeline("sobel").await?;
        ret.load_pipeline("gaussian_hblur").await?;
        ret.load_pipeline("gaussian_vblur").await?;

        Ok(ret)
    }

    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let entries = &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ];
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries,
            label: None,
        })
    }

    async fn load_pipeline(&mut self, pipeline_name: &'static str) -> Result<()> {
        let shader_source = match pipeline_name {
            "sobel" => include_str!("../shaders/sobel.wgsl"),
            "gaussian_hblur" => include_str!("../shaders/gaussian_hblur.wgsl"),
            "gaussian_vblur" => include_str!("../shaders/gaussian_vblur.wgsl"),
            _ => return Err(anyhow::anyhow!("Unknown pipeline: {}", pipeline_name)),
        };

        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(pipeline_name),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });

        let pipeline = self
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(pipeline_name),
                layout: Some(&self.pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        self.pipelines.insert(pipeline_name, pipeline);
        Ok(())
    }

    async fn init_textures(&self, img: &RgbaImage) -> Result<(wgpu::Texture, wgpu::Texture)> {
        let (width, height) = img.dimensions();
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let src_texture = self.device.create_texture_with_data(
            &self.queue,
            &wgpu::TextureDescriptor {
                label: Some("src"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &img,
        );

        let dst_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("dst"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        Ok((src_texture, dst_texture))
    }

    async fn bind_shader(
        &self,
        shader: &str,
        texture_size: wgpu::Extent3d,
        src_texture: &wgpu::Texture,
        dst_texture: &wgpu::Texture,
    ) -> Result<()> {
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &src_texture.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &dst_texture.create_view(&Default::default()),
                    ),
                },
            ],
            label: None,
        });

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut cpass = encoder.begin_compute_pass(&Default::default());
            cpass.set_pipeline(&self.pipelines[shader]);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(
                (texture_size.width + 15) / 16,
                (texture_size.height + 15) / 16,
                1,
            );
        }
        self.queue.submit(Some(encoder.finish()));
        self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })?;

        Ok(())
    }

    async fn read_from_texture(
        &self,
        dst_texture: &wgpu::Texture,
        texture_size: wgpu::Extent3d,
    ) -> Result<Vec<u8>> {
        let bytes_per_pixel = 4;
        let unpadded_bytes_per_row = texture_size.width * bytes_per_pixel;
        let padded_bytes_per_row = ((unpadded_bytes_per_row + 255) / 256) * 256;

        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("out"),
            size: (padded_bytes_per_row * texture_size.height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            dst_texture.as_image_copy(), // still works; now returns TexelCopyTextureInfo<'_>
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row as u32),
                    rows_per_image: Some(texture_size.height as u32),
                },
            },
            texture_size,
        );
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })?;

        let data = buffer_slice.get_mapped_range();

        let mut pixels = Vec::with_capacity(
            (texture_size.width * texture_size.height * bytes_per_pixel) as usize,
        );
        for chunk in data.chunks(padded_bytes_per_row as usize) {
            pixels.extend_from_slice(&chunk[..(unpadded_bytes_per_row as usize)]);
        }

        Ok(pixels)
    }

    pub async fn apply_filters(&self, img: &RgbaImage, filters: &[GpuFilter]) -> Result<RgbaImage> {
        let (mut a_texture, mut b_texture) = self.init_textures(img).await?;

        let texture_size = wgpu::Extent3d {
            width: img.width(),
            height: img.height(),
            depth_or_array_layers: 1,
        };

        for filter in filters.iter() {
            self.bind_shader(&filter.to_string(), texture_size, &a_texture, &b_texture)
                .await?;

            std::mem::swap(&mut a_texture, &mut b_texture);
        }

        let pixels = self.read_from_texture(&a_texture, texture_size).await?;

        let image_data =
            RgbaImage::from_raw(img.width(), img.height(), pixels).ok_or_else(|| {
                anyhow::anyhow!("Failed to create image from raw data after Sobel filter")
            })?;

        Ok(image_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[tokio::test]
    async fn test_gpu_image_processor_creation() {
        let processor = GpuImageProcessor::new().await;
        assert!(processor.is_ok());
    }

    #[tokio::test]
    async fn test_sobel() {
        let test_image = include_bytes!("../../tests/test_image.jpg");
        let img: RgbaImage = turbojpeg::decompress_image(test_image).unwrap();

        let processor = GpuImageProcessor::new().await.unwrap();

        // benchmark:
        let start = std::time::Instant::now();
        for _ in 1..=999 {
            let _ = processor.apply_sobel(&img).await.unwrap();
        }
        let image = processor.apply_sobel(&img).await.unwrap();
        let duration = start.elapsed();
        println!("Sobel filter took: {:?}", duration / 1000);

        let jpeg_data = turbojpeg::compress_image(&image, 95, turbojpeg::Subsamp::Sub2x2).unwrap();
        std::fs::write("tests/outputs/output.jpg", &jpeg_data).unwrap();
    }
}
