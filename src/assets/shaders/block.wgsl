
struct CameraUniform {
    view_projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) block_id: f32,
    @location(4) face: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) block_id: f32,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position =
        camera.view_projection * vec4<f32>(input.position, 1.0);

    output.normal = input.normal;
    output.uv = input.uv;
    output.block_id = input.block_id;

    return output;
}

fn get_block_color(block_id: f32) -> vec3<f32> {
    let id = u32(block_id);

    switch id {
        case 1u: {
            return vec3<f32>(0.55, 0.55, 0.55);
        }
        case 2u: {
            return vec3<f32>(0.45, 0.27, 0.12);
        }
        case 3u: {
            return vec3<f32>(0.30, 0.65, 0.20);
        }
        case 4u: {
            return vec3<f32>(0.20, 0.20, 0.20);
        }
        default: {
            return vec3<f32>(1.0, 0.0, 1.0);
        }
    }
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let light_direction = normalize(vec3<f32>(-0.4, 0.8, 0.3));
    let diffuse = max(dot(normalize(input.normal), light_direction), 0.0);

    let ambient = 0.35;
    let lighting = ambient + diffuse * 0.65;

    let color = get_block_color(input.block_id) * lighting;

    return vec4<f32>(color, 1.0);
}
