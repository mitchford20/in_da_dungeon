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

fn spawn_player(mut commands: Commands, level_assets: Res<LevelAssets>) {
    let center = level_assets
        .level_center
        .unwrap_or_else(|| Vec2::new(0.0, 128.0));
    let spawn_position = (center + Vec2::Y * 16.0).extend(1.0);

    commands.spawn((
        Name::new("Player"),
        Player,
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.8, 0.7, 0.6),
                custom_size: Some(Vec2::new(32.0, 48.0)),
                ..default()
            },
            transform: Transform::from_translation(spawn_position),
            ..default()
        },
        Velocity::default(),
        MovementState::default(),
        PlayerController::default(),
        Collider::from_size(Vec2::new(32.0, 48.0)),
    ));
}

fn despawn_player(mut commands: Commands, query: Query<Entity, With<Player>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
