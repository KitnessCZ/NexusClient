
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UiVertex {
    position: [f32; 2],
    color: [f32; 4],
}

#[derive(Clone, Copy)]
struct UiLayout {
    width: f32,
    height: f32,
    sidebar_width: f32,
    margin: f32,
    card_x: f32,
    card_y: f32,
    card_width: f32,
    card_height: f32,
    play_x: f32,
    play_y: f32,
    play_width: f32,
    play_height: f32,
    nav_y: f32,
    nav_item_height: f32,
}

impl UiLayout {
    fn new(width: f32, height: f32) -> Self {
        let width = width.max(1.0);
        let height = height.max(1.0);

        let sidebar_width = (width * 0.20).clamp(136.0, 220.0);
        let margin = 16.0;

        let card_x = sidebar_width + margin;
        let card_y = 52.0;
        let card_width = (width - card_x - margin).max(1.0);
        let card_height = (height - 126.0).max(150.0);

        let play_x = card_x;
        let play_y = height - 64.0;
        let play_width = (card_width * 0.55).clamp(160.0, 260.0);
        let play_height = 38.0;

        Self {
            width,
            height,
            sidebar_width,
            margin,
            card_x,
            card_y,
            card_width,
            card_height,
            play_x,
            play_y,
            play_width,
            play_height,
            nav_y: 100.0,
            nav_item_height: 38.0,
        }
    }

