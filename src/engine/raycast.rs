
use super::{BlockId, World};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockHit {
    pub block_position: [i32; 3],
    pub previous_position: [i32; 3],
    pub face_normal: [i32; 3],
    pub block: BlockId,
    pub distance: f32,
}

pub fn raycast(
    world: &World,
    origin: [f32; 3],
    direction: [f32; 3],
    max_distance: f32,
) -> Option<BlockHit> {
    if !max_distance.is_finite()
        || max_distance < 0.0
        || origin.iter().any(|v| !v.is_finite())
        || direction.iter().any(|v| !v.is_finite())
    {
        return None;
    }

    let length_squared =
        direction[0] * direction[0]
        + direction[1] * direction[1]
        + direction[2] * direction[2];

    if length_squared <= f32::EPSILON {
        return None;
    }

    let inverse_length = length_squared.sqrt().recip();
    let direction = [
        direction[0] * inverse_length,
        direction[1] * inverse_length,
        direction[2] * inverse_length,
    ];

    let mut cell = [
        origin[0].floor() as i32,
        origin[1].floor() as i32,
        origin[2].floor() as i32,
    ];

    let step = [
        sign(direction[0]),
        sign(direction[1]),
        sign(direction[2]),
    ];

    let t_delta = [
        axis_delta(direction[0]),
        axis_delta(direction[1]),
        axis_delta(direction[2]),
    ];

    let mut t_max = [
        first_boundary(origin[0], cell[0], direction[0]),
        first_boundary(origin[1], cell[1], direction[1]),
        first_boundary(origin[2], cell[2], direction[2]),
    ];

    let mut previous = cell;
    let mut normal = [0, 0, 0];
    let mut distance = 0.0;

    loop {
        let block = world.get_block(cell[0], cell[1], cell[2]);

        if !block.is_air() {
            return Some(BlockHit {
                block_position: cell,
                previous_position: previous,
                face_normal: normal,
                block,
                distance,
            });
        }

        previous = cell;

        let axis = if t_max[0] <= t_max[1]
            && t_max[0] <= t_max[2]
        {
            0
        } else if t_max[1] <= t_max[2] {
            1
        } else {
            2
        };

        distance = t_max[axis];

        if !distance.is_finite() || distance > max_distance {
            return None;
        }

        cell[axis] += step[axis];
        normal = [0, 0, 0];
        normal[axis] = -step[axis];

        t_max[axis] += t_delta[axis];
    }
}

fn sign(value: f32) -> i32 {
    if value > 0.0 {
        1
    } else if value < 0.0 {
        -1
    } else {
        0
    }
}

fn axis_delta(value: f32) -> f32 {
    if value == 0.0 {
        f32::INFINITY
    } else {
        value.abs().recip()
    }
}

fn first_boundary(origin: f32, cell: i32, direction: f32) -> f32 {
    if direction > 0.0 {
        (cell as f32 + 1.0 - origin) / direction
    } else if direction < 0.0 {
        (origin - cell as f32) / -direction
    } else {
        f32::INFINITY
    }
}

#[cfg(test)]
mod tests {
    use super::raycast;
    use crate::engine::{BlockId, World};

    #[test]
    fn finds_block_in_front_of_ray() {
        let mut world = World::new();
        world.set_block(0, 1, 3, BlockId::STONE);

        let hit = raycast(
            &world,
            [0.5, 1.5, 0.5],
            [0.0, 0.0, 1.0],
            6.0,
        )
        .unwrap();

        assert_eq!(hit.block_position, [0, 1, 3]);
        assert_eq!(hit.previous_position, [0, 1, 2]);
        assert_eq!(hit.face_normal, [0, 0, -1]);
        assert_eq!(hit.block, BlockId::STONE);
    }

    #[test]
    fn finds_block_in_negative_coordinates() {
        let mut world = World::new();
        world.set_block(-3, 1, 0, BlockId::DIRT);

        let hit = raycast(
            &world,
            [0.5, 1.5, 0.5],
            [-1.0, 0.0, 0.0],
            6.0,
        )
        .unwrap();

        assert_eq!(hit.block_position, [-3, 1, 0]);
        assert_eq!(hit.face_normal, [1, 0, 0]);
    }

    #[test]
    fn respects_maximum_distance() {
        let mut world = World::new();
        world.set_block(0, 1, 5, BlockId::STONE);

        assert!(raycast(
            &world,
            [0.5, 1.5, 0.5],
            [0.0, 0.0, 1.0],
            3.0,
        ).is_none());
    }

    #[test]
    fn returns_none_for_empty_world() {
        let world = World::new();

        assert!(raycast(
            &world,
            [0.5, 1.5, 0.5],
            [0.0, 0.0, 1.0],
            10.0,
        ).is_none());
    }

    #[test]
    fn rejects_zero_direction() {
        let mut world = World::new();
        world.set_block(0, 0, 0, BlockId::STONE);

        assert!(raycast(
            &world,
            [0.5, 0.5, 0.5],
            [0.0, 0.0, 0.0],
            5.0,
        ).is_none());
    }

    #[test]
    fn detects_block_containing_ray_origin() {
        let mut world = World::new();
        world.set_block(0, 0, 0, BlockId::STONE);

        let hit = raycast(
            &world,
            [0.5, 0.5, 0.5],
            [1.0, 0.0, 0.0],
            5.0,
        ).unwrap();

        assert_eq!(hit.block_position, [0, 0, 0]);
        assert_eq!(hit.distance, 0.0);
    }
}
