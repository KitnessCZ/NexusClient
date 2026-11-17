use std::collections::HashMap;

use super::BlockId;

pub const CHUNK_SIZE: usize = 16;
pub const SECTION_VOLUME: usize =
    CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

/// Position of a chunk in world coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

/// A 16x16x16 block section.
#[derive(Clone)]
pub struct ChunkSection {
    blocks: [BlockId; SECTION_VOLUME],
}

impl Default for ChunkSection {
    fn default() -> Self {
        Self {
            blocks: [BlockId::AIR; SECTION_VOLUME],
        }
    }
}

impl ChunkSection {
    pub fn get_block(
        &self,
        x: usize,
        y: usize,
        z: usize,
    ) -> Option<BlockId> {
        let index = Self::index(x, y, z)?;
        Some(self.blocks[index])
    }

    pub fn set_block(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        block: BlockId,
    ) -> bool {
        let Some(index) = Self::index(x, y, z) else {
            return false;
        };

        self.blocks[index] = block;
        true
    }

    fn index(x: usize, y: usize, z: usize) -> Option<usize> {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return None;
        }

        Some(
            y * CHUNK_SIZE * CHUNK_SIZE
                + z * CHUNK_SIZE
                + x,
        )
    }
}

/// A chunk contains vertical sections.
///
/// Sections are allocated only when needed, saving memory for empty areas.
pub struct Chunk {
    pub position: ChunkPos,
    sections: HashMap<i32, ChunkSection>,
}

impl Chunk {
    pub fn new(position: ChunkPos) -> Self {
        Self {
            position,
            sections: HashMap::new(),
        }
    }

    pub fn get_block(
        &self,
        local_x: usize,
        world_y: i32,
        local_z: usize,
    ) -> BlockId {
        if local_x >= CHUNK_SIZE || local_z >= CHUNK_SIZE {
            return BlockId::AIR;
        }

        let section_y = world_y.div_euclid(CHUNK_SIZE as i32);
        let local_y = world_y.rem_euclid(CHUNK_SIZE as i32) as usize;

        self.sections
            .get(&section_y)
            .and_then(|section| {
                section.get_block(local_x, local_y, local_z)
            })
            .unwrap_or(BlockId::AIR)
    }

    pub fn set_block(
        &mut self,
        local_x: usize,
        world_y: i32,
        local_z: usize,
        block: BlockId,
    ) -> bool {
        if local_x >= CHUNK_SIZE || local_z >= CHUNK_SIZE {
            return false;
        }

        let section_y = world_y.div_euclid(CHUNK_SIZE as i32);
        let local_y = world_y.rem_euclid(CHUNK_SIZE as i32) as usize;

        if block.is_air() {
            if let Some(section) = self.sections.get_mut(&section_y) {
                section.set_block(local_x, local_y, local_z, block);

                if section.blocks.iter().all(|b| b.is_air()) {
                    self.sections.remove(&section_y);
                }
            }

            return true;
        }

        self.sections
            .entry(section_y)
            .or_default()
            .set_block(local_x, local_y, local_z, block)
    }

    pub fn section_count(&self) -> usize {
        self.sections.len()
    }
}