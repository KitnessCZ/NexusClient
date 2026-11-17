
use super::{Player, World};

const PLAYER_WIDTH: f32 = 0.6;
const PLAYER_HEIGHT: f32 = 1.8;
const HALF_WIDTH: f32 = PLAYER_WIDTH * 0.5;

const GRAVITY: f32 = 24.0;
const JUMP_SPEED: f32 = 8.0;
const MAX_FALL_SPEED: f32 = 50.0;
const MAX_STEP_DISTANCE: f32 = 0.2;

#[derive(Clone, Copy, Debug)]
pub struct PhysicsInput {
    pub forward: f32,
    pub right: f32,
    pub jump: bool,
}

impl Default for PhysicsInput {
    fn default() -> Self {
        Self {
            forward: 0.0,
            right: 0.0,
            jump: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerPhysics {
    velocity_y: f32,
    on_ground: bool,
}

impl PlayerPhysics {
    pub fn new() -> Self {
        Self {
            velocity_y: 0.0,
            on_ground: false,
        }
    }

    pub fn velocity_y(&self) -> f32 {
        self.velocity_y
    }

    pub fn is_on_ground(&self) -> bool {
        self.on_ground
    }

    pub fn reset(&mut self) {
        self.velocity_y = 0.0;
        self.on_ground = false;
    }

    pub fn step(
        &mut self,
        world: &World,
        player: &mut Player,
        input: PhysicsInput,
        delta_seconds: f32,
    ) {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return;
        }

        let delta = delta_seconds.min(0.1);

        let forward = if input.forward.is_finite() {
            input.forward.clamp(-1.0, 1.0)
        } else {
            0.0
        };

        let right = if input.right.is_finite() {
            input.right.clamp(-1.0, 1.0)
        } else {
            0.0
        };

        self.move_horizontal(world, player, forward, right, delta);
        self.move_vertical(world, player, input.jump, delta);
    }

    fn move_horizontal(
        &self,
        world: &World,
        player: &mut Player,
        forward_input: f32,
        right_input: f32,
        delta: f32,
    ) {
        let forward = player.horizontal_forward();
        let right = player.right();

        let mut dx =
            forward[0] * forward_input + right[0] * right_input;
        let mut dz =
            forward[2] * forward_input + right[2] * right_input;

        let length_squared = dx * dx + dz * dz;

        if length_squared > 1.0 {
            let inverse_length = length_squared.sqrt().recip();
            dx *= inverse_length;
            dz *= inverse_length;
        }

        let distance = player.movement_speed() * delta;
        dx *= distance;
        dz *= distance;

        let steps = ((dx.abs().max(dz.abs()) / MAX_STEP_DISTANCE).ceil()
            as usize)
            .max(1);

        let step_x = dx / steps as f32;
        let step_z = dz / steps as f32;

        for _ in 0..steps {
            let mut position = player.position();

            position[0] += step_x;

            if !collides(world, position) {
                player.set_position(position);
            }

            position = player.position();
            position[2] += step_z;

            if !collides(world, position) {
                player.set_position(position);
            }
        }
    }

    fn move_vertical(
        &mut self,
        world: &World,
        player: &mut Player,
        jump: bool,
        delta: f32,
    ) {
        if jump && self.on_ground {
            self.velocity_y = JUMP_SPEED;
            self.on_ground = false;
        }

        self.velocity_y =
            (self.velocity_y - GRAVITY * delta).max(-MAX_FALL_SPEED);

        let distance = self.velocity_y * delta;
        let steps = ((distance.abs() / MAX_STEP_DISTANCE).ceil() as usize)
            .max(1);
        let step_y = distance / steps as f32;

        for _ in 0..steps {
            let mut position = player.position();
            position[1] += step_y;

            if collides(world, position) {
                if step_y < 0.0 {
                    self.on_ground = true;
                }

                self.velocity_y = 0.0;
                break;
            }

            player.set_position(position);
            self.on_ground = false;
        }

        if self.velocity_y == 0.0 {
            let mut below = player.position();
            below[1] -= 0.05;
            self.on_ground = collides(world, below);
        }
    }
}

impl Default for PlayerPhysics {
    fn default() -> Self {
        Self::new()
    }
}

fn collides(world: &World, position: [f32; 3]) -> bool {
    let min_x = (position[0] - HALF_WIDTH).floor() as i32;
    let max_x = (position[0] + HALF_WIDTH - 0.0001).floor() as i32;

    let min_y = position[1].floor() as i32;
    let max_y =
        (position[1] + PLAYER_HEIGHT - 0.0001).floor() as i32;

    let min_z = (position[2] - HALF_WIDTH).floor() as i32;
    let max_z = (position[2] + HALF_WIDTH - 0.0001).floor() as i32;

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                if !world.get_block(x, y, z).is_air() {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::{PhysicsInput, PlayerPhysics};
    use crate::engine::{BlockId, Player, World};

    fn test_world() -> World {
        let mut world = World::new();

        for x in -4..=4 {
            for z in -4..=4 {
                world.set_block(x, 0, z, BlockId::STONE);
            }
        }

        world
    }

    #[test]
    fn player_falls_until_reaching_ground() {
        let world = test_world();
        let mut player = Player::new([0.0, 5.0, 0.0]);
        let mut physics = PlayerPhysics::new();

        for _ in 0..120 {
            physics.step(
                &world,
                &mut player,
                PhysicsInput::default(),
                1.0 / 60.0,
            );
        }

        assert!((player.position()[1] - 1.0).abs() < 0.001);
        assert!(physics.is_on_ground());
        assert_eq!(physics.velocity_y(), 0.0);
    }

    #[test]
    fn player_can_jump() {
        let world = test_world();
        let mut player = Player::new([0.0, 1.0, 0.0]);
        let mut physics = PlayerPhysics::new();

        physics.step(
            &world,
            &mut player,
            PhysicsInput::default(),
            1.0 / 60.0,
        );

        assert!(physics.is_on_ground());

        physics.step(
            &world,
            &mut player,
            PhysicsInput {
                jump: true,
                ..PhysicsInput::default()
            },
            1.0 / 60.0,
        );

        assert!(player.position()[1] > 1.0);
        assert!(!physics.is_on_ground());
    }

    #[test]
    fn player_cannot_walk_through_wall() {
        let mut world = test_world();

        for y in 1..=3 {
            world.set_block(1, y, 0, BlockId::STONE);
        }

        let mut player = Player::new([0.0, 1.0, 0.0]);
        let mut physics = PlayerPhysics::new();

        for _ in 0..60 {
            physics.step(
                &world,
                &mut player,
                PhysicsInput {
                    right: 1.0,
                    ..PhysicsInput::default()
                },
                1.0 / 60.0,
            );
        }

        assert!(player.position()[0] < 0.71);
    }

    #[test]
    fn invalid_delta_does_not_move_player() {
        let world = test_world();
        let mut player = Player::new([0.0, 1.0, 0.0]);
        let mut physics = PlayerPhysics::new();

        physics.step(
            &world,
            &mut player,
            PhysicsInput {
                forward: 1.0,
                jump: true,
                ..PhysicsInput::default()
            },
            f32::NAN,
        );

        assert_eq!(player.position(), [0.0, 1.0, 0.0]);
    }
}
