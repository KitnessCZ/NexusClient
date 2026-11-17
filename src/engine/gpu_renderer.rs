
use super::GpuVertex;

/// The wgpu render pipeline used to draw block meshes.
///
/// The host application supplies the GPU device and surface color format.
/// The camera uniform bind group is exposed so the renderer can provide
/// the view-projection matrix required by block.wgsl.
pub struct BlockRenderPipeline {
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl BlockRenderPipeline {
    /// Creates the block rendering pipeline.
    ///
    /// `color_format` must match the format of the render target.
    pub fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("NexusClient Block Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../assets/shaders/block.wgsl").into(),
            ),
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("NexusClient Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bind_group_layouts = [Some(&camera_bind_group_layout)];

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("NexusClient Block Pipeline Layout"),
                bind_group_layouts: &bind_group_layouts,
                immediate_size: 0,
            });

        let vertex_layout = GpuVertex::vertex_buffer_layout();
        let vertex_buffers = [Some(vertex_layout)];

        let color_targets = [Some(wgpu::ColorTargetState {
            format: color_format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("NexusClient Block Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex_main"),
                    compilation_options: Default::default(),
                    buffers: &vertex_buffers,
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fragment_main"),
                    compilation_options: Default::default(),
                    targets: &color_targets,
                }),
                multiview_mask: None,
                cache: None,
            });

        Self {
            camera_bind_group_layout,
            render_pipeline,
        }
    }
}
