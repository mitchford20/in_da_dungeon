//! LDtk level orchestration: loads project data, tracks metadata, and aligns the camera.
//!
//! All persistent data is stored in Bevy resources (`LevelConfig`, `LevelAssets`). Rust's ownership
//! system ensures these allocations are freed when the app terminates; during runtime, they are
//! shared immutably or mutably through the ECS borrow rules.

use bevy::asset::LoadState;
use bevy::math::IVec2;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_ldtk::utils::ldtk_pixel_coords_to_translation;
use bevy_ecs_ldtk::LevelIid;

use crate::state::GameState;

/// Registers LDtk asset plumbing and camera synchronisation systems.
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelConfig::default())
            .init_resource::<LevelAssets>()
            .insert_resource(LevelSelection::index(0))
            .insert_resource(LdtkSettings {
                level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                    load_level_neighbors: true,
                },
                set_clear_color: SetClearColor::FromLevelBackground,
                ..default()
            })
            .add_plugins(LdtkPlugin)
            .add_systems(OnEnter(GameState::Loading), spawn_world)
            .add_systems(
                Update,
                monitor_level_loading.run_if(in_state(GameState::Loading)),
            )
            .add_systems(
                PostUpdate,
                (
                    cache_level_transform,
                    sync_level_spatial.after(cache_level_transform),
                ),
            );
    }
}

/// Runtime-tweakable configuration describing which LDtk project + level to load, how to shift it
/// in world space, and how large the tiles/camera zoom are. Cloned when other systems need read-only
/// access; cloning is cheap because it only copies primitive values.
#[derive(Resource, Clone)]
pub struct LevelConfig {
    pub project_path: String,
    pub start_level: Option<String>,
    pub frame_shift: Vec2,
    pub tile_size: f32,
    pub camera_zoom: f32,
}

impl Default for LevelConfig {
    fn default() -> Self {
        Self {
            project_path: "levels/test_map_1_newres.ldtk".to_owned(),
            start_level: Some("Level_0".to_owned()),
            frame_shift: Vec2::ZERO,
            tile_size: 16.0,
            camera_zoom: 0.5,
        }
    }
}

/// Mirror of the currently loaded level's metadata. Optional fields become `Some` once assets are
/// available. Other systems (camera/collision) read this without owning the LDtk structures.
#[derive(Resource, Default)]
pub struct LevelAssets {
    pub project: Option<Handle<LdtkProject>>,
    pub project_path: Option<String>,
    pub level_identifier: Option<String>,
    pub level_iid: Option<String>,
    pub level_origin: Option<Vec2>,
    pub level_size: Option<Vec2>,
    pub level_center: Option<Vec2>,
}

/// Marker on the LDtk world entity so we can despawn it before loading another level, avoiding
/// dangling entity graphs.
#[derive(Component)]
pub struct LevelRoot;

fn spawn_world(
    mut commands: Commands,
    world: Query<Entity, With<LevelRoot>>,
    asset_server: Res<AssetServer>,
    config: Res<LevelConfig>,
    mut level_assets: ResMut<LevelAssets>,
    mut selection: ResMut<LevelSelection>,
) {
    // Despawn any previously spawned LDtk world so we don't leak entities or memory. Bevy handles
    // recursive child destruction when `despawn_recursive` is used.
    for entity in &world {
        commands.entity(entity).despawn_recursive();
    }

    let project_handle: Handle<LdtkProject> = asset_server.load(config.project_path.clone());
    level_assets.project = Some(project_handle.clone());
    level_assets.project_path = Some(config.project_path.clone());

    *selection = config
        .start_level
        .as_ref()
        .map(|label| LevelSelection::Identifier(label.clone()))
        .unwrap_or_else(|| LevelSelection::index(0));

    // Spawn a new LDtk world bundle. The bundle contains the world entity and ensures LDtk systems
    // load the associated levels. `frame_shift` lets us offset the entire map if desired.
    commands.spawn((
        LevelRoot,
        Name::new("LevelRoot"),
        LdtkWorldBundle {
            ldtk_handle: project_handle,
            transform: Transform::from_translation(config.frame_shift.extend(0.0)),
            ..default()
        },
    ));
}

