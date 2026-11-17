
use super::MeshVertex;
use std::mem::size_of;

/// GPU-friendly vertex data.
///
/// Field order and types must match the WGSL vertex shader.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpuVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub block_id: f32,
    pub face: f32,
}

impl GpuVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 5] =
        wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32x2,
            3 => Float32,
            4 => Float32
        ];

    /// Converts an engine mesh vertex into GPU vertex data.
    pub fn from_mesh_vertex(vertex: &MeshVertex) -> Self {
        Self {
            position: vertex.position,
            normal: vertex.normal,
            uv: vertex.uv,
            block_id: vertex.block_id as f32,
            face: vertex.face as f32,
        }
    }

    /// Returns the vertex layout expected by the block shader.
    pub fn vertex_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

/// Converts a complete mesh vertex list into GPU vertex data.
pub fn convert_vertices(vertices: &[MeshVertex]) -> Vec<GpuVertex> {
    vertices
        .iter()
        .map(GpuVertex::from_mesh_vertex)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{convert_vertices, GpuVertex};
    use crate::engine::MeshVertex;

    #[test]
    fn converts_mesh_vertex_correctly() {
        let vertex = MeshVertex {
            position: [1.0, 2.0, 3.0],
            normal: [0.0, 1.0, 0.0],
            block_id: 4,
            uv: [0.5, 0.75],
            face: 2,
        };

        let gpu_vertex = GpuVertex::from_mesh_vertex(&vertex);

        assert_eq!(gpu_vertex.position, [1.0, 2.0, 3.0]);
        assert_eq!(gpu_vertex.normal, [0.0, 1.0, 0.0]);
        assert_eq!(gpu_vertex.uv, [0.5, 0.75]);
        assert_eq!(gpu_vertex.block_id, 4.0);
        assert_eq!(gpu_vertex.face, 2.0);
    }

    #[test]
    fn converts_multiple_vertices() {
        let vertices = vec![
            MeshVertex {
                position: [0.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                block_id: 1,
                uv: [0.0, 0.0],
                face: 4,
            },
            MeshVertex {
                position: [1.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                block_id: 2,
                uv: [1.0, 0.0],
                face: 4,
            },
        ];

        let converted = convert_vertices(&vertices);

        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0].block_id, 1.0);
        assert_eq!(converted[1].block_id, 2.0);
    }

    #[test]
    fn converts_empty_vertex_list() {
        assert!(convert_vertices(&[]).is_empty());
    }

    #[test]
    fn vertex_layout_matches_shader_attributes() {
        let layout = GpuVertex::vertex_buffer_layout();

        assert_eq!(layout.array_stride, 40);
        assert_eq!(layout.attributes.len(), 5);
        assert_eq!(layout.step_mode, wgpu::VertexStepMode::Vertex);
    }
}
