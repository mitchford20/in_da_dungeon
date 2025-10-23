//! Player entity lifecycle management. Handles spawning the avatar with the correct components
//! and cleaning it up when leaving the gameplay state.
//!
//! All memory for components is owned by Bevy's ECS tables; this module merely issues spawn/
//! despawn commands and lets Rust drop the components automatically when the entity is removed.

use bevy::prelude::*;

use crate::level::LevelAssets;
use crate::movement::{Collider, MovementState, PlayerController, Velocity};
use crate::state::GameState;
use crate::transition::SpawnPositions;

/// Registers systems that keep exactly one player entity alive while in the `Playing` state.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_player_if_needed.run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), despawn_player);
    }
}

/// Marker component used by many systems (camera follow, collision queries) to identify the player
/// entity. The component itself stores no data and therefore adds zero heap overhead.
#[derive(Component)]
pub struct Player;

/// Spawns the player once the LDtk world origin is known. If the world is still loading or a
/// player already exists, the system exits earl
fn spawn_player_if_needed(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    asset_server: Res<AssetServer>,
    spawn_positions: Res<SpawnPositions>,
    existing_player: Query<Entity, With<Player>>,
) {
    if !existing_player.is_empty() {
        return;
    }

    let Some(origin) = level_assets.level_origin else {
        return;
    };

    // Get spawn position based on the current level file
    let project_path = level_assets.project_path.as_deref().unwrap_or("levels/test_map_1_newres.ldtk");
    let spawn_offset = spawn_positions.get(project_path);
    let spawn_2d = origin + spawn_offset;
    // Place the sprite slightly in front of tile layers so it renders above the map.
    let spawn_position = spawn_2d.extend(200.0);

    let texture = asset_server.load("textures/blob.png");
    let sprite_size = Vec2::splat(32.0);

    // Spawn the player entity. The tuple inserted into the ECS is stored in a contiguous archetype
    // row, so memory access during gameplay remains cache-friendly.
    commands.spawn((
        Name::new("Player"),
        Player,
        SpriteBundle {
            texture,
            sprite: Sprite {
                custom_size: Some(sprite_size),
                ..default()
            },
            transform: Transform::from_translation(spawn_position),
            ..default()
        },
        Velocity::default(),
        MovementState::default(),
        PlayerController::default(),
        Collider::from_size(sprite_size),
    ));
}

fn despawn_player(mut commands: Commands, query: Query<Entity, With<Player>>) {
    // Remove the player entity and all of its components. No manual memory management required—
    // Bevy drops each component as part of the despawn operation.
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