fn monitor_level_loading(
    asset_server: Res<AssetServer>,
    mut level_assets: ResMut<LevelAssets>,
    projects: Res<Assets<LdtkProject>>,
    config: Res<LevelConfig>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(project_handle) = level_assets.project.as_ref() else {
        return;
    };

    match asset_server.get_load_state(project_handle.id()) {
        Some(LoadState::Loaded) => {
            // Once the project JSON + tilesets are loaded we can read metadata to cache sizes.
            if let Some(project) = projects.get(project_handle) {
                let level_data = config
                    .start_level
                    .as_ref()
                    .and_then(|identifier| {
                        project
                            .json_data()
                            .levels
                            .iter()
                            .find(|level| &level.identifier == identifier)
                    })
                    .or_else(|| project.json_data().levels.first());

                if let Some(level) = level_data {
                    let origin = ldtk_pixel_coords_to_translation(
                        IVec2::new(level.world_x, level.world_y + level.px_hei),
                        0,
                    );
                    let size = Vec2::new(level.px_wid as f32, level.px_hei as f32);
                    let center = Vec2::new(
                        level.world_x as f32 + size.x * 0.5,
                        -(level.world_y as f32) - size.y * 0.5,
                    );

                    level_assets.level_identifier = Some(level.identifier.clone());
                    level_assets.level_iid = Some(level.iid.clone());
                    level_assets.level_origin = Some(origin);
                    level_assets.level_size = Some(size);
                    level_assets.level_center = Some(center);
                }
            }

            next_state.set(GameState::Playing);
        }
        Some(LoadState::Failed(_)) => {
            let path = level_assets.project_path.as_deref().unwrap_or("<unknown>");
            warn!(
                "Unable to load LDtk project at '{}'; continuing with placeholder state.",
                path
            );
            next_state.set(GameState::Playing);
        }
        _ => {}
    }
}

fn cache_level_transform(
    mut level_assets: ResMut<LevelAssets>,
    level_query: Query<(&GlobalTransform, &LevelIid), Added<LevelIid>>,
) {
    // When LDtk instantiates a level entity, capture its world transform so other systems know
    // where the level origin sits in Bevy coordinates.
    for (transform, iid) in &level_query {
        let matches_current_level = level_assets
            .level_iid
            .as_ref()
            .map(|target| target == iid.get())
            .unwrap_or(true);

        if matches_current_level {
            let origin = transform.translation().truncate();
            level_assets.level_origin = Some(origin);

            if let Some(size) = level_assets.level_size {
                level_assets.level_center = Some(origin + size * 0.5);
            }
        }
    }
}

pub fn sync_level_spatial(
    level_assets: Res<LevelAssets>,
    config: Res<LevelConfig>,
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2d>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if !level_assets.is_changed() {
        return;
    }

    let (Some(center), Some(size)) = (level_assets.level_center, level_assets.level_size) else {
        return;
    };

    let Ok((mut camera_transform, mut projection)) = camera_query.get_single_mut() else {
        return;
    };

    if let Ok(window) = windows.get_single() {
        let window_size = window.resolution.size();
        if window_size.x > 0.0 && window_size.y > 0.0 {
            let width_ratio = size.x / window_size.x;
            let height_ratio = size.y / window_size.y;
            let base_scale = width_ratio.max(height_ratio).max(0.0001);
            projection.scale = (base_scale * config.camera_zoom).max(0.0001);
        }
    }

    // Move the camera to the cached center. Z is left untouched so other systems (e.g., follow
    // smoothing) can maintain depth. Transform writes are staged by Bevy to avoid data races.
    camera_transform.translation.x = center.x;
    camera_transform.translation.y = center.y;
}
