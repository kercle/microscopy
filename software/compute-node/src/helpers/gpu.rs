use std::collections::HashMap;

use anyhow::Result;

pub struct GpuImageProcessor {
    device: wgpu::Device,
    _queue: wgpu::Queue,
    _bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,

    pipelines: HashMap<&'static str, wgpu::ComputePipeline>,
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
            _queue: queue,
            _bind_group_layout: bind_group_layout,
            pipeline_layout,
            pipelines: HashMap::new(),
        };

        ret.load_pipeline("sobel").await?;

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
            _ => return Err(anyhow::anyhow!("Unknown pipeline: {}", pipeline_name)),
        };

        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Sobel Shader"),
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
}
