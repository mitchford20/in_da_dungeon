use bevy::prelude::*;

use crate::level::LevelAssets;
use crate::movement::{Collider, MovementState, PlayerController, Velocity};
use crate::state::GameState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(OnExit(GameState::Playing), despawn_player);
    }
}

#[derive(Component)]
pub struct Player;

fn spawn_player(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    asset_server: Res<AssetServer>,
) {
    let default_spawn = Vec2::new(30.0, 60.0);
    let spawn_2d = level_assets
        .level_origin
        .map(|origin| origin + default_spawn)
        .unwrap_or(default_spawn);
    let spawn_position = spawn_2d.extend(1.0);

    let texture = asset_server.load("textures/blob.png");
    let sprite_size = Vec2::splat(32.0);

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
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
