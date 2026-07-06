// hand_tracking.rs — Hand joint tracking and ASL fingerspelling recognition
#![allow(dead_code)]
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct PinchEvent {
    pub position: Vec3,
    pub hand: u8,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct PinchEvents {
    pub events: Vec<PinchEvent>,
}

#[derive(Component)]
pub struct HandJointMarker {
    pub hand: u8,  // 0 = Left, 1 = Right
    pub joint: u8, // Joint index (0-25)
}

#[derive(Resource, Default, Debug, Clone)]
pub struct HandTrackingState {
    pub left_index_tip: Option<Vec3>,
    pub right_index_tip: Option<Vec3>,
    pub left_thumb_tip: Option<Vec3>,
    pub right_thumb_tip: Option<Vec3>,
    pub left_wrist: Option<Vec3>,
    pub right_wrist: Option<Vec3>,
    pub detected_letter: Option<char>,
    pub left_pinching: bool,
    pub right_pinching: bool,
    pub last_wrist_pos: Option<Vec3>,
    pub gesture_intensity: f32,
    pub pinch_sequence: Vec<Vec3>,
}

pub fn update_hand_tracking(
    mut state: ResMut<HandTrackingState>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &HandJointMarker)>,
    mut pinch_events: ResMut<PinchEvents>,
    mut time_scale: ResMut<crate::components::TimeScale>,
) {
    pinch_events.events.clear();
    // Simulated desktop fallback wiggling joints on sine waves
    let t = time.elapsed_secs();
    
    // Left hand index and thumb positions
    let left_wrist_pos = Vec3::new(-1.0, 1.0, -1.0);
    let left_index_pos = left_wrist_pos + Vec3::new(0.0, 0.4 + (t * 3.0).sin() * 0.05, 0.0);
    let left_thumb_pos = left_wrist_pos + Vec3::new(0.02, 0.38 + (t * 3.0).sin() * 0.04, 0.0);
    
    state.left_wrist = Some(left_wrist_pos);
    state.left_index_tip = Some(left_index_pos);
    state.left_thumb_tip = Some(left_thumb_pos);

    // Calculate left hand pinch
    let previously_pinching = state.left_pinching;
    state.left_pinching = left_index_pos.distance(left_thumb_pos) < 0.05;

    // Gesture Intensity Calculation (Delta between wrist positions)
    if let Some(last_wrist) = state.last_wrist_pos {
        let delta = left_wrist_pos.distance(last_wrist);
        state.gesture_intensity = state.gesture_intensity * 0.9 + delta * 10.0 * 0.1;
    }
    state.last_wrist_pos = Some(left_wrist_pos);

    if state.left_pinching && !previously_pinching {
        let pinch_pos = left_index_pos.lerp(left_thumb_pos, 0.5);
        pinch_events.events.push(PinchEvent {
            position: pinch_pos,
            hand: 0,
        });
        state.pinch_sequence.push(pinch_pos);
        if state.pinch_sequence.len() > 3 {
            state.pinch_sequence.remove(0); // Keep last 3 for Subject -> Verb -> Object grammar golem assembly
        }
        info!("Left hand pinch detected at {:?} (Sequence length: {})", pinch_pos, state.pinch_sequence.len());
    }

    for (mut transform, marker) in &mut query {
        let wiggle = (t * 2.0).sin() * 0.05;
        if marker.hand == 0 {
            transform.translation = left_wrist_pos + Vec3::new(marker.joint as f32 * 0.02, wiggle, 0.0);
        } else {
            transform.translation = Vec3::new(1.0, 1.0, -1.0) + Vec3::new(marker.joint as f32 * 0.02, wiggle, 0.0);
        }
    }

    // Run ASL recognition heuristics
    recognize_asl_letter(&mut state);

    // Z-Space Time Dilation
    // If the player pushes their hands forward (Z < -0.8), enter Focus State
    let is_in_focus_zone = left_wrist_pos.z < -0.8;
    let target_scale = if is_in_focus_zone { 0.1 } else { 1.0 };
    
    // Lerp time_scale
    time_scale.0 = time_scale.0 + (target_scale - time_scale.0) * time.delta_secs() * 5.0;
}

fn recognize_asl_letter(state: &mut ResMut<HandTrackingState>) {
    let wrist = match state.left_wrist {
        Some(w) => w,
        None => return,
    };
    let index = match state.left_index_tip {
        Some(idx) => idx,
        None => return,
    };

    let dist = index.distance(wrist);
    
    // Simple heuristic: if distance is high, sign 'L', otherwise sign 'A'
    if dist > 0.42 {
        state.detected_letter = Some('L');
    } else {
        state.detected_letter = Some('A');
    }
}

pub fn grammar_fusion_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &crate::components::Summon)>,
) {
    let mut golems = Vec::new();
    let mut slimes = Vec::new();
    let mut robots = Vec::new();

    for (entity, transform, summon) in query.iter() {
        match summon.0 {
            crate::components::SummonClass::GrammarGolem => golems.push((entity, transform.translation)),
            crate::components::SummonClass::SemanticSlime => slimes.push((entity, transform.translation)),
            crate::components::SummonClass::RhetoricRobot => robots.push((entity, transform.translation)),
        }
    }

    // Check for overlap
    for (g_ent, g_pos) in &golems {
        for (s_ent, s_pos) in &slimes {
            for (r_ent, r_pos) in &robots {
                let dist_gs = g_pos.distance(*s_pos);
                let dist_sr = s_pos.distance(*r_pos);
                let dist_rg = r_pos.distance(*g_pos);

                if dist_gs < 1.0 && dist_sr < 1.0 && dist_rg < 1.0 {
                    // Valid overlap (Gestalt Fusion Triggered)
                    info!("GRAMMAR FUSION INITIATED! Slime (Noun) + Robot (Adverb) + Golem (Verb). Despawning individual pets...");
                    commands.entity(*g_ent).despawn();
                    commands.entity(*s_ent).despawn();
                    commands.entity(*r_ent).despawn();
                    
                    // Spawn Gestalt Mega-Entity
                    info!("Spawning Gestalt Mega-Entity (AoE Barrier)!");
                    // (In a real implementation, we would spawn the barrier mesh here)
                    return; // Prevent multiple fusions in one frame causing despawn panics
                }
            }
        }
    }
}
