// input.rs — Input: Touch Drag / Mouse Swipe / Keyboard Gesture Detection
use bevy::prelude::*;
use crate::components::*;
#[cfg(not(feature = "xr"))]
use rand;

#[derive(Resource, Default)]
pub struct DragState {
    pub active: bool,
    pub start_pos: Vec2,
    pub current_pos: Vec2,
}

const SWIPE_THRESHOLD: f32 = 150.0;

pub fn drag_start(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut drag: ResMut<DragState>,
    slide: Res<CurrentSlide>,
) {
    if !slide.ready_for_input || slide.depth_showing {
        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        if let Some(window) = windows.iter().next() {
            if let Some(pos) = window.cursor_position() {
                drag.active = true;
                drag.start_pos = pos;
                drag.current_pos = pos;
            }
        }
    }
}

pub fn drag_move(
    windows: Query<&Window>,
    mut drag: ResMut<DragState>,
) {
    if !drag.active { return; }
    if let Some(window) = windows.iter().next() {
        if let Some(pos) = window.cursor_position() {
            drag.current_pos = pos;
        }
    }
}

pub fn drag_end(
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<DragState>,
    mut writer: MessageWriter<crate::commands::GameCommand>,
    slide: Res<CurrentSlide>,
) {
    if !drag.active { return; }

    if mouse.just_released(MouseButton::Left) {
        let delta = drag.current_pos - drag.start_pos;
        let magnitude = delta.length();

        if magnitude > SWIPE_THRESHOLD && slide.ready_for_input {
            let abs_x = delta.x.abs();
            let abs_y = delta.y.abs();

            let choice = if abs_x > abs_y {
                Some(if delta.x > 0.0 { SwipeChoice::Yes } else { SwipeChoice::No })
            } else if delta.y > 0.0 {
                Some(SwipeChoice::Deeper)
            } else {
                None
            };
            if let Some(choice) = choice {
                info!("Swipe detected: {:?} (magnitude: {:.1})", choice, magnitude);
                writer.write(crate::commands::GameCommand::Swipe(choice));
            }
        }
        drag.active = false;
    }
}

pub fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<crate::commands::GameCommand>,
    slide: Res<CurrentSlide>,
) {
    if !slide.ready_for_input || slide.depth_showing { return; }

    if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
        info!("Keyboard swipe: Yes");
        writer.write(crate::commands::GameCommand::Swipe(SwipeChoice::Yes));
    } else if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
        info!("Keyboard swipe: No");
        writer.write(crate::commands::GameCommand::Swipe(SwipeChoice::No));
    } else if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS)
        || keys.just_pressed(KeyCode::Space) {
        info!("Keyboard swipe: Deeper");
        writer.write(crate::commands::GameCommand::Swipe(SwipeChoice::Deeper));
    }
}

pub fn handle_hand_card_button_interactions(
    mut writer: MessageWriter<crate::commands::GameCommand>,
    hand: Res<Hand>,
    mut buttons: Query<(&Interaction, &crate::hud::HandCardUi), With<Button>>,
) {
    for (interaction, card_ui) in &mut buttons {
        if *interaction == Interaction::Pressed {
            if card_ui.0 < hand.cards.len() {
                writer.write(crate::commands::GameCommand::SelectCard(card_ui.0));
                info!("Selected card index: {}", card_ui.0);
            } else {
                warn!("Hand card UI index {} out of bounds", card_ui.0);
            }
        }
    }
}

pub fn handle_play_card_button_interactions(
    mut writer: MessageWriter<crate::commands::GameCommand>,
    mut buttons: Query<&Interaction, With<crate::hud::PlayCardButton>>,
) {
    for interaction in &mut buttons {
        if *interaction == Interaction::Pressed {
            info!("Play Card clicked!");
            writer.write(crate::commands::GameCommand::PlayCard);
        }
    }
}

pub fn handle_quest_action_button_interactions(
    mut writer: MessageWriter<crate::commands::GameCommand>,
    state: Res<State<GameState>>,
    mut buttons: Query<&Interaction, With<crate::hud::QuestActionButton>>,
) {
    for interaction in &mut buttons {
        if *interaction == Interaction::Pressed {
            info!("Quest Action button clicked");
            if *state.get() == GameState::Playing {
                writer.write(crate::commands::GameCommand::StartQuest("Barnaby".to_string()));
            }
        }
    }
}

pub fn handle_battle_action_button_interactions(
    mut writer: MessageWriter<crate::commands::GameCommand>,
    state: Res<State<GameState>>,
    mut buttons: Query<&Interaction, With<crate::hud::BattleActionButton>>,
) {
    for interaction in &mut buttons {
        if *interaction == Interaction::Pressed {
            info!("Battle Action button clicked");
            if *state.get() == GameState::Playing {
                writer.write(crate::commands::GameCommand::StartBattle);
            }
        }
    }
}

pub fn handle_skip_button_interactions(
    mut writer: MessageWriter<crate::commands::GameCommand>,
    state: Res<State<GameState>>,
    mut buttons: Query<&Interaction, With<crate::hud::SkipButton>>,
) {
    for interaction in &mut buttons {
        if *interaction == Interaction::Pressed {
            info!("Skip clicked!");
            match *state.get() {
                GameState::Playing => { writer.write(crate::commands::GameCommand::StartQuest("Barnaby".to_string())); }
                GameState::Battling => { writer.write(crate::commands::GameCommand::FleeBattle); }
                GameState::Questing => { writer.write(crate::commands::GameCommand::CancelQuest); }
                _ => {}
            }
        }
    }
}

