//! Application entry point: composes the Bevy runtime, core plugins, and window configuration.
//!
//! Even though Rust automatically frees resources once they go out of scope, the Bevy engine
//! keeps long-lived singletons (plugins, resources) alive for the duration of the app. This file
//! wires those pieces together and defers to the `DungeonPlatformerPlugin` defined in `app.rs`.

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

    // The window resource drives swap-chain configuration. We keep the logical resolution at
    // 1280×720 so that LDtk's pixel grid maps 1:1 to Bevy world units. Resizing is enabled, but
    // constraints prevent collapsing the window to unusable sizes. Bevy handles the underlying
    // OS resources, so no manual deallocation is necessary.
    let primary_window = Window {
        title: "Dungeon Platformer".to_string(),
        resolution: WindowResolution::new(1280.0, 720.0),
        resizable: true,
        resize_constraints: WindowResizeConstraints {
            min_width: 640.0,
            min_height: 360.0,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
        },
        canvas: cfg!(all(target_arch = "wasm32", feature = "web"))
            .then(|| "#bevy-canvas".to_owned()),
        ..default()
    };

    // `DefaultPlugins` spins up rendering, input, audio, etc. We override pieces that matter for
    // this project: nearest-neighbor sampling for crisp pixels, and asset settings for desktop vs
    // web. Bevy keeps plugin instances in an internal registry, so we simply compose and hand them
    // to the App builder.
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

    // `App::new()` allocates the ECS world and schedule. Plugins + the clear color describe
    // startup state; once `run()` is called, Bevy drives the main loop until the process exits.
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.04)))
        .add_plugins(default_plugins)
        .add_plugins(DungeonPlatformerPlugin)
        .run();
}
