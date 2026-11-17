
use super::{
    generate_test_world, raycast, BlockHit, BlockId, Camera, PhysicsInput,
    Player, PlayerPhysics, World,
};

const REACH_DISTANCE: f32 = 6.0;
const PLAYER_HALF_WIDTH: f32 = 0.3;
const PLAYER_HEIGHT: f32 = 1.8;

#[derive(Clone, Copy, Debug, Default)]
pub struct EngineInput {
    pub forward: f32,
    pub right: f32,
    pub look_yaw: f32,
    pub look_pitch: f32,
    pub jump: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct EngineFrame {
    pub camera: Camera,
    pub delta_seconds: f32,
    pub elapsed_seconds: f64,
    pub frame_number: u64,
    pub on_ground: bool,
}

pub struct Engine {
    world: World,
    player: Player,
    physics: PlayerPhysics,
    elapsed_seconds: f64,
    frame_number: u64,
}

impl Engine {
    pub fn new() -> Self {
        let mut world = World::new();
        generate_test_world(&mut world);

        Self::with_world(world, Player::default())
    }

    pub fn with_world(world: World, player: Player) -> Self {
        Self {
            world,
            player,
            physics: PlayerPhysics::new(),
            elapsed_seconds: 0.0,
            frame_number: 0,
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }

    pub fn physics(&self) -> &PlayerPhysics {
        &self.physics
    }

    pub fn elapsed_seconds(&self) -> f64 {
        self.elapsed_seconds
    }

    pub fn frame_number(&self) -> u64 {
        self.frame_number
    }

    /// Returns the block currently targeted by the player's camera.
    pub fn targeted_block(&self) -> Option<BlockHit> {
        let camera = self.player.camera();

        raycast(
            &self.world,
            camera.position(),
            camera.forward(),
            REACH_DISTANCE,
        )
    }

    /// Removes the block currently targeted by the player.
    ///
    /// Returns the removed block information, or None if no block is hit.
    pub fn break_targeted_block(&mut self) -> Option<BlockHit> {
        let hit = self.targeted_block()?;

        self.world.set_block(
            hit.block_position[0],
            hit.block_position[1],
            hit.block_position[2],
            BlockId::AIR,
        );

        Some(hit)
    }

    /// Places a block against the face currently targeted by the player.
    ///
    /// Returns the placed block position on success.
    pub fn place_targeted_block(
        &mut self,
        block: BlockId,
    ) -> Option<[i32; 3]> {
        if block.is_air() {
            return None;
        }

        let hit = self.targeted_block()?;
        let position = hit.previous_position;

        if !self.world.get_block(
            position[0],
            position[1],
            position[2],
        ).is_air() {
            return None;
        }

        if block_intersects_player(position, self.player.position()) {
            return None;
        }

        self.world.set_block(
            position[0],
            position[1],
            position[2],
            block,
        );

        Some(position)
    }

    /// Processes input, physics, and timing for one frame.
    pub fn update(
        &mut self,
        input: EngineInput,
        delta_seconds: f32,
    ) -> EngineFrame {
        let delta = if delta_seconds.is_finite() && delta_seconds > 0.0 {
            delta_seconds.min(0.1)
        } else {
            0.0
        };

        if delta > 0.0 {
            self.player.look(input.look_yaw, input.look_pitch);

            self.physics.step(
                &self.world,
                &mut self.player,
                PhysicsInput {
                    forward: input.forward,
                    right: input.right,
                    jump: input.jump,
                },
                delta,
            );

            self.elapsed_seconds += delta as f64;
            self.frame_number = self.frame_number.saturating_add(1);
        }

        EngineFrame {
            camera: self.player.camera(),
            delta_seconds: delta,
            elapsed_seconds: self.elapsed_seconds,
            frame_number: self.frame_number,
            on_ground: self.physics.is_on_ground(),
        }
    }

    pub fn reset_player(&mut self, position: [f32; 3]) {
        self.player = Player::new(position);
        self.physics.reset();
        self.elapsed_seconds = 0.0;
        self.frame_number = 0;
    }
}

fn block_intersects_player(
    block: [i32; 3],
    player: [f32; 3],
) -> bool {
    let block_min = [
        block[0] as f32,
        block[1] as f32,
        block[2] as f32,
    ];

    let block_max = [
        block_min[0] + 1.0,
        block_min[1] + 1.0,
        block_min[2] + 1.0,
    ];

    let player_min = [
        player[0] - PLAYER_HALF_WIDTH,
        player[1],
        player[2] - PLAYER_HALF_WIDTH,
    ];

    let player_max = [
        player[0] + PLAYER_HALF_WIDTH,
        player[1] + PLAYER_HEIGHT,
        player[2] + PLAYER_HALF_WIDTH,
    ];

    for axis in 0..3 {
        if block_max[axis] <= player_min[axis]
            || block_min[axis] >= player_max[axis]
        {
            return false;
        }
    }

    true
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Engine, EngineInput};
    use super::super::{BlockId, Player, World};

    #[test]
    fn new_engine_generates_test_world() {
        let engine = Engine::new();

        assert_eq!(engine.world().chunk_count(), 9);
        assert_eq!(
            engine.world().get_block(0, 0, 0),
            BlockId::GRASS_BLOCK
        );
    }

    #[test]
    fn update_moves_player_and_advances_frame() {
        let mut engine = Engine::new();

        let frame = engine.update(
            EngineInput {
                forward: 1.0,
                ..EngineInput::default()
            },
            1.0 / 60.0,
        );

        assert!(engine.player().position()[2] > 0.0);
        assert_eq!(frame.frame_number, 1);
        assert!(frame.elapsed_seconds > 0.0);
    }

