
use std::f32::consts::{FRAC_PI_2, PI, TAU};

const MAX_PITCH: f32 = FRAC_PI_2 - 0.01;

/// Basic first-person player state.
///
/// World coordinates:
/// - X: east/west
/// - Y: up/down
/// - Z: north/south
///
/// A yaw of zero looks toward positive Z.
#[derive(Clone, Copy, Debug)]
pub struct Player {
    position: [f32; 3],
    yaw: f32,
    pitch: f32,
    movement_speed: f32,
}

impl Player {
    pub const DEFAULT_EYE_HEIGHT: f32 = 1.62;

    pub fn new(position: [f32; 3]) -> Self {
        Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
            movement_speed: 4.3,
        }
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn set_position(&mut self, position: [f32; 3]) {
        self.position = position;
    }

    pub fn rotation(&self) -> [f32; 2] {
        [self.yaw, self.pitch]
    }

    pub fn set_rotation(&mut self, yaw: f32, pitch: f32) {
        if !yaw.is_finite() || !pitch.is_finite() {
            return;
        }

        self.yaw = yaw.rem_euclid(TAU);
        self.pitch = pitch.clamp(-MAX_PITCH, MAX_PITCH);
    }

    /// Changes the view direction.
    ///
    /// Both arguments are radians, not degrees.
    pub fn look(&mut self, yaw_delta: f32, pitch_delta: f32) {
        if !yaw_delta.is_finite() || !pitch_delta.is_finite() {
            return;
        }

        self.set_rotation(
            self.yaw + yaw_delta,
            self.pitch + pitch_delta,
        );
    }

    pub fn movement_speed(&self) -> f32 {
        self.movement_speed
    }

    pub fn set_movement_speed(&mut self, speed: f32) {
        if speed.is_finite() && speed >= 0.0 {
            self.movement_speed = speed;
        }
    }

    /// Returns the horizontal forward direction.
    pub fn horizontal_forward(&self) -> [f32; 3] {
        [
            self.yaw.sin(),
            0.0,
            self.yaw.cos(),
        ]
    }

    /// Returns the horizontal right direction.
    pub fn right(&self) -> [f32; 3] {
        [
            self.yaw.cos(),
            0.0,
            -self.yaw.sin(),
        ]
    }

    /// Returns the full 3D direction the player is looking.
    pub fn forward(&self) -> [f32; 3] {
        let cos_pitch = self.pitch.cos();

        [
            self.yaw.sin() * cos_pitch,
            self.pitch.sin(),
            self.yaw.cos() * cos_pitch,
        ]
    }

    /// Moves relative to the player's horizontal facing direction.
    ///
    /// `forward_input`:
    /// - positive = forward
    /// - negative = backward
    ///
    /// `right_input`:
    /// - positive = right
    /// - negative = left
    ///
    /// `delta_seconds` is frame time in seconds.
    pub fn move_local(
        &mut self,
        forward_input: f32,
        right_input: f32,
        delta_seconds: f32,
    ) {
        if !forward_input.is_finite()
            || !right_input.is_finite()
            || !delta_seconds.is_finite()
            || delta_seconds <= 0.0
        {
            return;
        }

        let forward = self.horizontal_forward();
        let right = self.right();

        let mut move_x =
            forward[0] * forward_input + right[0] * right_input;
        let mut move_z =
            forward[2] * forward_input + right[2] * right_input;

        let length_squared = move_x * move_x + move_z * move_z;

        if length_squared > 1.0 {
            let inverse_length = length_squared.sqrt().recip();
            move_x *= inverse_length;
            move_z *= inverse_length;
        }

        let distance = self.movement_speed * delta_seconds;

        self.position[0] += move_x * distance;
        self.position[2] += move_z * distance;
    }

    /// Creates a first-person camera at the player's eye position.
    pub fn camera(&self) -> Camera {
        Camera::from_player(self)
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new([0.0, 1.0, 0.0])
    }
}

/// Camera data prepared from the player's current state.
///
/// This is renderer-independent. Vulkan and OpenGL ES renderers
/// can both consume this camera representation.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    position: [f32; 3],
    forward: [f32; 3],
    up: [f32; 3],
    fov_y_radians: f32,
    near_plane: f32,
    far_plane: f32,
}

