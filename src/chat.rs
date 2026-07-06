// chat.rs — 3D Spatial Chat Box & Pet FACES Dialogue/Taming System
use bevy::prelude::*;
use faces_protocol::{Focus, Action, Aura};
use crate::components::*;
use crate::spatial_ui::*;
#[cfg(all(feature = "tts", not(target_arch = "wasm32")))]
use std::io::Write;

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatLog>()
           .add_systems(OnEnter(GameState::Playing), setup_chat_panel)
           .add_systems(Update, (
               update_chat_display,
               handle_pet_taming_inputs,
               observe_pet_faces_changes,
           ).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender: String,
    pub text: String,
    pub timestamp: f32,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ChatLog {
    pub messages: Vec<ChatMessage>,
}

impl ChatLog {
    pub fn add_message(&mut self, sender: &str, text: &str, timestamp: f32) {
        self.messages.push(ChatMessage {
            sender: sender.to_string(),
            text: text.to_string(),
            timestamp,
        });
        if self.messages.len() > 5 {
            self.messages.remove(0);
        }
    }
}

#[derive(Component)]
pub struct ChatPanel;

#[derive(Component)]
pub struct ChatLineText {
    pub index: usize,
}

fn setup_chat_panel(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    panel_query: Query<Entity, With<ChatPanel>>,
) {
    // Prevent duplicate panels
    if !panel_query.is_empty() {
        return;
    }

    info!("Spawning 3D Spatial Chat Panel...");
    // Spawn panel on the left side of the stage
    let panel_id = spawn_spatial_panel(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-3.5, 2.0, -2.5),
        "Pet Communications",
    );

    // Add lines of text on the panel
    commands.entity(panel_id).insert(ChatPanel).with_children(|parent| {
        for i in 0..5 {
            parent.spawn((
                Text2d::new(""),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.3, 0.9, 0.9)),
                Transform::from_xyz(-1.3, 0.4 - (i as f32 * 0.22), 0.03),
                ChatLineText { index: i },
            ));
        }
    });
}

fn update_chat_display(
    chat_log: Res<ChatLog>,
    mut text_query: Query<(&mut Text2d, &ChatLineText)>,
) {
    if chat_log.is_changed() {
        for (mut text, line) in &mut text_query {
            if let Some(msg) = chat_log.messages.get(line.index) {
                text.0 = format!("[{:.1}s] {}: {}", msg.timestamp, msg.sender, msg.text);
            } else {
                text.0 = String::new();
            }
        }
    }
}

fn handle_pet_taming_inputs(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut pet_query: Query<(&PetAvatar, &mut PetFacesState, &Element)>,
    mut chat_log: ResMut<ChatLog>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let t = time.elapsed_secs();
    for (avatar, mut faces, element) in &mut pet_query {
        if keys.just_pressed(KeyCode::KeyP) {
            // Pet the avatar: shift focus to Happy/Open
            faces.0.focus = Focus::Happy;
            faces.0.action = Action::Playful;
            let dialogue = get_pet_dialogue(&avatar.word, faces.0.focus, faces.0.action, *element);
            chat_log.add_message(&avatar.word, &dialogue, t);
            commands.spawn(AudioPlayer::<AudioSource>(asset_server.load("sounds/pet.ogg")));
            speak_dialogue(dialogue, "af_bella".to_string(), &mut commands, &asset_server);
            info!("Petted {}", avatar.word);
        } else if keys.just_pressed(KeyCode::KeyF) {
            // Feed the avatar: shift action to Assertive
            faces.0.focus = Focus::Intense;
            faces.0.action = Action::Assertive;
            let dialogue = get_pet_dialogue(&avatar.word, faces.0.focus, faces.0.action, *element);
            chat_log.add_message(&avatar.word, &dialogue, t);
            commands.spawn(AudioPlayer::<AudioSource>(asset_server.load("sounds/feed.ogg")));
            speak_dialogue(dialogue, "af_bella".to_string(), &mut commands, &asset_server);
            info!("Fed {}", avatar.word);
        } else if keys.just_pressed(KeyCode::KeyT) {
            // Attune the avatar: toggle aura index
            let current_idx = faces.0.aura.index();
            let next_idx = current_idx.wrapping_add(32);
            faces.0.aura = Aura::from_index(next_idx);
            let dialogue = format!("*Aura color shifted to index {}!*", next_idx);
            chat_log.add_message("System", &dialogue, t);
            commands.spawn(AudioPlayer::<AudioSource>(asset_server.load("sounds/attune.ogg")));
            info!("Attuned {}", avatar.word);
        }
    }
}

