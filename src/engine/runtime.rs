
use super::{generate_test_world, Camera, Player, World};

/// Input supplied to one engine update.
#[derive(Clone, Copy, Debug, Default)]
pub struct EngineInput {
    /// Positive moves forward; negative moves backward.
    pub forward: f32,

    /// Positive moves right; negative moves left.
    pub right: f32,

    /// Horizontal look change in radians.
    pub look_yaw: f32,

    /// Vertical look change in radians.
    pub look_pitch: f32,
}

/// Data produced after an engine update.
///
/// The renderer can consume this without depending on the engine's
/// internal world or player implementation.
#[derive(Clone, Copy, Debug)]
pub struct EngineFrame {
    pub camera: Camera,
    pub delta_seconds: f32,
    pub elapsed_seconds: f64,
    pub frame_number: u64,
}

/// Main runtime state for NexusClient.
pub struct Engine {
    world: World,
    player: Player,
    elapsed_seconds: f64,
    frame_number: u64,
}

impl Engine {
    /// Creates an engine with a generated flat test world.
    pub fn new() -> Self {
        let mut world = World::new();
        generate_test_world(&mut world);

        Self {
            world,
            player: Player::default(),
            elapsed_seconds: 0.0,
            frame_number: 0,
        }
    }

    /// Creates an engine using an existing world and player.
    pub fn with_world(world: World, player: Player) -> Self {
        Self {
            world,
            player,
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

    pub fn elapsed_seconds(&self) -> f64 {
        self.elapsed_seconds
    }

    pub fn frame_number(&self) -> u64 {
        self.frame_number
    }

    /// Processes input and advances the engine by one frame.
    ///
    /// Invalid or non-positive frame times are ignored.
    /// Large frame times are capped to avoid huge movement after pauses.
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

            self.player.move_local(
                input.forward,
                input.right,
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
        }
    }

    /// Resets the player and engine timing without deleting the world.
    pub fn reset_player(&mut self, position: [f32; 3]) {
        self.player = Player::new(position);
        self.elapsed_seconds = 0.0;
        self.frame_number = 0;
    }
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
    fn reset_player_resets_timing() {
        let mut engine = Engine::new();

        engine.update(EngineInput::default(), 0.05);
        engine.reset_player([10.0, 3.0, 10.0]);

        assert_eq!(engine.player().position(), [10.0, 3.0, 10.0]);
        assert_eq!(engine.elapsed_seconds(), 0.0);
        assert_eq!(engine.frame_number(), 0);
    }
}