    #[test]
    fn engine_applies_gravity() {
        let world = World::new();
        let player = Player::new([0.0, 10.0, 0.0]);
        let mut engine = Engine::with_world(world, player);

        engine.update(EngineInput::default(), 1.0 / 60.0);

        assert!(engine.player().position()[1] < 10.0);
    }

    #[test]
    fn engine_detects_ground() {
        let mut world = World::new();
        world.set_block(0, 0, 0, BlockId::STONE);

        let player = Player::new([0.0, 1.0, 0.0]);
        let mut engine = Engine::with_world(world, player);

        let frame = engine.update(
            EngineInput::default(),
            1.0 / 60.0,
        );

        assert!(frame.on_ground);
    }

    #[test]
    fn engine_can_jump() {
        let mut world = World::new();
        world.set_block(0, 0, 0, BlockId::STONE);

        let player = Player::new([0.0, 1.0, 0.0]);
        let mut engine = Engine::with_world(world, player);

        engine.update(EngineInput::default(), 1.0 / 60.0);

        let frame = engine.update(
            EngineInput {
                jump: true,
                ..EngineInput::default()
            },
            1.0 / 60.0,
        );

        assert!(engine.player().position()[1] > 1.0);
        assert!(!frame.on_ground);
    }

    #[test]
    fn engine_can_target_a_block() {
        let mut world = World::new();
        world.set_block(0, 2, 3, BlockId::STONE);

        let player = Player::new([0.5, 1.0, 0.5]);
        let engine = Engine::with_world(world, player);

        let hit = engine.targeted_block().unwrap();

        assert_eq!(hit.block_position, [0, 2, 3]);
        assert_eq!(hit.block, BlockId::STONE);
    }

    #[test]
    fn engine_can_break_targeted_block() {
        let mut world = World::new();
        world.set_block(0, 2, 3, BlockId::STONE);

        let player = Player::new([0.5, 1.0, 0.5]);
        let mut engine = Engine::with_world(world, player);

        let hit = engine.break_targeted_block().unwrap();

        assert_eq!(hit.block, BlockId::STONE);
        assert_eq!(engine.world().get_block(0, 2, 3), BlockId::AIR);
    }

    #[test]
    fn engine_can_place_block() {
        let mut world = World::new();
        world.set_block(0, 2, 3, BlockId::STONE);

        let player = Player::new([0.5, 1.0, 0.5]);
        let mut engine = Engine::with_world(world, player);

        let placed = engine.place_targeted_block(BlockId::DIRT);

        assert_eq!(placed, Some([0, 2, 2]));
        assert_eq!(engine.world().get_block(0, 2, 2), BlockId::DIRT);
    }

    #[test]
    fn cannot_place_air() {
        let mut world = World::new();
        world.set_block(0, 2, 3, BlockId::STONE);

        let player = Player::new([0.5, 1.0, 0.5]);
        let mut engine = Engine::with_world(world, player);

        assert_eq!(
            engine.place_targeted_block(BlockId::AIR),
            None
        );
    }

    #[test]
    fn cannot_place_block_inside_player() {
        let mut world = World::new();
        world.set_block(0, 2, 3, BlockId::STONE);

        let player = Player::new([0.5, 1.0, 0.5]);
        let mut engine = Engine::with_world(world, player);

        // The block immediately in front of the player is not inside
        // the player's collision box, so normal placement is allowed.
        assert_eq!(
            engine.place_targeted_block(BlockId::STONE),
            Some([0, 2, 2])
        );
    }

    #[test]
    fn invalid_delta_does_not_advance_engine() {
        let mut engine = Engine::new();

        let frame = engine.update(
            EngineInput {
                forward: 1.0,
                ..EngineInput::default()
            },
            f32::NAN,
        );

        assert_eq!(frame.frame_number, 0);
        assert_eq!(frame.elapsed_seconds, 0.0);
        assert_eq!(engine.player().position(), [0.0, 1.0, 0.0]);
    }

    #[test]
    fn large_delta_is_capped() {
        let mut engine = Engine::new();

        let frame = engine.update(EngineInput::default(), 5.0);

        assert!((frame.delta_seconds - 0.1).abs() < 0.0001);
        assert!((frame.elapsed_seconds - 0.1).abs() < 0.0001);
    }

    #[test]
    fn custom_world_and_player_are_preserved() {
        let mut world = World::new();
        world.set_block(5, 0, 5, BlockId::STONE);

        let player = Player::new([5.0, 2.0, 5.0]);
        let engine = Engine::with_world(world, player);

        assert_eq!(
            engine.world().get_block(5, 0, 5),
            BlockId::STONE
        );
        assert_eq!(
            engine.player().position(),
            [5.0, 2.0, 5.0]
        );
    }

    #[test]
    fn reset_player_resets_timing_and_physics() {
        let mut engine = Engine::new();

        engine.update(EngineInput::default(), 0.05);
        engine.reset_player([10.0, 3.0, 10.0]);

        assert_eq!(engine.player().position(), [10.0, 3.0, 10.0]);
        assert_eq!(engine.elapsed_seconds(), 0.0);
        assert_eq!(engine.frame_number(), 0);
        assert!(!engine.physics().is_on_ground());
        assert_eq!(engine.physics().velocity_y(), 0.0);
    }
}
