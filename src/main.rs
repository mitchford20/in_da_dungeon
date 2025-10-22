mod app;
mod audio;
mod camera;
mod collision;
mod level;
mod movement;
mod player;
mod state;
mod ui;

#[cfg(all(target_arch = "wasm32", feature = "web"))]
mod wasm;

use app::DungeonPlatformerPlugin;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::render::texture::ImagePlugin;
use bevy::window::{Window, WindowResizeConstraints, WindowResolution};

fn main() {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    wasm::set_panic_hook();

    let primary_window = Window {
        title: "Dungeon Platformer".to_string(),
        resolution: WindowResolution::new(1280.0, 720.0),
        resizable: false,
        resize_constraints: WindowResizeConstraints {
            min_width: 1280.0,
            min_height: 720.0,
            max_width: 1280.0,
            max_height: 720.0,
        },
        canvas: cfg!(all(target_arch = "wasm32", feature = "web"))
            .then(|| "#bevy-canvas".to_owned()),
        ..default()
    };

    let mut default_plugins = DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(primary_window),
            ..default()
        })
        .set(ImagePlugin::default_nearest());

    #[cfg(not(target_arch = "wasm32"))]
    {
        default_plugins = default_plugins.set(AssetPlugin {
            file_path: "assets".to_owned(),
            watch_for_changes_override: Some(true),
            ..default()
        });
    }

    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        default_plugins = default_plugins.set(AssetPlugin {
            file_path: "assets".to_owned(),
            watch_for_changes_override: Some(false),
            ..default()
        });
    }

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.04)))
        .add_plugins(default_plugins)
        .add_plugins(DungeonPlatformerPlugin)
        .run();
}
