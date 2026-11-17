
use super::{BlockId, ChunkPos, World};

/// One vertex in a generated chunk mesh.
#[derive(Clone, Copy, Debug)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub block_id: u16,
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

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

const FACES: [([i32; 3], [f32; 3], [[f32; 3]; 4]); 6] = [
    // Front (+Z)
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
    // Back (-Z)
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
    // Right (+X)
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
    // Left (-X)
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
    // Top (+Y)
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
    // Bottom (-Y)
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

/// Builds visible block faces for a chunk.
///
/// `min_y` is inclusive and `max_y` is exclusive.
/// Vertex positions are relative to the chunk's origin.
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

                for (offset, normal, corners) in FACES {
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
) {
    let base = mesh.vertices.len() as u32;

    for corner in corners {
        mesh.vertices.push(MeshVertex {
            position: [
                x + corner[0],
                y + corner[1],
                z + corner[2],
            ],
            normal,
            block_id: block.0,
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
