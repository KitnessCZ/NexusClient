
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UiVertex {
    position: [f32; 2],
    color: [f32; 4],
}

pub struct NexusUi {
    pipeline: wgpu::RenderPipeline,
    screen: usize,
    notice: Option<String>,
}

impl NexusUi {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("NexusClient UI Shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
"#
                .into(),
            ),
        });

        let attributes = wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x4
        ];

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("NexusClient UI Pipeline Layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("NexusClient UI Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<UiVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attributes,
                }],
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            screen: 0,
            notice: None,
        }
    }

    pub fn tap(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let (nav_y, nav_height) = if width >= height {
            (height * 0.5 - 24.0, 48.0)
        } else {
            (height - 76.0, 52.0)
        };

        if width >= height {
            let labels = 5;
            let nav_width = (width - 48.0) / labels as f32;

            if y >= nav_y && y <= nav_y + nav_height {
                let index = ((x - 24.0) / nav_width).floor() as usize;
                if index < labels {
                    self.screen = index;
                    self.notice = None;
                    return;
                }
            }
        } else if y >= nav_y && y <= nav_y + nav_height {
            let nav_width = width / 5.0;
            let index = (x / nav_width).floor() as usize;
            if index < 5 {
                self.screen = index;
                self.notice = None;
                return;
            }
        }

        if self.screen == 0 {
            let button_y = if width >= height {
                height * 0.5 + 115.0
            } else {
                height * 0.5 + 110.0
            };

            if y >= button_y && y <= button_y + 54.0 {
                self.notice = Some("GAME RUNTIME NOT READY YET".to_owned());
            }
        }
    }

    pub fn draw(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) {
        let w = width.max(1) as f32;
        let h = height.max(1) as f32;
        let mut vertices = Vec::new();

        let bg = [0.035, 0.043, 0.065, 1.0];
        let panel = [0.075, 0.085, 0.12, 1.0];
        let panel2 = [0.105, 0.12, 0.16, 1.0];
        let accent = [0.25, 0.55, 1.0, 1.0];
        let white = [0.92, 0.94, 0.98, 1.0];
        let muted = [0.53, 0.58, 0.68, 1.0];

        rect(&mut vertices, 0.0, 0.0, w, h, bg, w, h);

        let compact = w < h;
        let margin = if compact { 18.0 } else { 30.0 };

        // Header
        rect(
            &mut vertices,
            0.0,
            0.0,
            w,
            if compact { 82.0 } else { 76.0 },
            [0.055, 0.065, 0.095, 1.0],
            w,
            h,
        );

        rect(
            &mut vertices,
            margin,
            18.0,
            40.0,
            40.0,
            accent,
            w,
            h,
        );

        text(
            &mut vertices,
            "N",
            margin + 13.0,
            25.0,
            4.0,
            white,
            w,
            h,
        );

        text(
            &mut vertices,
            "NEXUSCLIENT",
            margin + 54.0,
            20.0,
            3.0,
            white,
            w,
            h,
        );

        text(
            &mut vertices,
            "ANDROID / JAVA",
            margin + 55.0,
            43.0,
            1.5,
            muted,
            w,
            h,
        );

        // Navigation
        let labels = ["HOME", "VERSIONS", "MODS", "PACKS", "SETTINGS"];
        let nav_y = if compact { h - 76.0 } else { 96.0 };
        let nav_height = 48.0;
        let nav_width = (w - margin * 2.0) / labels.len() as f32;

        for (index, label) in labels.iter().enumerate() {
            let x = margin + index as f32 * nav_width;
            let selected = self.screen == index;

            if selected {
                rect(
                    &mut vertices,
                    x + 2.0,
                    nav_y,
                    nav_width - 4.0,
                    nav_height,
                    [0.10, 0.18, 0.31, 1.0],
                    w,
                    h,
                );
                rect(
                    &mut vertices,
                    x + 2.0,
                    nav_y,
                    3.0,
                    nav_height,
                    accent,
                    w,
                    h,
                );
            }

            let label_width = label.len() as f32 * 6.0 * 1.5;
            text(
                &mut vertices,
                label,
                x + (nav_width - label_width) * 0.5,
                nav_y + 18.0,
                1.5,
                if selected { white } else { muted },
                w,
                h,
            );
        }

        if self.screen == 0 {
            let content_top = if compact { 112.0 } else { 174.0 };
            let card_y = content_top + 74.0;
            let card_x = margin;
            let card_width = w - margin * 2.0;
            let card_height = if compact {
                (h - card_y - 112.0).max(240.0)
            } else {
                (h - card_y - 50.0).max(260.0)
            };

            text(
                &mut vertices,
                "WELCOME BACK",
                card_x,
                content_top,
                2.0,
                accent,
                w,
                h,
            );

            text(
                &mut vertices,
                "YOUR GAME STARTS HERE",
                card_x,
                content_top + 24.0,
                3.0,
                white,
                w,
                h,
            );

            rect(
                &mut vertices,
                card_x,
                card_y,
                card_width,
                card_height,
                panel,
                w,
                h,
            );

            rect(
                &mut vertices,
                card_x,
                card_y,
                4.0,
                card_height,
                accent,
                w,
                h,
            );

            text(
                &mut vertices,
                "READY WHEN YOU ARE",
                card_x + 22.0,
                card_y + 24.0,
                2.0,
                accent,
                w,
                h,
            );

            text(
                &mut vertices,
                "NEXUSCLIENT",
                card_x + 22.0,
                card_y + 58.0,
                4.0,
                white,
                w,
                h,
            );

            text(
                &mut vertices,
                "A MINECRAFT JAVA CLIENT BUILT IN RUST",
                card_x + 22.0,
                card_y + 94.0,
                1.5,
                muted,
                w,
                h,
            );

            let info_y = card_y + 122.0;
            let info_width = (card_width - 44.0).min(460.0);

            rect(
                &mut vertices,
                card_x + 22.0,
                info_y,
                info_width,
                66.0,
                panel2,
                w,
                h,
            );

            text(
                &mut vertices,
                "CLIENT STATUS",
                card_x + 36.0,
                info_y + 12.0,
                1.5,
                accent,
                w,
                h,
            );

            text(
                &mut vertices,
                "FOUNDATION BUILD",
                card_x + 36.0,
                info_y + 34.0,
                2.0,
                white,
                w,
                h,
            );

            let button_y = card_y + card_height - 70.0;
            let button_width = (card_width - 44.0).min(240.0);

            rect(
                &mut vertices,
                card_x + 22.0,
                button_y,
                button_width,
                48.0,
                accent,
                w,
                h,
            );

            text(
                &mut vertices,
                "PLAY",
                card_x + 22.0 + button_width * 0.5 - 12.0,
                button_y + 17.0,
                2.5,
                white,
                w,
                h,
            );

            if let Some(notice) = &self.notice {
                text(
                    &mut vertices,
                    notice,
                    card_x + 22.0,
                    button_y + 60.0,
                    1.5,
                    accent,
                    w,
                    h,
                );
            } else {
                text(
                    &mut vertices,
                    "GAME ENGINE IN DEVELOPMENT",
                    card_x + 22.0,
                    button_y + 60.0,
                    1.5,
                    muted,
                    w,
                    h,
                );
            }
        } else {
            let titles = ["", "VERSIONS", "MODS", "RESOURCE PACKS", "SETTINGS"];
            let title = titles[self.screen.min(4)];

            text(
                &mut vertices,
                title,
                margin,
                if compact { 120.0 } else { 180.0 },
                4.0,
                white,
                w,
                h,
            );

            rect(
                &mut vertices,
                margin,
                if compact { 170.0 } else { 230.0 },
                w - margin * 2.0,
                150.0,
                panel,
                w,
                h,
            );

            text(
                &mut vertices,
                "COMING IN A FUTURE MILESTONE",
                margin + 22.0,
                if compact { 200.0 } else { 260.0 },
                2.0,
                accent,
                w,
                h,
            );
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NexusClient UI Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("NexusClient UI Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }
}

fn rect(
    vertices: &mut Vec<UiVertex>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: [f32; 4],
    screen_width: f32,
    screen_height: f32,
) {
    let x1 = x / screen_width * 2.0 - 1.0;
    let x2 = (x + width) / screen_width * 2.0 - 1.0;
    let y1 = 1.0 - y / screen_height * 2.0;
    let y2 = 1.0 - (y + height) / screen_height * 2.0;

    let a = UiVertex { position: [x1, y1], color };
    let b = UiVertex { position: [x2, y1], color };
    let c = UiVertex { position: [x2, y2], color };
    let d = UiVertex { position: [x1, y2], color };

    vertices.extend_from_slice(&[a, b, c, a, c, d]);
}

fn text(
    vertices: &mut Vec<UiVertex>,
    value: &str,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    width: f32,
    height: f32,
) {
    let mut cursor = x;

    for character in value.chars() {
        let glyph = glyph(character);
        for (row, bits) in glyph.iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    rect(
                        vertices,
                        cursor + column as f32 * scale,
                        y + row as f32 * scale,
                        scale,
                        scale,
                        color,
                        width,
                        height,
                    );
                }
            }
        }
        cursor += 6.0 * scale;
    }
}

