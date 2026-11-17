
use super::{BlockId, World};

/// Generates a flat test world made from several chunks.
///
/// radius_chunks = 1 creates a 3x3 chunk area.
/// surface_y is the Y coordinate of the grass layer.
pub fn generate_flat_world(
    world: &mut World,
    radius_chunks: i32,
    surface_y: i32,
) {
    if radius_chunks < 0 {
        return;
    }

    for chunk_z in -radius_chunks..=radius_chunks {
        for chunk_x in -radius_chunks..=radius_chunks {
            let start_x = chunk_x * 16;
            let start_z = chunk_z * 16;

            for local_z in 0..16 {
                for local_x in 0..16 {
                    let x = start_x + local_x;
                    let z = start_z + local_z;

                    world.set_block(
                        x, surface_y, z,
                        BlockId::GRASS_BLOCK,
                    );

                    for depth in 1..=3 {
                        world.set_block(
                            x,
                            surface_y - depth,
                            z,
                            BlockId::DIRT,
                        );
                    }

                    for depth in 4..=8 {
                        world.set_block(
                            x,
                            surface_y - depth,
                            z,
                            BlockId::STONE,
                        );
                    }

                    world.set_block(
                        x,
                        surface_y - 9,
                        z,
                        BlockId::BEDROCK,
                    );
                }
            }
        }
    }
}

/// Generates a 3x3-chunk test world with its surface at Y = 0.
pub fn generate_test_world(world: &mut World) {
    generate_flat_world(world, 1, 0);
}
