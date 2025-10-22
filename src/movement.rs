//! Player movement systems: input sampling, kinematic integration, and tile collision resolution.
//!
//! The high numeric values used here reflect the world-unit scale (1 unit = 1 LDtk pixel). Because
//! sprites are small, accelerations and gravity must be large to achieve responsive motion. No
//! manual memory management is needed—the ECS owns component data.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::collision::CollisionMap;
use crate::state::{GameSet, GameState};

/// Registers movement-related systems. The plugin itself carries no runtime state.
pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MovementSettings>().add_systems(
            Update,
            (
                read_player_input.in_set(GameSet::Input),
                apply_kinematics.in_set(GameSet::Movement),
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Global physics tuning parameters. Stored as a resource so designers can tweak gravity or
/// terminal velocity at runtime (e.g., via debug UI).
#[derive(Resource)]
pub struct MovementSettings {
    pub gravity: f32,
    pub terminal_velocity: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            gravity: 1150.0,
            terminal_velocity: -1800.0,
        }
    }
}

/// Velocity component storing horizontal/vertical speeds in world units per second. The `Deref`
/// derives let systems treat it like a `Vec2` while still enabling component borrowing.
#[derive(Component, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

/// Controller tuning specific to the player. Acceleration values are large to hit max speed in a
/// fraction of a second, keeping movement snappy given the pixel-scale world units.
#[derive(Component)]
pub struct PlayerController {
    pub ground_accel: f32,
    pub air_accel: f32,
    pub ground_max_speed: f32,
    pub air_max_speed: f32,
    pub jump_strength: f32,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            ground_accel: 1600.0,
            air_accel: 1200.0,
            ground_max_speed: 325.0,
            air_max_speed: 275.0,
            jump_strength: 480.0,
        }
    }
}

/// Per-entity movement state flags. `axis` stores the last input direction so the kinematics system
/// can ramp velocity toward the desired target after the input sampling stage.
#[derive(Component)]
pub struct MovementState {
    pub on_ground: bool,
    pub wants_jump: bool,
    pub axis: f32,
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            on_ground: true,
            wants_jump: false,
            axis: 0.0,
        }
    }
}

/// Axis-aligned bounding box for collision checks. Only half extents are stored because they make
/// overlap tests inexpensive.
#[derive(Component, Copy, Clone)]
pub struct Collider {
    pub half_extents: Vec2,
}

impl Collider {
    pub fn from_size(size: Vec2) -> Self {
        Self {
            half_extents: size * 0.5,
        }
    }
}

/// Samples keyboard input and writes intent into the movement state. Using a separate system keeps
/// input handling deterministic and easy to test.
fn read_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&PlayerController, &mut Velocity, &mut MovementState)>,
) {
    for (_controller, mut velocity, mut state) in &mut query {
        let mut axis: f32 = 0.0;
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            axis -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            axis += 1.0;
        }

        state.axis = axis.clamp(-1.0, 1.0);

        if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::ArrowUp) {
            state.wants_jump = true;
        }

        // Zero-out tiny residual velocities when grounded for crisp stopping behaviour.
        if state.axis.abs() < f32::EPSILON && state.on_ground && velocity.x.abs() < 1.0 {
            velocity.x = 0.0;
        }
    }
}

/// Applies physics each frame: acceleration toward target velocity, gravity, collision sweeps, and
/// jump execution. All calculations mutate `Transform`/`Velocity` in place; Bevy batches those
/// writes and applies them after the system completes.
fn apply_kinematics(
    time: Res<Time>,
    settings: Res<MovementSettings>,
    collision_map: Res<CollisionMap>,
    mut query: Query<(
        &mut Transform,
        &mut Velocity,
        &mut MovementState,
        &PlayerController,
        &Collider,
    )>,
) {
    let dt = time.delta_seconds();

    for (mut transform, mut velocity, mut state, controller, collider) in &mut query {
        // Capture jump intent so we can resolve collisions before applying it. This avoids the
        // classic "press jump on the landing frame" issue where intent would be cleared too early.
        let wants_jump = state.wants_jump;
        state.wants_jump = false;

        if !state.on_ground {
            velocity.y -= settings.gravity * dt;
            if velocity.y < settings.terminal_velocity {
                velocity.y = settings.terminal_velocity;
            }
        } else if velocity.y < 0.0 {
            velocity.y = 0.0;
        }

        let (accel_rate, max_speed) = if state.on_ground {
            (controller.ground_accel, controller.ground_max_speed)
        } else {
            (controller.air_accel, controller.air_max_speed)
        };

        if state.axis.abs() > f32::EPSILON {
            let target = state.axis * max_speed;
            velocity.x = move_towards(velocity.x, target, accel_rate * dt);
        } else {
            velocity.x = move_towards(velocity.x, 0.0, accel_rate * dt);
        }

        let mut position = transform.translation;
        let half = collider.half_extents;

        resolve_horizontal(&mut position, &mut velocity.x, half, dt, &collision_map);
        let vertical_collision =
            resolve_vertical(&mut position, &mut velocity.y, half, dt, &collision_map);

        let grounded = vertical_collision.down || grounded_check(position, half, &collision_map);

        state.on_ground = grounded;

        if wants_jump && state.on_ground {
            velocity.y = controller.jump_strength;
            state.on_ground = false;
        }

        transform.translation = position;
    }
}

