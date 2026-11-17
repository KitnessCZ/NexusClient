
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
                        x,
                        surface_y,
                        z,
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

#[cfg(test)]
mod tests {
    use super::{generate_flat_world, generate_test_world};
    use super::super::{build_chunk_mesh, BlockId, ChunkPos, World};

    #[test]
    fn generates_nine_chunks() {
        let mut world = World::new();

        generate_test_world(&mut world);

        assert_eq!(world.chunk_count(), 9);
    }

    #[test]
    fn generates_correct_block_layers() {
        let mut world = World::new();

        generate_test_world(&mut world);

        assert_eq!(world.get_block(0, 0, 0), BlockId::GRASS_BLOCK);
        assert_eq!(world.get_block(0, -1, 0), BlockId::DIRT);
        assert_eq!(world.get_block(0, -3, 0), BlockId::DIRT);
        assert_eq!(world.get_block(0, -4, 0), BlockId::STONE);
        assert_eq!(world.get_block(0, -8, 0), BlockId::STONE);
        assert_eq!(world.get_block(0, -9, 0), BlockId::BEDROCK);
        assert_eq!(world.get_block(0, 1, 0), BlockId::AIR);
    }

    #[test]
    fn handles_negative_chunk_coordinates() {
        let mut world = World::new();

        generate_test_world(&mut world);

        assert_eq!(world.get_block(-1, 0, -1), BlockId::GRASS_BLOCK);
        assert_eq!(world.get_block(-16, 0, -16), BlockId::GRASS_BLOCK);
        assert_eq!(world.get_block(-17, 0, -17), BlockId::GRASS_BLOCK);
    }

    #[test]
    fn generates_empty_world_for_negative_radius() {
        let mut world = World::new();

        generate_flat_world(&mut world, -1, 0);

        assert_eq!(world.chunk_count(), 0);
    }

    #[test]
    fn mesh_hides_faces_between_neighboring_chunks() {
        let mut world = World::new();

        generate_test_world(&mut world);

        let mesh = build_chunk_mesh(
            &world,
            ChunkPos::new(0, 0),
            -9,
            1,
        );

        // The central chunk has only its top and bottom surfaces visible.
        // Its side faces are hidden by the surrounding chunks.
        assert_eq!(mesh.vertex_count(), 2048);
        assert_eq!(mesh.triangle_count(), 1024);
    }
}
