use bevy::prelude::*;

use crate::audio::GameAudioPlugin;
use crate::camera::{CameraPlugin, FollowCamera};
use crate::collision::CollisionPlugin;
use crate::level::LevelPlugin;
use crate::movement::MovementPlugin;
use crate::player::PlayerPlugin;
use crate::state::{toggle_pause, GameSet, GameState};
use crate::ui::UiPlugin;

pub struct DungeonPlatformerPlugin;

impl Plugin for DungeonPlatformerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                LevelPlugin,
                PlayerPlugin,
                GameAudioPlugin,
                CameraPlugin,
                CollisionPlugin,
                MovementPlugin,
                UiPlugin,
            ))
            .configure_sets(
                Update,
                (GameSet::Input, GameSet::Movement, GameSet::Effects)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(Startup, setup_camera)
            .add_systems(Update, toggle_pause);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("MainCamera"),
        Camera2dBundle::default(),
        FollowCamera,
    ));
}
