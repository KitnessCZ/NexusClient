
use super::MeshVertex;
use std::mem::size_of;

/// GPU-friendly vertex data.
///
/// The layout must match the vertex inputs in block.wgsl.
/// Pod and Zeroable allow safe conversion to GPU upload bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
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

    /// Returns this vertex as bytes for GPU buffer uploads.
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

/// Converts a complete mesh vertex list into GPU vertex data.
pub fn convert_vertices(vertices: &[MeshVertex]) -> Vec<GpuVertex> {
    vertices
        .iter()
        .map(GpuVertex::from_mesh_vertex)
        .collect()
}

/// Converts a vertex slice into bytes suitable for GPU buffer uploads.
pub fn vertices_as_bytes(vertices: &[GpuVertex]) -> &[u8] {
    bytemuck::cast_slice(vertices)
}

#[cfg(test)]
mod tests {
    use super::{convert_vertices, vertices_as_bytes, GpuVertex};
    use crate::engine::MeshVertex;

    fn test_vertex() -> MeshVertex {
        MeshVertex {
            position: [1.0, 2.0, 3.0],
            normal: [0.0, 1.0, 0.0],
            block_id: 4,
            uv: [0.5, 0.75],
            face: 2,
        }
    }

    #[test]
    fn converts_mesh_vertex_correctly() {
        let gpu_vertex = GpuVertex::from_mesh_vertex(&test_vertex());

        assert_eq!(gpu_vertex.position, [1.0, 2.0, 3.0]);
        assert_eq!(gpu_vertex.normal, [0.0, 1.0, 0.0]);
        assert_eq!(gpu_vertex.uv, [0.5, 0.75]);
        assert_eq!(gpu_vertex.block_id, 4.0);
        assert_eq!(gpu_vertex.face, 2.0);
    }

    #[test]
    fn converts_multiple_vertices() {
        let vertices = vec![
            test_vertex(),
            MeshVertex {
                position: [4.0, 5.0, 6.0],
                normal: [0.0, 0.0, 1.0],
                block_id: 2,
                uv: [1.0, 0.0],
                face: 4,
            },
        ];

        let converted = convert_vertices(&vertices);

        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0].block_id, 4.0);
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

    #[test]
    fn vertex_bytes_have_expected_size() {
        let vertex = GpuVertex::from_mesh_vertex(&test_vertex());

        assert_eq!(vertex.as_bytes().len(), 40);
    }

    #[test]
    fn vertex_slice_can_be_converted_to_bytes() {
        let vertices = vec![
            GpuVertex::from_mesh_vertex(&test_vertex()),
            GpuVertex::from_mesh_vertex(&test_vertex()),
        ];

        let bytes = vertices_as_bytes(&vertices);

        assert_eq!(bytes.len(), 80);
    }

    #[test]
    fn empty_vertex_slice_has_empty_bytes() {
        assert!(vertices_as_bytes(&[]).is_empty());
    }
}