fn observe_pet_faces_changes(
    time: Res<Time>,
    mut chat_log: ResMut<ChatLog>,
    changed_pets: Query<(&PetAvatar, &PetFacesState, &Element), Changed<PetFacesState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let t = time.elapsed_secs();
    for (avatar, faces, element) in &changed_pets {
        let dialogue = get_pet_dialogue(&avatar.word, faces.0.focus, faces.0.action, *element);
        chat_log.add_message(&avatar.word, &dialogue, t);
        speak_dialogue(dialogue, "af_bella".to_string(), &mut commands, &asset_server);
    }
}

pub fn get_pet_dialogue(
    word: &str,
    focus: Focus,
    action: Action,
    element: Element,
) -> String {
    let description = match (focus, action) {
        (Focus::Happy, Action::Playful) => "vibrates bounce-bounce-bouncy! 'Let's spell more!'",
        (Focus::Intense, Action::Assertive) => "pulses with raw grammar sparks! 'Synonyms ready!'",
        (Focus::Tired, Action::Hesitant) => "curls up and sighs. 'Need some letters...'",
        (Focus::Distant, Action::Thoughtful) => "looks at the sky rings. 'Thinking about adjectives...'",
        (Focus::Open, Action::Assertive) => "stands firm. 'Prepared for the next quest.'",
        _ => "looks at you with glowing curiosity.",
    };
    format!("[{:?}] {} {}", element, word, description)
}

pub fn speak_dialogue(text: String, voice: String, commands: &mut Commands, asset_server: &Res<AssetServer>) {
    #[cfg(all(feature = "tts", not(target_arch = "wasm32")))]
    std::thread::spawn(move || {
        // Send POST to Kokoro TTS sidecar
        // Path: http://localhost:8200/v1/audio/speech (OpenAI TTS compatible API)
        let client = reqwest::blocking::Client::new();
        let payload = serde_json::json!({
            "model": "kokoro",
            "input": text,
            "voice": voice,
            "response_format": "mp3"
        });
        
        match client.post("http://localhost:8200/v1/audio/speech")
            .json(&payload)
            .send() {
            Ok(resp) => {
                if resp.status().is_success() {
                    if let Ok(bytes) = resp.bytes() {
                        // Write to a temporary file, e.g. assets/sounds/tts_output.mp3
                        // So that Bevy's AssetServer can load it and play it!
                        if let Ok(mut file) = std::fs::File::create("assets/sounds/tts_output.mp3") {
                            let _ = file.write_all(&bytes);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to contact Kokoro TTS sidecar: {}", e);
            }
        }
    });

    #[cfg(any(not(feature = "tts"), target_arch = "wasm32"))]
    {
        let _ = text;
        let _ = voice;
        commands.spawn(AudioPlayer::<AudioSource>(asset_server.load("sounds/blip.ogg")));
    }
}

pub fn generate_rhetoric_argument(word: &str, target: &str, is_synonym: bool) -> String {
    if is_synonym {
        format!("By the undeniable logic of semantics, '{}' and '{}' share the same essence! Your defense is structurally invalid!", word, target)
    } else {
        format!("I present a paradox! '{}' is the complete antithesis of '{}'! Prepare your definitions to be destabilized!", word, target)
    }
}

pub fn trigger_social_combat(
    word: &str,
    target: &str,
    is_synonym: bool,
    time_elapsed: f32,
    chat_log: &mut ChatLog,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let argument = generate_rhetoric_argument(word, target, is_synonym);
    chat_log.add_message(&format!("RhetoricRobot[{}]", word), &argument, time_elapsed);
    speak_dialogue(argument, "af_bella".to_string(), commands, asset_server);
}

