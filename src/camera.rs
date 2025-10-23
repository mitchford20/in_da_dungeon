//! Camera follow system. Keeps the main 2D camera centered on the player while respecting
//! level boundaries. All transformations are managed through Bevy's ECS—no raw pointers or
//! manual memory management required.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::level::LevelAssets;
use crate::player::Player;
use crate::state::GameSet;

/// Plugin that registers the camera-follow system. Bevy stores plugins on the heap owned by the
/// app; once the app exits, resources are dropped automatically.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            follow_player_camera
                .after(GameSet::Movement) // run after movement so camera sees latest transform
                .run_if(has_player_and_camera),
        );
    }
}

/// Marker component so the follow system can locate the camera entity without relying on names.
#[derive(Component)]
pub struct FollowCamera;

/// Run condition that only schedules the follow system when both a player and camera exist. This
/// prevents `get_single` panics during level transitions when entities may be missing.
fn has_player_and_camera(
    player_query: Query<Entity, With<Player>>,
    camera_query: Query<Entity, With<FollowCamera>>,
) -> bool {
    !player_query.is_empty() && !camera_query.is_empty()
}

/// Smoothly interpolates the camera transform toward the player's location. The Lerped motion
/// uses an exponential decay constant (`follow_speed`) to avoid overshooting while keeping the
/// player centered. Orthographic scale is accounted for when clamping within level bounds so the
/// camera never shows outside the playable area.
fn follow_player_camera(
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<FollowCamera>>,
    player_query: Query<&Transform, (With<Player>, Without<FollowCamera>)>,
    level_assets: Res<LevelAssets>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let Ok((mut camera_transform, projection)) = camera_query.get_single_mut() else {
        return;
    };

    let target_z = camera_transform.translation.z;
    let mut desired = Vec3::new(
        player_transform.translation.x,
        player_transform.translation.y,
        target_z,
    );

    if let (Some(origin), Some(size)) = (level_assets.level_origin, level_assets.level_size) {
        if let Ok(window) = window_query.get_single() {
            let half_width = window.resolution.width() * 0.5 * projection.scale;
            let half_height = window.resolution.height() * 0.5 * projection.scale;

            let width_world = size.x;
            let height_world = size.y;

            if width_world > half_width * 2.0 {
                let min_x = origin.x + half_width;
                let max_x = origin.x + width_world - half_width;
                desired.x = desired.x.clamp(min_x, max_x);
            }

            if height_world > half_height * 2.0 {
                let min_y = origin.y + half_height;
                let max_y = origin.y + height_world - half_height;
                desired.y = desired.y.clamp(min_y, max_y);
            }
        }
    }

    let follow_speed = 6.0;
    let lerp_t = 1.0 - f32::exp(-follow_speed * time.delta_seconds());
    camera_transform.translation = camera_transform.translation.lerp(desired, lerp_t);
}
