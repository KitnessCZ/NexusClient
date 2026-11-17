
use bytemuck::{Pod, Zeroable};

use super::Camera;

/// Data layout expected by block.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct CameraUniform {
    view_projection: [[f32; 4]; 4],
}

/// GPU uniform buffer and bind group for the active camera.
pub struct GpuCamera {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl GpuCamera {
    /// Creates the camera uniform buffer and its bind group.
    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        camera: &Camera,
        aspect_ratio: f32,
    ) -> Self {
        let uniform = CameraUniform {
            view_projection: view_projection_matrix(camera, aspect_ratio),
        };

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("NexusClient Camera Uniform Buffer"),
            size: std::mem::size_of::<CameraUniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("NexusClient Camera Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self { buffer, bind_group }
    }

    /// Updates the camera matrix for the current frame.
    pub fn update(
        &self,
        queue: &wgpu::Queue,
        camera: &Camera,
        aspect_ratio: f32,
    ) {
        let uniform = CameraUniform {
            view_projection: view_projection_matrix(camera, aspect_ratio),
        };

        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&uniform));
    }

    /// Returns the bind group required by BlockRenderPipeline.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

fn view_projection_matrix(
    camera: &Camera,
    aspect_ratio: f32,
) -> [[f32; 4]; 4] {
    let view = view_matrix(
        camera.position(),
        camera.forward(),
        camera.up(),
    );

    let projection = projection_matrix(
        camera.fov_y_radians(),
        aspect_ratio,
        camera.near_plane(),
        camera.far_plane(),
    );

    multiply_matrices(projection, view)
}

fn view_matrix(
    position: [f32; 3],
    forward: [f32; 3],
    up: [f32; 3],
) -> [[f32; 4]; 4] {
    let forward = normalize(forward);
    let right = normalize(cross(up, forward));
    let up = normalize(cross(forward, right));

    let right_dot = dot(right, position);
    let up_dot = dot(up, position);
    let forward_dot = dot(forward, position);

    [
        [right[0], up[0], forward[0], 0.0],
        [right[1], up[1], forward[1], 0.0],
        [right[2], up[2], forward[2], 0.0],
        [-right_dot, -up_dot, -forward_dot, 1.0],
    ]
}

fn projection_matrix(
    fov_y: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
) -> [[f32; 4]; 4] {
    let aspect = if aspect_ratio.is_finite() && aspect_ratio > 0.0 {
        aspect_ratio
    } else {
        1.0
    };

    let safe_near = near.max(0.0001);
    let safe_far = far.max(safe_near + 0.0001);
    let f = 1.0 / (fov_y * 0.5).tan();

    let z_scale = safe_far / (safe_far - safe_near);
    let z_translation =
        -(safe_near * safe_far) / (safe_far - safe_near);

    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, z_scale, 1.0],
        [0.0, 0.0, z_translation, 0.0],
    ]
}

fn multiply_matrices(
    a: [[f32; 4]; 4],
    b: [[f32; 4]; 4],
) -> [[f32; 4]; 4] {
    let mut result = [[0.0; 4]; 4];

    for column in 0..4 {
        for row in 0..4 {
            for index in 0..4 {
                result[column][row] +=
                    a[index][row] * b[column][index];
            }
        }
    }

    result
}

fn normalize(vector: [f32; 3]) -> [f32; 3] {
    let length_squared = dot(vector, vector);

    if !length_squared.is_finite() || length_squared <= 0.000001 {
        return [0.0, 0.0, 1.0];
    }

    let inverse_length = length_squared.sqrt().recip();

    [
        vector[0] * inverse_length,
        vector[1] * inverse_length,
        vector[2] * inverse_length,
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::{multiply_matrices, projection_matrix, view_matrix};

    #[test]
    fn identity_multiplication_preserves_matrix() {
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        let result = multiply_matrices(identity, identity);

        assert_eq!(result, identity);
    }

    #[test]
    fn camera_at_origin_has_identity_view_matrix() {
        let view = view_matrix(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0],
        );

        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        assert_eq!(view, identity);
    }

    #[test]
    fn projection_uses_valid_depth_range() {
        let projection = projection_matrix(
            70.0_f32.to_radians(),
            16.0 / 9.0,
            0.05,
            512.0,
        );

        assert!(projection[0][0] > 0.0);
        assert!(projection[1][1] > 0.0);
        assert!(projection[2][2] > 0.0);
        assert_eq!(projection[2][3], 1.0);
    }
}
