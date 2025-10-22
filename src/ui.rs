use bevy::prelude::*;

use crate::state::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
            .add_systems(OnExit(GameState::Paused), despawn_pause_menu);
    }
}

#[derive(Component)]
struct PauseMenu;

fn spawn_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            PauseMenu,
            Name::new("PauseMenu"),
            NodeBundle {
                background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Paused\nPress ESC to resume",
                TextStyle {
                    font_size: 36.0,
                    color: Color::srgba(0.9, 0.9, 0.9, 1.0),
                    ..default()
                },
            ));
        });
}

fn despawn_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenu>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
