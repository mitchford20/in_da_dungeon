//! Level transition system with fade effects. Detects when the player touches special trigger tiles
//! (IntGrid value 2) and smoothly transitions to the next level with a black screen fade.

use bevy::math::IVec2;
use bevy::prelude::*;

use crate::collision::CollisionMap;
use crate::level::{LevelAssets, LevelConfig};
use crate::player::Player;
use crate::state::{GameSet, GameState};

/// Registers the transition system and fade overlay.
pub struct TransitionPlugin;

impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransitionState>()
            .init_resource::<SpawnPositions>()
            .add_systems(
                Update,
                (
                    check_level_triggers.in_set(GameSet::Effects),
                    update_transition.in_set(GameSet::Effects),
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::Playing), spawn_fade_overlay)
            .add_systems(Update, update_fade_overlay);
    }
}

/// Tracks the current transition state and timing.
#[derive(Resource, Default)]
pub struct TransitionState {
    pub is_transitioning: bool,
    pub fade_timer: f32,
    pub fade_duration: f32,
    pub next_level_path: Option<String>,
    pub next_level_name: Option<String>,
}

impl TransitionState {
    pub fn start_transition(&mut self, level_path: String, level_name: String) {
        self.is_transitioning = true;
        self.fade_timer = 0.0;
        self.fade_duration = 1.0; // Total fade time (0.5 out + 0.5 in)
        self.next_level_path = Some(level_path);
        self.next_level_name = Some(level_name);
    }

    pub fn reset(&mut self) {
        self.is_transitioning = false;
        self.fade_timer = 0.0;
        self.next_level_path = None;
        self.next_level_name = None;
    }

    /// Returns the current fade alpha (0.0 = transparent, 1.0 = fully black)
    pub fn get_fade_alpha(&self) -> f32 {
        if !self.is_transitioning {
            return 0.0;
        }

        let half_duration = self.fade_duration * 0.5;
        if self.fade_timer < half_duration {
            // Fade out
            self.fade_timer / half_duration
        } else {
            // Fade in
            1.0 - ((self.fade_timer - half_duration) / half_duration)
        }
    }
}

/// Stores spawn positions for each level by project path.
#[derive(Resource)]
pub struct SpawnPositions {
    positions: std::collections::HashMap<String, Vec2>,
}

impl Default for SpawnPositions {
    fn default() -> Self {
        let mut positions = std::collections::HashMap::new();
        positions.insert("levels/test_map_1_newres.ldtk".to_owned(), Vec2::new(340.0, 340.0));
        positions.insert("levels/level_2.ldtk".to_owned(), Vec2::new(57.0, 552.0));
        Self { positions }
    }
}

impl SpawnPositions {
    pub fn get(&self, project_path: &str) -> Vec2 {
        self.positions
            .get(project_path)
            .copied()
            .unwrap_or(Vec2::new(340.0, 340.0))
    }
}

/// Marker component for the fade overlay sprite.
#[derive(Component)]
pub struct FadeOverlay;

/// Spawns a fullscreen black overlay for fade transitions.
fn spawn_fade_overlay(mut commands: Commands) {
    // Check if overlay already exists
    commands.spawn((
        FadeOverlay,
        Name::new("FadeOverlay"),
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(10000.0, 10000.0)), // Large enough to cover screen
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1000.0)), // High Z to render on top
            ..default()
        },
    ));
}

/// Updates the fade overlay opacity based on transition state.
fn update_fade_overlay(
    transition: Res<TransitionState>,
    mut overlay_query: Query<&mut Sprite, With<FadeOverlay>>,
) {
    for mut sprite in &mut overlay_query {
        let alpha = transition.get_fade_alpha();
        sprite.color = Color::srgba(0.0, 0.0, 0.0, alpha);
    }
}