    fn sidebar_item_y(&self, index: usize) -> f32 {
        self.nav_y + index as f32 * self.nav_item_height
    }
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
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<UiVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attributes,
                })],
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
        let layout = UiLayout::new(width, height);

        // Sidebar Play button.
        if x >= 10.0
            && x <= layout.sidebar_width - 10.0
            && y >= 56.0
            && y <= 96.0
        {
            self.screen = 0;
            self.notice = Some("GAME RUNTIME NOT READY YET".to_owned());
            return;
        }

        // Sidebar navigation.
        for index in 0..7 {
            let item_y = layout.sidebar_item_y(index);

            if x >= 10.0
                && x <= layout.sidebar_width - 10.0
                && y >= item_y
                && y <= item_y + 32.0
            {
                self.screen = index;
                self.notice = None;
                return;
            }
        }

        // Main Play button.
        if self.screen == 0
            && x >= layout.play_x
            && x <= layout.play_x + layout.play_width
            && y >= layout.play_y
            && y <= layout.play_y + layout.play_height
        {
            self.notice = Some("GAME RUNTIME NOT READY YET".to_owned());
            return;
        }

        // Top-right settings control.
        if x >= layout.width - 58.0
            && y >= 8.0
            && y <= 48.0
        {
            self.screen = 5;
            self.notice = None;
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
        let layout = UiLayout::new(width as f32, height as f32);
        let w = layout.width;
        let h = layout.height;

        let mut vertices = Vec::new();

        let background = [0.055, 0.045, 0.075, 1.0];
        let sidebar = [0.045, 0.035, 0.060, 1.0];
        let panel = [0.105, 0.085, 0.135, 1.0];
        let panel_light = [0.145, 0.120, 0.175, 1.0];
        let red = [0.90, 0.16, 0.20, 1.0];
        let red_dark = [0.48, 0.075, 0.12, 1.0];
        let white = [0.96, 0.94, 0.98, 1.0];
        let muted = [0.62, 0.57, 0.68, 1.0];
        let border = [0.19, 0.15, 0.22, 1.0];

        // Background.
        rect(
            &mut vertices,
            0.0,
            0.0,
            w,
            h,
            background,
            w,
            h,
        );

        // Left sidebar.
        rect(
            &mut vertices,
            0.0,
            0.0,
            layout.sidebar_width,
            h,
            sidebar,
            w,
            h,
        );

        // Sidebar divider.
        rect(
            &mut vertices,
            layout.sidebar_width - 1.0,
            48.0,
            1.0,
            h - 64.0,
            border,
            w,
            h,
        );

        // Brand icon.
        rect(
            &mut vertices,
            16.0,
            12.0,
            32.0,
            32.0,
            red,
            w,
            h,
        );

        text(
            &mut vertices,
            "N",
            24.0,
            18.0,
            3.0,
            white,
            w,
            h,
        );

        text(
            &mut vertices,
            "NEXUSCLIENT",
            56.0,
            13.0,
            1.8,
            white,
            w,
            h,
        );

        text(
            &mut vertices,
            "ANDROID",
            57.0,
            29.0,
            1.1,
            muted,
            w,
            h,
        );

        // Sidebar Play button.
        let play_selected = self.screen == 0;

        rect(
            &mut vertices,
            10.0,
            56.0,
            layout.sidebar_width - 20.0,
            40.0,
            if play_selected { red_dark } else { panel },
            w,
            h,
        );

        rect(
            &mut vertices,
            22.0,
            69.0,
            10.0,
            14.0,
            white,
            w,
            h,
        );

        text(
            &mut vertices,
            "PLAY",
            42.0,
            70.0,
            1.8,
            white,
            w,
            h,
        );

        // Sidebar navigation.
        let labels = [
            "MODS",
            "RESOURCE PACK",
            "SHADERS",
            "INSTALLED",
            "SETTINGS",
            "SKINS",
        ];

        let symbols = ["M", "R", "S", "I", "T", "K"];

        for (index, label) in labels.iter().enumerate() {
            let item_y = layout.sidebar_item_y(index + 1);
            let selected = self.screen == index + 1;

            if selected {
                rect(
                    &mut vertices,
                    10.0,
                    item_y,
                    layout.sidebar_width - 20.0,
                    32.0,
                    panel,
                    w,
                    h,
                );

                rect(
                    &mut vertices,
                    10.0,
                    item_y,
                    3.0,
                    32.0,
                    red,
                    w,
                    h,
                );
            }

            text(
                &mut vertices,
                symbols[index],
                22.0,
                item_y + 10.0,
                1.7,
                if selected { red } else { muted },
                w,
                h,
            );

            text(
                &mut vertices,
                label,
                42.0,
                item_y + 11.0,
                1.35,
                if selected { white } else { muted },
                w,
                h,
            );
        }

        // Top-right controls.
        rect(
            &mut vertices,
            w - 104.0,
            8.0,
            38.0,
            36.0,
            panel,
            w,
            h,
        );

        text(
            &mut vertices,
            "T",
            w - 91.0,
            19.0,
            2.0,
            red,
            w,
            h,
        );

        rect(
            &mut vertices,
            w - 60.0,
            8.0,
            52.0,
            36.0,
            panel,
            w,
            h,
        );

        text(
            &mut vertices,
            "USER",
            w - 53.0,
            20.0,
            1.2,
            white,
            w,
            h,
        );

        // Main content card.
        rect(
            &mut vertices,
            layout.card_x,
            layout.card_y,
            layout.card_width,
            layout.card_height,
            panel,
            w,
            h,
        );

        // Card border.
        rect(
            &mut vertices,
            layout.card_x,
            layout.card_y,
            layout.card_width,
            1.0,
            border,
            w,
            h,
        );

        if self.screen == 0 {
            self.draw_home(
                &mut vertices,
                &layout,
                w,
                h,
                white,
                muted,
                red,
                panel_light,
            );
        } else {
            self.draw_section(
                &mut vertices,
                &layout,
                w,
                h,
                white,
                muted,
                red,
                panel_light,
            );
        }

        // Main bottom Play button.
        if self.screen == 0 {
            rect(
                &mut vertices,
                layout.play_x,
                layout.play_y,
                layout.play_width,
                layout.play_height,
                red,
                w,
                h,
            );

            text(
                &mut vertices,
                "PLAY MINECRAFT",
                layout.play_x + 18.0,
                layout.play_y + 13.0,
                1.7,
                white,
                w,
                h,
            );

            rect(
                &mut vertices,
                layout.play_x + layout.play_width + 10.0,
                layout.play_y,
                104.0,
                layout.play_height,
                panel_light,
                w,
                h,
            );

            text(
                &mut vertices,
                "PROFILE 1",
                layout.play_x + layout.play_width + 20.0,
                layout.play_y + 13.0,
                1.3,
                white,
                w,
                h,
            );

            if let Some(notice) = &self.notice {
                text(
                    &mut vertices,
                    notice,
                    layout.card_x,
                    h - 17.0,
                    1.1,
                    red,
                    w,
                    h,
                );
            }
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

    fn draw_home(
        &self,
        vertices: &mut Vec<UiVertex>,
        layout: &UiLayout,
        w: f32,
        h: f32,
        white: [f32; 4],
        muted: [f32; 4],
        red: [f32; 4],
        panel_light: [f32; 4],
    ) {
        let x = layout.card_x + 18.0;
        let y = layout.card_y + 18.0;

        text(
            vertices,
            "READY TO PLAY",
            x,
            y,
            3.0,
            white,
            w,
            h,
        );

        text(
            vertices,
            "MINECRAFT JAVA CLIENT",
            x,
            y + 30.0,
            1.7,
            muted,
            w,
            h,
        );

        text(
            vertices,
            "NEXUSCLIENT ENGINE",
            x,
            y + 68.0,
            1.8,
            white,
            w,
            h,
        );

        text(
            vertices,
            "RUST RENDERER / WGPU",
            x,
            y + 88.0,
            1.4,
            muted,
            w,
            h,
        );

        let notice_y = y + 116.0;
        let notice_height = (layout.card_height - 140.0).max(38.0);

        rect(
            vertices,
            x,
            notice_y,
            layout.card_width - 36.0,
            notice_height.min(64.0),
            panel_light,
            w,
            h,
        );

        rect(
            vertices,
            x,
            notice_y,
            3.0,
            notice_height.min(64.0),
            red,
            w,
            h,
        );

        text(
            vertices,
            "CLIENT STATUS",
            x + 14.0,
            notice_y + 12.0,
            1.4,
            red,
            w,
            h,
        );

        text(
            vertices,
            "FOUNDATION BUILD",
            x + 14.0,
            notice_y + 34.0,
            1.7,
            white,
            w,
            h,
        );
    }

    fn draw_section(
        &self,
        vertices: &mut Vec<UiVertex>,
        layout: &UiLayout,
        w: f32,
        h: f32,
        white: [f32; 4],
        muted: [f32; 4],
        red: [f32; 4],
        panel_light: [f32; 4],
    ) {
        let titles = [
            "PLAY",
            "MODS",
            "RESOURCE PACKS",
            "SHADERS",
            "INSTALLED",
            "SETTINGS",
            "SKINS",
        ];

        let title = titles[self.screen.min(6)];

        let x = layout.card_x + 18.0;
        let y = layout.card_y + 22.0;

        text(
            vertices,
            title,
            x,
            y,
            3.0,
            white,
            w,
            h,
        );

        rect(
            vertices,
            x,
            y + 54.0,
            layout.card_width - 36.0,
            100.0,
            panel_light,
            w,
            h,
        );

        rect(
            vertices,
            x,
            y + 54.0,
            3.0,
            100.0,
            red,
            w,
            h,
        );

        text(
            vertices,
            "COMING SOON",
            x + 16.0,
            y + 72.0,
            2.0,
            red,
            w,
            h,
        );

        text(
            vertices,
            "THIS SECTION IS UNDER DEVELOPMENT",
            x + 16.0,
            y + 102.0,
            1.4,
            muted,
            w,
            h,
        );
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
    if width <= 0.0 || height <= 0.0 {
        return;
    }

    let x1 = x / screen_width * 2.0 - 1.0;
    let x2 = (x + width) / screen_width * 2.0 - 1.0;
    let y1 = 1.0 - y / screen_height * 2.0;
    let y2 = 1.0 - (y + height) / screen_height * 2.0;

    let a = UiVertex {
        position: [x1, y1],
        color,
    };
    let b = UiVertex {
        position: [x2, y1],
        color,
    };
    let c = UiVertex {
        position: [x2, y2],
        color,
    };
    let d = UiVertex {
        position: [x1, y2],
        color,
    };

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
        let bitmap = glyph(character);

        for (row, bits) in bitmap.iter().enumerate() {
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
