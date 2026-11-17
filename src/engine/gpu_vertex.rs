
use super::MeshVertex;

/// GPU-friendly vertex data.
///
/// Uses only 32-bit floating-point values so the renderer can
/// describe the vertex layout consistently across graphics backends.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpuVertex {
    /// Position relative to the chunk origin.
    pub position: [f32; 3],

    /// Surface normal.
    pub normal: [f32; 3],

    /// Texture coordinates.
    pub uv: [f32; 2],

    /// Block identifier converted to a float for GPU attributes.
    pub block_id: f32,

    /// Face direction:
    /// 0 = +Z, 1 = -Z, 2 = +X,
    /// 3 = -X, 4 = +Y, 5 = -Y.
    pub face: f32,
}

impl GpuVertex {
    /// Converts one engine mesh vertex into GPU vertex data.
    pub fn from_mesh_vertex(vertex: &MeshVertex) -> Self {
        Self {
            position: vertex.position,
            normal: vertex.normal,
            uv: vertex.uv,
            block_id: vertex.block_id as f32,
            face: vertex.face as f32,
        }
    }
}

/// Converts a complete engine mesh vertex list into GPU vertex data.
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
        let converted = convert_vertices(&[]);

        assert!(converted.is_empty());
    }
}