/// Checks if the player is touching a trigger tile (value 2) and initiates level transition.
fn check_level_triggers(
    player_query: Query<(&Transform, &crate::movement::Collider), With<Player>>,
    collision_map: Res<CollisionMap>,
    mut transition: ResMut<TransitionState>,
    level_assets: Res<LevelAssets>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if transition.is_transitioning {
        return;
    }

    let Ok((transform, collider)) = player_query.get_single() else {
        return;
    };

    let position = transform.translation.truncate();
    let half_size = collider.half_extents;

    // Debug: Press 'T' to print collision map info
    if keyboard.just_pressed(KeyCode::KeyT) {
        info!("=== Collision Map Debug ===");
        info!("Total tiles in map: {}", collision_map.tile_values.len());
        info!("Player position: {:?}", position);
        info!("Map origin: {:?}", collision_map.origin);
        info!("Tile size: {:?}", collision_map.tile_size);

        let mut value_2_tiles = Vec::new();
        for (tile_pos, value) in &collision_map.tile_values {
            if *value == 2 {
                value_2_tiles.push(*tile_pos);
            }
        }
        info!("Value 2 tiles found: {:?}", value_2_tiles);

        // Show what tiles the player is currently checking
        info!("=== Player Tile Check ===");
        for (i, offset) in [
            Vec2::ZERO,
            Vec2::new(-half_size.x, -half_size.y),
            Vec2::new(half_size.x, -half_size.y),
            Vec2::new(-half_size.x, half_size.y),
            Vec2::new(half_size.x, half_size.y),
        ].iter().enumerate() {
            let check_pos = position + *offset;
            let tile_x = ((check_pos.x - collision_map.origin.x) / collision_map.tile_size.x).floor() as i32;
            let tile_y = ((check_pos.y - collision_map.origin.y) / collision_map.tile_size.y).floor() as i32;
            let tile = IVec2::new(tile_x, tile_y);
            let value = collision_map.get_tile_value(tile);
            info!("  Check point {}: world_pos={:?}, tile={:?}, value={:?}", i, check_pos, tile, value);
        }
    }

    // Check the center and 4 corners plus middle edges of the player's collider
    let offsets = [
        Vec2::ZERO, // Center
        Vec2::new(-half_size.x, -half_size.y), // Bottom-left
        Vec2::new(half_size.x, -half_size.y),  // Bottom-right
        Vec2::new(-half_size.x, half_size.y),  // Top-left
        Vec2::new(half_size.x, half_size.y),   // Top-right
        Vec2::new(0.0, -half_size.y),          // Bottom-center
        Vec2::new(0.0, half_size.y),           // Top-center
        Vec2::new(-half_size.x, 0.0),          // Left-center
        Vec2::new(half_size.x, 0.0),           // Right-center
    ];

    for offset in &offsets {
        let check_pos = position + *offset;
        let tile_x = ((check_pos.x - collision_map.origin.x) / collision_map.tile_size.x).floor() as i32;
        let tile_y = ((check_pos.y - collision_map.origin.y) / collision_map.tile_size.y).floor() as i32;
        let tile = IVec2::new(tile_x, tile_y);

        if let Some(value) = collision_map.get_tile_value(tile) {
            if value == 2 {
                info!("Detected value 2 tile at {:?}, triggering transition!", tile);
                // Trigger transition to second level
                let current_path = level_assets.project_path.as_deref().unwrap_or("levels/test_map_1_newres.ldtk");
                if current_path == "levels/test_map_1_newres.ldtk" {
                    info!("Starting transition from first level to second level");
                    transition.start_transition(
                        "levels/level_2.ldtk".to_owned(),
                        "Level_0".to_owned(),
                    );
                }
                return;
            }
        }
    }
}

/// Updates the transition timer and switches levels at the right moment.
fn update_transition(
    time: Res<Time>,
    mut transition: ResMut<TransitionState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut level_config: ResMut<LevelConfig>,
) {
    if !transition.is_transitioning {
        return;
    }

    transition.fade_timer += time.delta_seconds();

    // Switch level at the midpoint when screen is fully black
    let half_duration = transition.fade_duration * 0.5;
    if transition.fade_timer >= half_duration && transition.fade_timer - time.delta_seconds() < half_duration {
        if let (Some(path), Some(name)) = (transition.next_level_path.take(), transition.next_level_name.take()) {
            level_config.project_path = path;
            level_config.start_level = Some(name);
            next_state.set(GameState::Loading);
        }
    }

    // Reset transition when complete
    if transition.fade_timer >= transition.fade_duration {
        transition.reset();
    }
}
