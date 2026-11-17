
use super::{BlockId, ChunkPos, World};

/// One vertex in a generated chunk mesh.
///
/// The data is renderer-independent and can be uploaded to either
/// a Vulkan or OpenGL ES GPU backend through wgpu.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshVertex {
    /// Position relative to the chunk origin.
    pub position: [f32; 3],

    /// Surface normal used for lighting.
    pub normal: [f32; 3],

    /// Block identifier used by the texture/material system.
    pub block_id: u16,

    /// Texture coordinates within a block texture.
    pub uv: [f32; 2],

    /// Face direction:
    /// 0 = +Z, 1 = -Z, 2 = +X,
    /// 3 = -X, 4 = +Y, 5 = -Y.
    pub face: u8,
}

/// CPU-side geometry for one chunk.
#[derive(Default, Debug)]
pub struct ChunkMesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
}

impl ChunkMesh {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn face_count(&self) -> usize {
        self.vertices.len() / 4
    }
}

/// Block face geometry.
///
/// Face order:
/// 0 = Front (+Z)
/// 1 = Back (-Z)
/// 2 = Right (+X)
/// 3 = Left (-X)
/// 4 = Top (+Y)
/// 5 = Bottom (-Y)
const FACES: [([i32; 3], [f32; 3], [[f32; 3]; 4]); 6] = [
    (
        [0, 0, 1],
        [0.0, 0.0, 1.0],
        [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ],
    ),
    (
        [0, 0, -1],
        [0.0, 0.0, -1.0],
        [
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ],
    ),
    (
        [1, 0, 0],
        [1.0, 0.0, 0.0],
        [
            [1.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
        ],
    ),
    (
        [-1, 0, 0],
        [-1.0, 0.0, 0.0],
        [
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
        ],
    ),
    (
        [0, 1, 0],
        [0.0, 1.0, 0.0],
        [
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
    ),
    (
        [0, -1, 0],
        [0.0, -1.0, 0.0],
        [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
    ),
];

/// UV coordinates for one complete block face.
///
/// The texture system can later transform these coordinates into
/// positions inside a Minecraft texture atlas.
const FACE_UVS: [[f32; 2]; 4] = [
    [0.0, 0.0],
    [1.0, 0.0],
    [1.0, 1.0],
    [0.0, 1.0],
];

/// Builds visible block faces for a chunk.
///
/// `min_y` is inclusive and `max_y` is exclusive.
///
/// Vertex positions are relative to the chunk's origin.
/// Faces touching another non-air block are omitted.
pub fn build_chunk_mesh(
    world: &World,
    chunk_pos: ChunkPos,
    min_y: i32,
    max_y: i32,
) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();

    if min_y >= max_y {
        return mesh;
    }

    let origin_x = chunk_pos.x * 16;
    let origin_z = chunk_pos.z * 16;

    for y in min_y..max_y {
        for z in 0..16 {
            for x in 0..16 {
                let world_x = origin_x + x;
                let world_z = origin_z + z;

                let block = world.get_block(world_x, y, world_z);

                if block.is_air() {
                    continue;
                }

                for (face_index, (offset, normal, corners))
                    in FACES.into_iter().enumerate()
                {
                    let neighbor = world.get_block(
                        world_x + offset[0],
                        y + offset[1],
                        world_z + offset[2],
                    );

                    if !neighbor.is_air() {
                        continue;
                    }

                    append_face(
                        &mut mesh,
                        x as f32,
                        y as f32,
                        z as f32,
                        block,
                        normal,
                        corners,
                        face_index as u8,
                    );
                }
            }
        }
    }

    mesh
}

fn append_face(
    mesh: &mut ChunkMesh,
    x: f32,
    y: f32,
    z: f32,
    block: BlockId,
    normal: [f32; 3],
    corners: [[f32; 3]; 4],
    face: u8,
) {
    let base = mesh.vertices.len() as u32;

    for index in 0..4 {
        let corner = corners[index];

        mesh.vertices.push(MeshVertex {
            position: [
                x + corner[0],
                y + corner[1],
                z + corner[2],
            ],
            normal,
            block_id: block.0,
            uv: FACE_UVS[index],
            face,
        });
    }

    mesh.indices.extend_from_slice(&[
        base,
        base + 1,
        base + 2,
        base,
        base + 2,
        base + 3,
    ]);
}

#[cfg(test)]
mod tests {
    use super::{build_chunk_mesh, ChunkPos};
    use crate::engine::{BlockId, World};

    #[test]
    fn one_block_generates_six_faces() {
        let mut world = World::new();
        world.set_block(0, 0, 0, BlockId::STONE);

        let mesh = build_chunk_mesh(
            &world,
            ChunkPos::new(0, 0),
            0,
            1,
        );

        assert_eq!(mesh.face_count(), 6);
        assert_eq!(mesh.vertex_count(), 24);
        assert_eq!(mesh.index_count(), 36);
        assert_eq!(mesh.triangle_count(), 12);
    }

    #[test]
    fn adjacent_blocks_hide_internal_faces() {
        let mut world = World::new();

        world.set_block(0, 0, 0, BlockId::STONE);
        world.set_block(1, 0, 0, BlockId::DIRT);

        let mesh = build_chunk_mesh(
            &world,
            ChunkPos::new(0, 0),
            0,
            1,
        );

        assert_eq!(mesh.face_count(), 10);
        assert_eq!(mesh.vertex_count(), 40);
        assert_eq!(mesh.triangle_count(), 20);
    }

    #[test]
    fn mesh_vertices_contain_texture_coordinates() {
        let mut world = World::new();
        world.set_block(0, 0, 0, BlockId::GRASS_BLOCK);

        let mesh = build_chunk_mesh(
            &world,
            ChunkPos::new(0, 0),
            0,
            1,
        );

        assert!(!mesh.vertices.is_empty());

        let expected_uvs = [
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ];

        for face in mesh.vertices.chunks_exact(4) {
            for index in 0..4 {
                assert_eq!(face[index].uv, expected_uvs[index]);
            }
        }
    }

    #[test]
    fn mesh_vertices_contain_face_identifiers() {
        let mut world = World::new();
        world.set_block(0, 0, 0, BlockId::STONE);

        let mesh = build_chunk_mesh(
            &world,
            ChunkPos::new(0, 0),
            0,
            1,
        );

        let mut faces = [false; 6];

        for vertex in &mesh.vertices {
            assert!(vertex.face < 6);
            faces[vertex.face as usize] = true;
        }

        assert!(faces.into_iter().all(|visible| visible));
    }

    #[test]
    fn empty_world_generates_empty_mesh() {
        let world = World::new();

        let mesh = build_chunk_mesh(
            &world,
            ChunkPos::new(0, 0),
            0,
            16,
        );

        assert!(mesh.is_empty());
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.index_count(), 0);
    }

    #[test]
    fn invalid_height_range_generates_empty_mesh() {
        let mut world = World::new();
        world.set_block(0, 0, 0, BlockId::STONE);

        let mesh = build_chunk_mesh(
            &world,
            ChunkPos::new(0, 0),
            10,
            5,
        );

        assert!(mesh.is_empty());
    }

    #[test]
    fn chunk_mesh_uses_chunk_relative_positions() {
        let mut world = World::new();
        world.set_block(16, 0, 0, BlockId::STONE);

        let mesh = build_chunk_mesh(
            &world,
            ChunkPos::new(1, 0),
            0,
            1,
        );

        assert!(!mesh.vertices.is_empty());

        for vertex in &mesh.vertices {
            assert!(vertex.position[0] >= 0.0);
            assert!(vertex.position[0] <= 17.0);
        }
    }
}
