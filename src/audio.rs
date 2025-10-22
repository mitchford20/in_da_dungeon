//! Audio preloading utilities. Stashes Bevy `Handle<AudioSource>` references so they are kept alive in memory.
//!
//! Bevy's asset system reference-counts handles; when the last handle is dropped, the underlying
//! audio buffer is released. The `AudioHandles` resource keeps optional handles alive until the
//! user replaces them with real assets.

use bevy::prelude::*;

use crate::state::GameState;

/// Registers the audio loading system and allocates the persistent handle cache.
/// The plugin itself is lightweight—just bookkeeping for asset handles.
pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioHandles>()
            .add_systems(OnEnter(GameState::Loading), load_audio_handles);
    }
}

/// Resource that stores optional handles to game-wide audio clips. Because each `Handle` is just a
/// cloneable pointer into Bevy's asset storage, this struct is cheap to copy and keeps asset memory
/// alive until explicit replacement.
#[derive(Resource, Default)]
pub struct AudioHandles {
    pub jump: Option<Handle<AudioSource>>,
    pub pickup: Option<Handle<AudioSource>>,
    pub ambient: Option<Handle<AudioSource>>,
}

/// Loads placeholder audio files using the global `AssetServer`. The server queues asynchronous
/// asset fetches; once loaded, Bevy caches the decoded audio in memory and the handles in
/// `AudioHandles` reference that cache. Until real files are provided, these act as no-ops.
fn load_audio_handles(asset_server: Res<AssetServer>, mut handles: ResMut<AudioHandles>) {
    handles.jump = Some(asset_server.load("audio/jump.ogg"));
    handles.pickup = Some(asset_server.load("audio/pickup.ogg"));
    handles.ambient = Some(asset_server.load("audio/ambient.ogg"));

    info!("Queued audio placeholders. Add actual files under assets/audio/ to enable playback.");
}
