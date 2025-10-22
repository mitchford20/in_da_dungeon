use bevy::prelude::*;

use crate::state::GameState;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioHandles>()
            .add_systems(OnEnter(GameState::Loading), load_audio_handles);
    }
}

#[derive(Resource, Default)]
pub struct AudioHandles {
    pub jump: Option<Handle<AudioSource>>,
    pub pickup: Option<Handle<AudioSource>>,
    pub ambient: Option<Handle<AudioSource>>,
}

fn load_audio_handles(asset_server: Res<AssetServer>, mut handles: ResMut<AudioHandles>) {
    handles.jump = Some(asset_server.load("audio/jump.ogg"));
    handles.pickup = Some(asset_server.load("audio/pickup.ogg"));
    handles.ambient = Some(asset_server.load("audio/ambient.ogg"));

    info!("Queued audio placeholders. Add actual files under assets/audio/ to enable playback.");
}