pub fn handle_touch_input(
    mut touch_evr: MessageReader<bevy::input::touch::TouchInput>,
    mut gestures: ResMut<ActiveGestures>,
    mut cards: Query<(Entity, &mut Transform, &mut DraggableCard)>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = match camera.single() {
        Ok(c) => c,
        Err(_) => return,
    };

    for ev in touch_evr.read() {
        match ev.phase {
            bevy::input::touch::TouchPhase::Started => {
                gestures.traces.insert(ev.id, vec![ev.position]);
                // Assign the touch to the first available draggable card (stub hit-test).
                if let Some((entity, _, _)) = cards.iter_mut().find(|(_, _, card)| card.touch_id.is_none()) {
                    if let Ok((_, _, mut card)) = cards.get_mut(entity) {
                        card.touch_id = Some(ev.id);
                    }
                }
            }
            bevy::input::touch::TouchPhase::Moved => {
                if let Some(trace) = gestures.traces.get_mut(&ev.id) {
                    trace.push(ev.position);
                }

                // Convert screen-space touch position to world-space translation on the z=0 plane.
                if let Ok(ray) = camera.viewport_to_world(camera_transform, ev.position) {
                    let t = -ray.origin.z / ray.direction.z;
                    let world_pos = ray.origin + ray.direction * t;
                    for (_, mut transform, card) in cards.iter_mut() {
                        if card.touch_id == Some(ev.id) {
                            transform.translation.x = world_pos.x;
                            transform.translation.y = world_pos.y;
                        }
                    }
                }
            }
            bevy::input::touch::TouchPhase::Ended | bevy::input::touch::TouchPhase::Canceled => {
                gestures.traces.remove(&ev.id);
                for (_, _, mut card) in cards.iter_mut() {
                    if card.touch_id == Some(ev.id) {
                        card.touch_id = None;
                    }
                }
            }
        }
    }
}

#[cfg(not(feature = "xr"))]
pub fn keyboard_quest_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    hand: Res<Hand>,
    session: Option<Res<crate::quest::QuestSession>>,
    mut writer: MessageWriter<crate::commands::GameCommand>,
) {
    if session.is_none() {
        return;
    }

    if keys.just_pressed(KeyCode::Enter) {
        writer.write(crate::commands::GameCommand::CompleteQuest);
        return;
    }

    let pressed_idx = [
        KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3,
        KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6,
        KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9,
    ].iter().position(|&k| keys.just_pressed(k));

    if let Some(idx) = pressed_idx {
        if idx < hand.cards.len() {
            writer.write(crate::commands::GameCommand::FillQuestSlot(idx));
        }
    }
}

#[cfg(not(feature = "xr"))]
pub fn keyboard_battle_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    hand: Res<Hand>,
    session: Option<Res<crate::battle::BattleSession>>,
    mut writer: MessageWriter<crate::commands::GameCommand>,
) {
    if session.is_none() {
        return;
    }

    let pressed_idx = [
        KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3,
        KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6,
        KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9,
    ].iter().position(|&k| keys.just_pressed(k));

    if let Some(idx) = pressed_idx {
        if idx < hand.cards.len() {
            writer.write(crate::commands::GameCommand::PlayBattleCard(idx));
        }
    }
}

#[cfg(not(feature = "xr"))]
pub fn keyboard_debug_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut writer: MessageWriter<crate::commands::GameCommand>,
) {
    // Debug battle shortcuts (only in Playing state)
    if *state.get() == GameState::Playing && keys.just_pressed(KeyCode::KeyB) {
        // Start debug battle with a random word
        let debug_words = ["happy", "sad", "angry", "joyful", "fierce", "calm", "thunder", "shadow"];
        let random_word = debug_words[rand::random::<usize>() % debug_words.len()];
        writer.write(crate::commands::GameCommand::DebugBattle(random_word.to_string()));
    }

    // Face switching (any state)
    if keys.just_pressed(KeyCode::KeyF) {
        let faces = ["fierce", "joyful", "calm", "angry"];
        let random_face = faces[rand::random::<usize>() % faces.len()];
        writer.write(crate::commands::GameCommand::SetFace(random_face.to_string()));
    }

    // VAAM print (any state)
    if keys.just_pressed(KeyCode::KeyV) {
        writer.write(crate::commands::GameCommand::PrintVaam);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_state_defaults_inactive() {
        let drag = DragState::default();
        assert!(!drag.active);
        assert_eq!(drag.start_pos, Vec2::ZERO);
        assert_eq!(drag.current_pos, Vec2::ZERO);
    }

    #[test]
    fn swipe_choice_from_delta_right_is_yes() {
        let delta = Vec2::new(200.0, 10.0);
        let abs_x = delta.x.abs();
        let abs_y = delta.y.abs();
        let choice = if abs_x > abs_y {
            Some(if delta.x > 0.0 { SwipeChoice::Yes } else { SwipeChoice::No })
        } else if delta.y > 0.0 {
            Some(SwipeChoice::Deeper)
        } else {
            None
        };
        assert_eq!(choice, Some(SwipeChoice::Yes));
    }

    #[test]
    fn swipe_choice_from_delta_down_is_deeper() {
        let delta = Vec2::new(10.0, 200.0);
        let abs_x = delta.x.abs();
        let abs_y = delta.y.abs();
        let choice = if abs_x > abs_y {
            Some(if delta.x > 0.0 { SwipeChoice::Yes } else { SwipeChoice::No })
        } else if delta.y > 0.0 {
            Some(SwipeChoice::Deeper)
        } else {
            None
        };
        assert_eq!(choice, Some(SwipeChoice::Deeper));
    }
}