/// Flags describing whether a vertical sweep collided above or below the player.
struct VerticalCollision {
    down: bool,
    up: bool,
}

const SKIN: f32 = 0.001;

/// Resolves horizontal position/velocity against the collision map using a swept AABB. The tiny
/// `SKIN` offset prevents the collider from getting stuck on edges by keeping it a hair away from
/// solid tiles.
fn resolve_horizontal(
    position: &mut Vec3,
    velocity: &mut f32,
    half: Vec2,
    dt: f32,
    map: &CollisionMap,
) {
    if velocity.abs() < f32::EPSILON {
        return;
    }

    let new_x = position.x + *velocity * dt;
    let dir = velocity.signum();

    let bottom = position.y - half.y + SKIN;
    let top = position.y + half.y - SKIN;

    let tile_size = map.tile_size.x;
    let min_tile_y = ((bottom - map.origin.y) / map.tile_size.y).floor() as i32;
    let max_tile_y = ((top - map.origin.y) / map.tile_size.y).floor() as i32;

    if dir > 0.0 {
        let edge = new_x + half.x;
        let tile_x = ((edge - map.origin.x) / tile_size).floor() as i32;
        for ty in min_tile_y..=max_tile_y {
            if map.is_solid(IVec2::new(tile_x, ty)) {
                let tile_left = map.origin.x + tile_x as f32 * tile_size;
                position.x = tile_left - half.x - SKIN;
                *velocity = 0.0;
                return;
            }
        }
    } else if dir < 0.0 {
        let edge = new_x - half.x;
        let tile_x = ((edge - map.origin.x) / tile_size).floor() as i32;
        for ty in min_tile_y..=max_tile_y {
            if map.is_solid(IVec2::new(tile_x, ty)) {
                let tile_right = map.origin.x + (tile_x + 1) as f32 * tile_size;
                position.x = tile_right + half.x + SKIN;
                *velocity = 0.0;
                return;
            }
        }
    }

    position.x = new_x;
}

/// Vertical counterpart to `resolve_horizontal`. Returns whether a collision occurred above or
/// below so grounded state can be updated. All arithmetic is in f32 and only local temporaries are
/// allocated on the stack.
fn resolve_vertical(
    position: &mut Vec3,
    velocity: &mut f32,
    half: Vec2,
    dt: f32,
    map: &CollisionMap,
) -> VerticalCollision {
    let mut collision = VerticalCollision {
        down: false,
        up: false,
    };

    let new_y = position.y + *velocity * dt;
    let dir = velocity.signum();
    let left = position.x - half.x + SKIN;
    let right = position.x + half.x - SKIN;
    let tile_width = map.tile_size.x;
    let tile_height = map.tile_size.y;
    let min_tile_x = ((left - map.origin.x) / tile_width).floor() as i32;
    let max_tile_x = ((right - map.origin.x) / tile_width).floor() as i32;

    if dir < 0.0 {
        let edge = new_y - half.y;
        let tile_y = ((edge - map.origin.y) / tile_height).floor() as i32;
        for tx in min_tile_x..=max_tile_x {
            if map.is_solid(IVec2::new(tx, tile_y)) {
                let tile_top = map.origin.y + (tile_y + 1) as f32 * tile_height;
                position.y = tile_top + half.y + SKIN;
                *velocity = 0.0;
                collision.down = true;
                return collision;
            }
        }
    } else if dir > 0.0 {
        let edge = new_y + half.y;
        let tile_y = ((edge - map.origin.y) / tile_height).floor() as i32;
        for tx in min_tile_x..=max_tile_x {
            if map.is_solid(IVec2::new(tx, tile_y)) {
                let tile_bottom = map.origin.y + tile_y as f32 * tile_height;
                position.y = tile_bottom - half.y - SKIN;
                *velocity = 0.0;
                collision.up = true;
                return collision;
            }
        }
    }

    position.y = new_y;
    collision
}

/// Moves `current` toward `target` by at most `max_delta`, preserving smooth acceleration and
/// deceleration curves.
fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    let delta = target - current;
    if delta.abs() <= max_delta {
        target
    } else {
        current + delta.signum() * max_delta
    }
}

/// Secondary grounded check that samples just below the feet. Helps catch situations where the
/// player barely leaves the ground for a single frame (e.g., sliding down steps) to avoid jump
/// input loss.
fn grounded_check(position: Vec3, half: Vec2, map: &CollisionMap) -> bool {
    let foot = position.y - half.y;
    let probe = foot - SKIN * 2.0;
    let tile_height = map.tile_size.y;
    let tile_width = map.tile_size.x;

    let tile_y = ((probe - map.origin.y) / tile_height).floor() as i32;
    let left = position.x - half.x + SKIN;
    let right = position.x + half.x - SKIN;
    let min_tile_x = ((left - map.origin.x) / tile_width).floor() as i32;
    let max_tile_x = ((right - map.origin.x) / tile_width).floor() as i32;

    for tx in min_tile_x..=max_tile_x {
        if map.is_solid(IVec2::new(tx, tile_y)) {
            let tile_top = map.origin.y + (tile_y + 1) as f32 * tile_height;
            if foot >= tile_top - SKIN * 4.0 {
                return true;
            }
        }
    }

    false
}
