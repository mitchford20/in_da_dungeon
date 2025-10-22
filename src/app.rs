//! High-level plugin composition.
//!
//! The `DungeonPlatformerPlugin` glues together all domain-specific plugins
//! (levels, player, audio, collisions, etc.) and sets up system ordering.
//! Each subsystem is responsible for its own state; this orchestrator merely
//! registers them with the Bevy application.

use bevy::prelude::*;

use crate::audio::GameAudioPlugin;
use crate::camera::{CameraPlugin, FollowCamera};
use crate::collision::CollisionPlugin;
use crate::level::LevelPlugin;
use crate::movement::MovementPlugin;
use crate::player::PlayerPlugin;
use crate::state::{toggle_pause, GameSet, GameState};
use crate::ui::UiPlugin;

/// Bundles every gameplay-centric plugin into a single unit that can be added
/// to the Bevy `App`. Memory for each plugin is managed by Bevy; once the app
/// shuts down, all resources owned by these plugins are dropped automatically.
pub struct DungeonPlatformerPlugin;

impl Plugin for DungeonPlatformerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>() // Allocates the state machine in the ECS world.
            .add_plugins((
                LevelPlugin,      // Level loading + LDtk asset plumbing.
                PlayerPlugin,     // Player entity spawning logic.
                GameAudioPlugin,  // Audio handle preloading.
                CameraPlugin,     // Camera follow behaviour.
                CollisionPlugin,  // Tile-based collision map.
                MovementPlugin,   // Input + kinematic updates.
                UiPlugin,         // Pause overlay.
            ))
            // Systems inside these sets execute sequentially while the game
            // is in the `Playing` state. `chain()` enforces Input → Movement
            // → Effects ordering so memory writes to components happen in
            // deterministic stages.
            .configure_sets(
                Update,
                (GameSet::Input, GameSet::Movement, GameSet::Effects)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(Startup, setup_camera) // Creates the primary camera entity once.
            .add_systems(Update, toggle_pause); // Hot-swaps GameState based on keyboard input.
    }
}

/// Spawns the initial 2D camera tagged with `FollowCamera` so the follow system
/// can locate it. The Bevy ECS automatically stores this entity in an archetype
/// table; the camera components stay alive until the entity is despawned.
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("MainCamera"),
        Camera2dBundle::default(),
        FollowCamera,
    ));
}
