// music.rs — Adaptive, cross-fading music driven by game state.
//
// The world should sound like a place. Each game state maps to a generated
// looping stem, and the music crossfades between them while respecting
// `GameSettings.music_volume`.

use bevy::prelude::*;
use bevy::audio::{AudioSink, AudioSinkPlayback, Volume};
use crate::asset_catalog as catalog;
use crate::components::GameState;
use crate::settings::GameSettings;

pub struct MusicPlugin;

/// A looping music stem. The enum value is also the marker component on the
/// audio entity.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicTrack {
    Menu,
    World,
    Battle,
}

/// Tracks which music stem should currently be audible.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct MusicState {
    pub target: Option<MusicTrack>,
}

impl MusicTrack {
    fn asset_path(&self) -> &'static str {
        match self {
            MusicTrack::Menu => catalog::MUSIC_MENU,
            MusicTrack::World => catalog::MUSIC_WORLD,
            MusicTrack::Battle => catalog::MUSIC_BATTLE,
        }
    }
}

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicState>()
           .add_systems(Update, update_music);
    }
}

const FADE_IN_SPEED: f32 = 1.2;   // volume per second
const FADE_OUT_SPEED: f32 = 2.5;  // volume per second
const FADE_OUT_CUTOFF: f32 = 0.02;

fn track_for_state(state: GameState) -> MusicTrack {
    match state {
        GameState::MainMenu
        | GameState::Settings
        | GameState::Difficulty
        | GameState::Paywall
        | GameState::Loading => MusicTrack::Menu,

        GameState::Playing
        | GameState::Exploring
        | GameState::Collecting
        | GameState::Constructing
        | GameState::PetCollection
        | GameState::Questing
        | GameState::Reviewing
        | GameState::RevealingPet => MusicTrack::World,

        GameState::Battling => MusicTrack::Battle,
    }
}

#[allow(clippy::too_many_arguments)]
fn update_music(
    mut commands: Commands,
    settings: Res<GameSettings>,
    state: Res<State<GameState>>,
    mut music_state: ResMut<MusicState>,
    asset_server: Res<AssetServer>,
    mut sinks: Query<(Entity, &MusicTrack, &mut AudioSink)>,
    time: Res<Time>,
) {
    let desired = track_for_state(state.get().clone());

    if music_state.target != Some(desired) {
        music_state.target = Some(desired);

        let already_playing = sinks.iter().any(|(_, track, _)| *track == desired);
        if !already_playing {
            let handle = asset_server.load::<AudioSource>(desired.asset_path());
            commands.spawn((
                AudioPlayer::<AudioSource>(handle),
                PlaybackSettings::LOOP.with_volume(Volume::SILENT),
                desired,
            ));
        }
    }

    for (entity, track, mut sink) in sinks.iter_mut() {
        let target_volume = if *track == desired {
            settings.music_volume
        } else {
            0.0
        };

        let current = sink.volume().to_linear();
        let rate = if target_volume > current { FADE_IN_SPEED } else { FADE_OUT_SPEED };
        let dt = time.delta_secs();
        let step = rate * dt;

        let next = if (target_volume - current).abs() <= step {
            target_volume
        } else if target_volume > current {
            current + step
        } else {
            current - step
        };

        sink.set_volume(Volume::Linear(next.clamp(0.0, 1.0)));

        if *track != desired && next <= FADE_OUT_CUTOFF {
            commands.entity(entity).despawn();
        }
    }
}
