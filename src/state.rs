use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Paused,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    Movement,
    Effects,
}

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
