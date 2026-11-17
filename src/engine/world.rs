use std::collections::HashMap;

use super::{BlockId, Chunk, ChunkPos};

/// The engine's block world.
///
/// Coordinates are world-space block coordinates. Negative coordinates
/// are handled correctly using Euclidean division.
pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> BlockId {
        let chunk_x = x.div_euclid(16);
        let chunk_z = z.div_euclid(16);

        let local_x = x.rem_euclid(16) as usize;
        let local_z = z.rem_euclid(16) as usize;

        let position = ChunkPos::new(chunk_x, chunk_z);

        self.chunks
            .get(&position)
            .map(|chunk| chunk.get_block(local_x, y, local_z))
            .unwrap_or(BlockId::AIR)
    }

    pub fn set_block(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        block: BlockId,
    ) {
        let chunk_x = x.div_euclid(16);
        let chunk_z = z.div_euclid(16);

        let local_x = x.rem_euclid(16) as usize;
        let local_z = z.rem_euclid(16) as usize;

        let position = ChunkPos::new(chunk_x, chunk_z);

        if block.is_air() {
            if let Some(chunk) = self.chunks.get_mut(&position) {
                chunk.set_block(local_x, y, local_z, block);
            }

            return;
        }

        self.chunks
            .entry(position)
            .or_insert_with(|| Chunk::new(position))
            .set_block(local_x, y, local_z, block);
    }

    pub fn get_chunk(&self, position: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&position)
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}