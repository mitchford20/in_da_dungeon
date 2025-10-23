//! Global game state definitions. States are stored by Bevy in a stack; switching states simply
//! updates an enum value and triggers on-enter/on-exit schedules. No heap allocations occur when
//! toggling states.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

/// High-level state machine for the game loop.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Paused,
}

/// Named system sets to structure the Update schedule.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    Movement,
    Effects,
}

/// Toggles between Playing and Paused when `ESC` is pressed. The `State` resource is read-only
/// snapshot; `NextState` writes the pending transition which Bevy applies at the end of the frame.
pub fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    match state.get() {
        GameState::Playing => next_state.set(GameState::Paused),
        GameState::Paused => next_state.set(GameState::Playing),
        GameState::Loading => {}
    }
}