fn glyph(character: char) -> [u8; 7] {
    match character.to_ascii_uppercase() {
        'A' => [14,17,17,31,17,17,17],
        'B' => [30,17,17,30,17,17,30],
        'C' => [14,17,16,16,16,17,14],
        'D' => [30,17,17,17,17,17,30],
        'E' => [31,16,16,30,16,16,31],
        'F' => [31,16,16,30,16,16,16],
        'G' => [14,17,16,23,17,17,15],
        'H' => [17,17,17,31,17,17,17],
        'I' => [14,4,4,4,4,4,14],
        'J' => [7,2,2,2,18,18,12],
        'K' => [17,18,20,24,20,18,17],
        'L' => [16,16,16,16,16,16,31],
        'M' => [17,27,21,21,17,17,17],
        'N' => [17,25,21,19,17,17,17],
        'O' => [14,17,17,17,17,17,14],
        'P' => [30,17,17,30,16,16,16],
        'Q' => [14,17,17,17,21,18,13],
        'R' => [30,17,17,30,20,18,17],
        'S' => [15,16,16,14,1,1,30],
        'T' => [31,4,4,4,4,4,4],
        'U' => [17,17,17,17,17,17,14],
        'V' => [17,17,17,17,17,10,4],
        'W' => [17,17,17,21,21,21,10],
        'X' => [17,17,10,4,10,17,17],
        'Y' => [17,17,10,4,4,4,4],
        'Z' => [31,1,2,4,8,16,31],
        '0' => [14,17,19,21,25,17,14],
        '1' => [4,12,4,4,4,4,14],
        '2' => [14,17,1,2,4,8,31],
        '3' => [30,1,1,14,1,1,30],
        '4' => [2,6,10,18,31,2,2],
        '5' => [31,16,16,30,1,1,30],
        '6' => [14,16,16,30,17,17,14],
        '7' => [31,1,2,4,8,8,8],
        '8' => [14,17,17,14,17,17,14],
        '9' => [14,17,17,15,1,1,14],
        '/' => [1,2,2,4,8,8,16],
        '.' => [0,0,0,0,0,12,12],
        ':' => [0,12,12,0,12,12,0],
        '-' => [0,0,0,31,0,0,0],
        _ => [0; 7],
    }
}