impl Camera {
    pub const DEFAULT_FOV_DEGREES: f32 = 70.0;
    pub const DEFAULT_NEAR_PLANE: f32 = 0.05;
    pub const DEFAULT_FAR_PLANE: f32 = 512.0;

    pub fn from_player(player: &Player) -> Self {
        let player_position = player.position();

        Self {
            position: [
                player_position[0],
                player_position[1] + Player::DEFAULT_EYE_HEIGHT,
                player_position[2],
            ],
            forward: player.forward(),
            up: [0.0, 1.0, 0.0],
            fov_y_radians: Self::DEFAULT_FOV_DEGREES.to_radians(),
            near_plane: Self::DEFAULT_NEAR_PLANE,
            far_plane: Self::DEFAULT_FAR_PLANE,
        }
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn forward(&self) -> [f32; 3] {
        self.forward
    }

    pub fn up(&self) -> [f32; 3] {
        self.up
    }

    pub fn fov_y_radians(&self) -> f32 {
        self.fov_y_radians
    }

    pub fn near_plane(&self) -> f32 {
        self.near_plane
    }

    pub fn far_plane(&self) -> f32 {
        self.far_plane
    }

    pub fn set_fov_degrees(&mut self, degrees: f32) {
        if degrees.is_finite() && (1.0..179.0).contains(&degrees) {
            self.fov_y_radians = degrees.to_radians();
        }
    }

    pub fn set_clip_planes(&mut self, near: f32, far: f32) {
        if near.is_finite()
            && far.is_finite()
            && near > 0.0
            && far > near
        {
            self.near_plane = near;
            self.far_plane = far;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Camera, Player};

    #[test]
    fn default_player_has_expected_position() {
        let player = Player::default();

        assert_eq!(player.position(), [0.0, 1.0, 0.0]);
    }

    #[test]
    fn player_moves_forward() {
        let mut player = Player::new([0.0, 0.0, 0.0]);

        player.move_local(1.0, 0.0, 1.0);

        let position = player.position();

        assert!((position[0]).abs() < 0.0001);
        assert!((position[2] - 4.3).abs() < 0.0001);
    }

    #[test]
    fn diagonal_movement_is_normalized() {
        let mut player = Player::new([0.0, 0.0, 0.0]);

        player.move_local(1.0, 1.0, 1.0);

        let position = player.position();
        let distance = (position[0] * position[0]
            + position[2] * position[2])
            .sqrt();

        assert!((distance - 4.3).abs() < 0.0001);
    }

    #[test]
    fn looking_changes_forward_direction() {
        let mut player = Player::default();

        player.set_rotation(0.0, 0.0);
        let before = player.forward();

        player.look(std::f32::consts::FRAC_PI_2, 0.0);
        let after = player.forward();

        assert!((before[2] - 1.0).abs() < 0.0001);
        assert!((after[0] - 1.0).abs() < 0.0001);
    }

    #[test]
    fn pitch_is_clamped() {
        let mut player = Player::default();

        player.set_rotation(0.0, 100.0);

        assert!(player.rotation()[1] < std::f32::consts::FRAC_PI_2);
    }

    #[test]
    fn camera_uses_player_eye_position() {
        let player = Player::new([2.0, 3.0, 4.0]);
        let camera = player.camera();

        assert_eq!(camera.position(), [2.0, 4.62, 4.0]);
        assert_eq!(camera.forward(), player.forward());
    }

    #[test]
    fn invalid_movement_does_not_change_position() {
        let mut player = Player::default();
        let before = player.position();

        player.move_local(1.0, 0.0, f32::NAN);

        assert_eq!(player.position(), before);
    }

    #[test]
    fn camera_has_valid_default_projection_settings() {
        let mut camera = Camera::from_player(&Player::default());

        assert!(camera.fov_y_radians() > 0.0);
        assert!(camera.near_plane() > 0.0);
        assert!(camera.far_plane() > camera.near_plane());

        camera.set_fov_degrees(80.0);
        assert!((camera.fov_y_radians() - 80.0_f32.to_radians()).abs() < 0.0001);
    }
}
