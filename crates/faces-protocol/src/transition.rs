// ═══════════════════════════════════════════════════════════════════════════════
// FACES PROTOCOL — faces-protocol
// FILE:        src/transition.rs
// PURPOSE:     Contrastive Transition Vector — emotional velocity & interpolation
// ═══════════════════════════════════════════════════════════════════════════════
//
// THE CONTRASTIVE TRANSITION VECTOR
//
// Static geometric frames fail to capture the kinetic momentum of human
// conversation. Effective human-machine alignment requires visualization
// of the emotional trajectory — not just the current state, but the
// *velocity* of intent. By calculating the shift between subsequent
// FACES states, the system represents how the agent's internal state
// is evolving in response to the dialogue.
//
// The Transition Function (Delta State):
//
//   T = S_t ⊖ S_{t-1} = [Δ Aura, Δ Container, Δ Focus, Δ Action]
//
// This delta represents the specific magnitude of change across the
// 4-byte payload, providing a quantitative metric for emotional
// volatility or stabilization.
//
// APPLICATIONS
//
//   1. Emotional Volatility Detection — Large transition vectors
//      indicate rapid emotional shifts (potential instability,
//      excitement, or crisis). Small vectors indicate stability.
//
//   2. Toxic Escalation Prediction — A sequence of increasing
//      transition magnitudes may predict escalation in multi-user
//      environments, enabling proactive intervention.
//
//   3. Smooth Morphing — Interpolation between discrete coordinate
//      states allows gradual "hardening" of the Container from fluid
//      '{}' to rigid '[]' or narrowing of Focus from 'oo' to '><'.
//      These smooth transitions visualize the psychological process
//      of becoming defensive or concentrated.
//
//   4. Temporal Decay — When interaction ceases, the expression
//      smoothly interpolates back to the baseline neutral state
//      over a configurable time window (default: 5 seconds).
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::action::Action;
use crate::aura::Aura;
use crate::container::Container;
use crate::focus::Focus;
use crate::protocol::FacesState;

// ── TransitionVector ─────────────────────────────────────────────────────────

/// The Contrastive Transition Vector — the delta between two FACES states.
///
/// Represents the emotional "velocity" between consecutive states.
/// Each component is a signed integer indicating the direction and
/// magnitude of change:
///
/// - `delta_aura`: Change in color index (-255 to +255)
/// - `delta_container`: Change in container shape (-4 to +4)
/// - `delta_focus`: Change in focus shape (-5 to +5)
/// - `delta_action`: Change in action shape (-4 to +4)
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionVector {
    /// Change in Aura (Byte 0) — signed delta of color index.
    pub delta_aura: i16,

    /// Change in Container (Byte 1) — signed delta of shape index.
    pub delta_container: i8,

    /// Change in Focus (Byte 2) — signed delta of shape index.
    pub delta_focus: i8,

    /// Change in Action (Byte 3) — signed delta of shape index.
    pub delta_action: i8,
}

impl TransitionVector {
    /// Calculate the transition vector between two FACES states.
    ///
    /// T = S_t ⊖ S_{t-1}
    ///
    /// # Arguments
    ///
    /// * `current` — The current state (S_t)
    /// * `previous` — The previous state (S_{t-1})
    ///
    /// # Returns
    ///
    /// A `TransitionVector` with the signed delta for each component.
    pub fn between(previous: &FacesState, current: &FacesState) -> Self {
        Self {
            delta_aura: current.aura.index() as i16 - previous.aura.index() as i16,
            delta_container: current.container as i8 - previous.container as i8,
            delta_focus: current.focus as i8 - previous.focus as i8,
            delta_action: current.action as i8 - previous.action as i8,
        }
    }

    /// Calculate the magnitude (L1 norm) of the transition vector.
    ///
    /// This is the sum of absolute values of all deltas. A magnitude
    /// of 0 means no change. Higher magnitudes indicate larger
    /// emotional shifts.
    ///
    /// # Interpretation
    ///
    /// - 0: No change (stable)
    /// - 1-3: Minor shift (normal conversation flow)
    /// - 4-10: Moderate shift (topic change, new information)
    /// - 11+: Major shift (emotional volatility, crisis, breakthrough)
    pub fn magnitude(&self) -> u16 {
        self.delta_aura.unsigned_abs()
            + self.delta_container.unsigned_abs() as u16
            + self.delta_focus.unsigned_abs() as u16
            + self.delta_action.unsigned_abs() as u16
    }

    /// Returns true if this transition represents a stable state
    /// (magnitude == 0, no change).
    pub fn is_stable(&self) -> bool {
        self.magnitude() == 0
    }

    /// Returns true if this transition represents high volatility
    /// (magnitude >= 11).
    pub fn is_volatile(&self) -> bool {
        self.magnitude() >= 11
    }

    /// Get a human-readable description of the transition.
    pub fn describe(&self) -> String {
        if self.is_stable() {
            return "stable (no change)".to_string();
        }

        let mag = self.magnitude();
        let volatility = if mag <= 3 {
            "minor"
        } else if mag <= 10 {
            "moderate"
        } else {
            "major"
        };

        format!(
            "{} shift (magnitude {}): Δaura={}, Δcontainer={}, Δfocus={}, Δaction={}",
            volatility,
            mag,
            self.delta_aura,
            self.delta_container,
            self.delta_focus,
            self.delta_action,
        )
    }

    /// Calculate the Pythagorean harmonic distance of this transition.
    ///
    /// Unlike `magnitude()` which uses L1 (Manhattan) distance and treats
    /// all step sizes linearly, `harmonic_distance()` weights each
    /// dimension's step by the **ratio complexity** of the corresponding
    /// musical interval.
    ///
    /// Key insight from psychoacoustics: a 1-step shift (minor 2nd, 16:15)
    /// is **more dissonant** than a 2-step shift (major 3rd, 5:4). L1
    /// distance gets this backwards — it treats 1-step as "smaller" and
    /// therefore "smoother." Harmonic distance corrects this.
    ///
    /// Grounded in:
    /// - Pallesen et al. 2010 (fMRI: neural correlates of ratio rules)
    /// - McDermott et al. 2010 (harmonicity predicts consonance)
    /// - Cousineau et al. 2014 (brainstem phase-locking to simple ratios)
    ///
    /// See FACES_PYTHAGOREAN_RESEARCH.md for full citations and rationale.
    ///
    /// # Returns
    ///
    /// A float where:
    /// - 0.0 = no change (unison)
    /// - Lower values = consonant (smooth) transitions
    /// - Higher values = dissonant (jarring) transitions
    ///
    /// The value is weighted by information entropy per dimension
    /// (log2 of the number of values), so dimensions with more
    /// possible values contribute more to the distance.
    pub fn harmonic_distance(&self) -> f32 {
        // Container: 5 values, circular step
        let c_step = self.delta_container.rem_euclid(Container::COUNT as i8) as u8;
        let c = ratio_complexity(c_step, Container::COUNT as u8);

        // Focus: 6 values, circular step
        let f_step = self.delta_focus.rem_euclid(Focus::COUNT as i8) as u8;
        let f = ratio_complexity(f_step, Focus::COUNT as u8);

        // Action: 5 values, circular step
        let a_step = self.delta_action.rem_euclid(Action::COUNT as i8) as u8;
        let a = ratio_complexity(a_step, Action::COUNT as u8);

        // Aura: continuous 0-255, use consonance delta
        // The Aura distance is the absolute change in consonance level,
        // scaled by the information weight of the Aura dimension.
        let aura_dist = self.delta_aura.unsigned_abs() as f32 / 255.0;

        // Information-entropy weights: log2(N) per dimension
        // Container: log2(5) ≈ 2.32
        // Focus: log2(6) ≈ 2.58
        // Action: log2(5) ≈ 2.32
        // Aura: log2(10 named) ≈ 3.32 (we use 10 named, not 256)
        c * 2.32 + f * 2.58 + a * 2.32 + aura_dist * 3.32
    }
}

// ── Pythagorean Ratio Complexity ─────────────────────────────────────────────

/// Map a circular step distance to a Pythagorean ratio complexity score.
///
/// The FACES dimensions (Container=5, Focus=6, Action=5) map to musical
/// scales with Pythagorean origins:
/// - 5-value enums → pentatonic scale (circle of fifths, ratio 3:2 iterated)
/// - 6-value enums → whole-tone scale (ambiguous, atmospheric)
///
/// Each step distance corresponds to a musical interval, and each interval
/// has a ratio whose complexity can be measured. Simple ratios (unison 1:1,
/// octave 2:1, fifth 3:2) are consonant. Complex ratios (minor 2nd 16:15,
/// tritone sqrt(2):1) are dissonant.
///
/// # The Mapping
///
/// | Step | Interval | Ratio | Complexity |
/// |------|----------|-------|------------|
/// | 0 | unison | 1:1 | 0.00 |
/// | 1 | minor 2nd | 16:15 | 0.83 |
/// | 2 | major 3rd | 5:4 | 0.43 |
/// | 3 | perfect 4th | 4:3 | 0.33 |
/// | 4 | perfect 5th | 3:2 | 0.50 |
/// | 5 | octave | 2:1 | 0.50 |
///
/// # Key Insight
///
/// Step 1 (minor 2nd) has **higher** complexity than step 2 (major 3rd)
/// or step 3 (perfect 4th). This means a small shift can be more jarring
/// than a large shift — which L1 distance cannot capture.
///
/// # Arguments
///
/// * `step` - The circular distance between two enum values (already modded)
/// * `max` - The number of values in the enum (5 or 6)
///
/// # Returns
///
/// A float in [0, 1] where 0 = no change (unison) and higher = more dissonant.
pub fn ratio_complexity(step: u8, max: u8) -> f32 {
    let s = step % max;
    match s {
        0 => 0.00, // unison — no change
        1 => 0.83, // minor 2nd (16:15) — most dissonant small step
        2 => 0.43, // major 3rd (5:4) — consonant
        3 => 0.33, // perfect 4th (4:3) — very consonant
        4 => 0.50, // perfect 5th (3:2) — consonant (or octave for 5-value)
        5 => 0.50, // octave (2:1) — consonant (for 6-value enums)
        _ => 1.00, // fallback — should not occur for max <= 6
    }
}

// ── Interpolation ────────────────────────────────────────────────────────────

/// Linearly interpolate between two FACES states.
///
/// Produces a smooth transition between discrete coordinate states,
/// allowing gradual "hardening" of the Container from fluid '{}' to
/// rigid '[]' or narrowing of Focus from 'oo' to '><'.
///
/// # Arguments
///
/// * `from` — The starting state
/// * `to` — The target state
/// * `t` — Interpolation factor (0.0 = `from`, 1.0 = `to`)
///
/// # Returns
///
/// A new `FacesState` interpolated between `from` and `to`.
///
/// # Note
///
/// The Aura byte interpolates linearly across the full 0-255 range.
/// Container, Focus, and Action use modular interpolation — they
/// take the shortest path around their circular value space.
pub fn lerp(from: &FacesState, to: &FacesState, t: f32) -> FacesState {
    let aura = lerp_u8(from.aura.index(), to.aura.index(), t);
    let container = lerp_circular(from.container as u8, to.container as u8, t, Container::COUNT as u8);
    let focus = lerp_circular(from.focus as u8, to.focus as u8, t, Focus::COUNT as u8);
    let action = lerp_circular(from.action as u8, to.action as u8, t, Action::COUNT as u8);

    FacesState::new(
        Aura::from_index(aura),
        Container::from_byte(container),
        Focus::from_byte(focus),
        Action::from_byte(action),
    )
}

/// Linear interpolation for u8 (Aura byte).
fn lerp_u8(from: u8, to: u8, t: f32) -> u8 {
    let from = from as f32;
    let to = to as f32;
    let result = from + (to - from) * t.clamp(0.0, 1.0);
    result.round() as u8
}

/// Circular interpolation for enum bytes (Container, Focus, Action).
///
/// Takes the shortest path around the circular value space.
/// For example, from Container 0 (Neutral) to Container 4 (Sharp),
/// the shortest path is 0 → 4 (forward, distance 4) not 0 → 255 → 4
/// (backward, distance 252).
fn lerp_circular(from: u8, to: u8, t: f32, count: u8) -> u8 {
    let from = from % count;
    let to = to % count;

    if from == to {
        return from;
    }

    // Calculate forward and backward distances
    let forward = ((to as i16 - from as i16 + count as i16) % count as i16) as u8;
    let backward = count - forward;

    let t = t.clamp(0.0, 1.0);

    // Take the shorter path around the circular space
    if forward <= backward {
        // Go forward
        let step = (t * forward as f32).round() as u8;
        ((from as u16 + step as u16) % count as u16) as u8
    } else {
        // Go backward
        let step = (t * backward as f32).round() as u8;
        ((from as u16 + count as u16 - step as u16) % count as u16) as u8
    }
}

// ── Temporal Decay ───────────────────────────────────────────────────────────

/// Interpolate a state toward the neutral baseline by a decay factor.
///
/// This implements the protocol's temporal decay mechanism: when
/// interaction ceases, the expression smoothly interpolates back to
/// the baseline neutral state over a time window (default: 5 seconds).
///
/// # Arguments
///
/// * `current` — The current active state
/// * `decay_factor` — How far toward neutral to decay (0.0 = no change,
///   1.0 = fully neutral). Typically calculated as `elapsed_time / decay_window`.
///
/// # Returns
///
/// A new `FacesState` interpolated between `current` and the neutral baseline.
///
/// # Example
///
/// ```
/// use faces_protocol::transition::decay_toward_neutral;
/// use faces_protocol::FacesState;
/// use faces_protocol::Aura;
/// use faces_protocol::Container;
/// use faces_protocol::Focus;
/// use faces_protocol::Action;
///
/// let active = FacesState::new(
///     Aura::CREATIVE,
///     Container::Fluid,
///     Focus::Happy,
///     Action::Playful,
/// );
///
/// // After 2.5 seconds of a 5-second decay window
/// let decayed = decay_toward_neutral(&active, 0.5);
/// ```
pub fn decay_toward_neutral(current: &FacesState, decay_factor: f32) -> FacesState {
    lerp(current, &FacesState::neutral(), decay_factor.clamp(0.0, 1.0))
}

// ═══════════════════════════════════════════════════════════════════════════════
// STATIC LOCK / DYNAMIC INTERACTION — TWO-PHASE TEMPORAL MODEL
// ═══════════════════════════════════════════════════════════════════════════════
//
// FACES state changes happen in two temporal phases:
//
//   1. STATIC LOCK — A committed state is frozen. No transitions occur.
//      This is the resting phase between interactions. The user's
//      emotional context is stable and protected.
//
//   2. DYNAMIC INTERACTION — Live state changes are allowed, but with
//      optional "keystroke friction" — resistance that models the
//      effort of changing one's mind. High friction rejects changes,
//      medium friction partially applies them (lerped), low friction
//      accepts them fully. Friction decays over time (warming up).
//
// The phase manager integrates with the Consent Gate:
//   StaticLock ↔ GateState::Locked / Committed
//   DynamicInteraction ↔ GateState::Unlocked
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::consent::{ConsentGate, GateState};

/// The two temporal phases of FACES state management.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Committed state is frozen — no transitions allowed.
    StaticLock,
    /// Live state changes allowed with optional friction.
    DynamicInteraction,
}

/// Result of attempting a state change during DynamicInteraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeResult {
    /// Change applied fully (friction was low).
    Accepted(FacesState),
    /// Change applied partially — lerped toward proposed (friction was medium).
    Partial(FacesState),
    /// Change refused — friction was too high.
    Rejected,
}

/// Two-phase temporal state manager for FACES.
///
/// Manages the transition between frozen (StaticLock) and live
/// (DynamicInteraction) states, with keystroke friction that models
/// the resistance of changing one's mind.
///
/// # Example
///
/// ```
/// use faces_protocol::transition::{PhaseManager, Phase, ChangeResult};
/// use faces_protocol::FacesState;
///
/// let mut pm = PhaseManager::new(FacesState::neutral());
/// assert_eq!(pm.phase(), Phase::StaticLock);
///
/// pm.force_unlock(0.0);
/// assert_eq!(pm.phase(), Phase::DynamicInteraction);
///
/// let result = pm.apply_change(&FacesState::default());
/// assert!(matches!(result, ChangeResult::Accepted(_)));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseManager {
    /// Current temporal phase.
    phase: Phase,
    /// The committed (locked-in) state.
    committed_state: FacesState,
    /// The live state (may drift from committed during DynamicInteraction).
    current_state: FacesState,
    /// Resistance to change: 0.0 = no resistance, 1.0 = maximum.
    friction: f32,
    /// Ticks since the current phase was entered.
    tick_count: u16,
    /// Minimum ticks in StaticLock before unlock is allowed.
    min_lock_ticks: u16,
    /// Drift threshold for acceptability checks.
    drift_threshold: u16,
}

impl PhaseManager {
    /// Create a new PhaseManager in StaticLock around the given state.
    pub fn new(state: FacesState) -> Self {
        Self {
            phase: Phase::StaticLock,
            committed_state: state,
            current_state: state,
            friction: 0.0,
            tick_count: 0,
            min_lock_ticks: 10,
            drift_threshold: 10,
        }
    }

    /// Create a PhaseManager synced from a ConsentGate.
    ///
    /// StaticLock if gate is Locked/Committed, DynamicInteraction if Unlocked.
    pub fn from_gate(gate: &ConsentGate) -> Self {
        let mut pm = Self::new(gate.locked_state());
        match gate.gate_state() {
            GateState::Locked | GateState::Committed => {
                // Stay in StaticLock
            }
            GateState::Unlocked => {
                pm.force_unlock(0.0);
            }
        }
        pm
    }

    /// Get the current phase.
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Get the committed state.
    pub fn committed_state(&self) -> FacesState {
        self.committed_state
    }

    /// Get the current (live) state.
    pub fn current_state(&self) -> FacesState {
        self.current_state
    }

    /// Get the current friction level.
    pub fn friction(&self) -> f32 {
        self.friction
    }

    /// Get ticks since phase was entered.
    pub fn tick_count(&self) -> u16 {
        self.tick_count
    }

    /// Set the minimum lock ticks before unlock is allowed.
    pub fn set_min_lock_ticks(&mut self, ticks: u16) {
        self.min_lock_ticks = ticks;
    }

    /// Set the drift acceptability threshold.
    pub fn set_drift_threshold(&mut self, threshold: u16) {
        self.drift_threshold = threshold;
    }

    // ── Phase Transitions ───────────────────────────────────────────────────

    /// Enter StaticLock phase, committing the given state.
    ///
    /// Freezes the state. Any drift from the previous committed state
    /// is discarded — the new committed state is what matters now.
    pub fn enter_static_lock(&mut self, state: FacesState) {
        self.committed_state = state;
        self.current_state = state;
        self.friction = 0.0;
        self.tick_count = 0;
        self.phase = Phase::StaticLock;
    }

    /// Enter DynamicInteraction phase with the given friction level.
    ///
    /// Only valid after `min_lock_ticks` have elapsed in StaticLock.
    /// Returns `true` if the transition succeeded, `false` if blocked
    /// by the minimum lock timer.
    pub fn enter_dynamic_interaction(&mut self, friction: f32) -> bool {
        if self.phase == Phase::StaticLock && self.tick_count < self.min_lock_ticks {
            return false;
        }
        self.friction = friction.clamp(0.0, 1.0);
        self.tick_count = 0;
        self.phase = Phase::DynamicInteraction;
        true
    }

    /// Emergency unlock — ignores min_lock_ticks.
    ///
    /// Forces entry into DynamicInteraction regardless of how long
    /// the gate has been in StaticLock. Used for urgent overrides.
    pub fn force_unlock(&mut self, friction: f32) {
        self.friction = friction.clamp(0.0, 1.0);
        self.tick_count = 0;
        self.phase = Phase::DynamicInteraction;
    }

    /// Tick the phase manager.
    ///
    /// Increments the tick counter. In DynamicInteraction, friction
    /// decays by 5% per tick (warming up — resistance decreases over time).
    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.saturating_add(1);
        if self.phase == Phase::DynamicInteraction {
            self.friction *= 0.95;
            if self.friction < 0.001 {
                self.friction = 0.0;
            }
        }
    }

    // ── Keystroke Friction ──────────────────────────────────────────────────

    /// Attempt to apply a state change during DynamicInteraction.
    ///
    /// Friction determines the outcome:
    /// - Friction > 0.8: `Rejected` (change refused)
    /// - Friction 0.3-0.8: `Partial` (change lerped toward proposed)
    /// - Friction < 0.3: `Accepted` (change applied fully)
    ///
    /// In StaticLock, all changes are rejected.
    pub fn apply_change(&mut self, proposed: &FacesState) -> ChangeResult {
        if self.phase == Phase::StaticLock {
            return ChangeResult::Rejected;
        }

        if self.friction > 0.8 {
            return ChangeResult::Rejected;
        }

        if self.friction >= 0.3 {
            let lerp_factor = 1.0 - self.friction;
            let new_state = lerp(&self.current_state, proposed, lerp_factor);
            self.current_state = new_state;
            return ChangeResult::Partial(new_state);
        }

        self.current_state = *proposed;
        ChangeResult::Accepted(*proposed)
    }

    // ── Drift Detection ─────────────────────────────────────────────────────

    /// Calculate how far the current state has drifted from committed.
    ///
    /// Returns the L1 magnitude of the transition vector between
    /// committed and current states.
    pub fn drift_magnitude(&self) -> u16 {
        TransitionVector::between(&self.committed_state, &self.current_state).magnitude()
    }

    /// Calculate the harmonic distance of drift from committed state.
    pub fn drift_harmonic(&self) -> f32 {
        TransitionVector::between(&self.committed_state, &self.current_state).harmonic_distance()
    }

    /// Check if drift is within acceptable threshold.
    ///
    /// Returns `true` if `drift_magnitude()` is below the threshold
    /// (default 10).
    pub fn is_drift_acceptable(&self) -> bool {
        self.drift_magnitude() < self.drift_threshold
    }

    // ── Recommit ────────────────────────────────────────────────────────────

    /// Recommit: the current state becomes the new committed state.
    ///
    /// Only valid in DynamicInteraction. After recommit, the phase
    /// returns to StaticLock.
    ///
    /// Returns the new committed state.
    pub fn recommit(&mut self) -> FacesState {
        self.committed_state = self.current_state;
        self.friction = 0.0;
        self.tick_count = 0;
        self.phase = Phase::StaticLock;
        self.committed_state
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_no_change() {
        let state = FacesState::neutral();
        let transition = TransitionVector::between(&state, &state);
        assert!(transition.is_stable());
        assert_eq!(transition.magnitude(), 0);
    }

    #[test]
    fn test_transition_aura_change() {
        let from = FacesState::neutral();
        let to = FacesState::new(
            Aura::from_index(100),
            Container::Neutral,
            Focus::Neutral,
            Action::Withheld,
        );
        let transition = TransitionVector::between(&from, &to);
        assert_eq!(transition.delta_aura, 100 - 245);
        assert_eq!(transition.delta_container, 0);
        assert_eq!(transition.delta_focus, 0);
        assert_eq!(transition.delta_action, 0);
    }

    #[test]
    fn test_transition_container_change() {
        let from = FacesState::neutral();
        let to = FacesState::new(
            Aura::NEUTRAL,
            Container::Sharp,
            Focus::Neutral,
            Action::Withheld,
        );
        let transition = TransitionVector::between(&from, &to);
        assert_eq!(transition.delta_container, 4);
    }

    #[test]
    fn test_transition_all_components() {
        let from = FacesState::neutral();
        let to = FacesState::new(
            Aura::URGENT,
            Container::Sharp,
            Focus::Intense,
            Action::Assertive,
        );
        let transition = TransitionVector::between(&from, &to);
        assert!(!transition.is_stable());
        assert!(transition.magnitude() > 0);
    }

    #[test]
    fn test_transition_is_volatile() {
        let from = FacesState::neutral();
        let to = FacesState::new(
            Aura::from_index(0),
            Container::Sharp,
            Focus::Tired,
            Action::Hesitant,
        );
        let transition = TransitionVector::between(&from, &to);
        // Aura change alone is 245, so this should be volatile
        assert!(transition.is_volatile());
    }

    #[test]
    fn test_transition_describe_stable() {
        let state = FacesState::neutral();
        let transition = TransitionVector::between(&state, &state);
        assert_eq!(transition.describe(), "stable (no change)");
    }

    #[test]
    fn test_transition_describe_shift() {
        let from = FacesState::neutral();
        let to = FacesState::new(
            Aura::CREATIVE,
            Container::Fluid,
            Focus::Happy,
            Action::Playful,
        );
        let transition = TransitionVector::between(&from, &to);
        let desc = transition.describe();
        assert!(desc.contains("shift"));
        assert!(desc.contains("magnitude"));
    }

    #[test]
    fn test_lerp_at_zero() {
        let from = FacesState::neutral();
        let to = FacesState::new(
            Aura::CREATIVE,
            Container::Fluid,
            Focus::Happy,
            Action::Playful,
        );
        let result = lerp(&from, &to, 0.0);
        assert_eq!(result, from);
    }

    #[test]
    fn test_lerp_at_one() {
        let from = FacesState::neutral();
        let to = FacesState::new(
            Aura::CREATIVE,
            Container::Fluid,
            Focus::Happy,
            Action::Playful,
        );
        let result = lerp(&from, &to, 1.0);
        assert_eq!(result, to);
    }

    #[test]
    fn test_lerp_at_half() {
        let from = FacesState::neutral();
        let to = FacesState::new(
            Aura::from_index(100),
            Container::Neutral,
            Focus::Neutral,
            Action::Withheld,
        );
        let result = lerp(&from, &to, 0.5);
        // (245 + 100) / 2 = 172.5, f32::round() rounds away from zero → 173
        let expected_aura = 173u8;
        assert_eq!(result.aura.index(), expected_aura);
    }

    #[test]
    fn test_lerp_clamps() {
        let from = FacesState::neutral();
        let to = FacesState::new(
            Aura::CREATIVE,
            Container::Fluid,
            Focus::Happy,
            Action::Playful,
        );
        // t > 1.0 should clamp to 1.0
        let result = lerp(&from, &to, 2.0);
        assert_eq!(result, to);
        // t < 0.0 should clamp to 0.0
        let result = lerp(&from, &to, -1.0);
        assert_eq!(result, from);
    }

    #[test]
    fn test_decay_full() {
        let state = FacesState::new(
            Aura::CREATIVE,
            Container::Fluid,
            Focus::Happy,
            Action::Playful,
        );
        let decayed = decay_toward_neutral(&state, 1.0);
        assert_eq!(decayed, FacesState::neutral());
    }

    #[test]
    fn test_decay_none() {
        let state = FacesState::new(
            Aura::CREATIVE,
            Container::Fluid,
            Focus::Happy,
            Action::Playful,
        );
        let decayed = decay_toward_neutral(&state, 0.0);
        assert_eq!(decayed, state);
    }

    #[test]
    fn test_decay_half() {
        let state = FacesState::new(
            Aura::from_index(100),
            Container::Neutral,
            Focus::Neutral,
            Action::Withheld,
        );
        let decayed = decay_toward_neutral(&state, 0.5);
        // (100 + 245) / 2 = 172.5, f32::round() rounds away from zero → 173
        let expected = 173u8;
        assert_eq!(decayed.aura.index(), expected);
    }

    // ── Pythagorean Harmonic Distance Tests ──────────────────────────────

    #[test]
    fn test_ratio_complexity_unison() {
        assert_eq!(ratio_complexity(0, 5), 0.0);
        assert_eq!(ratio_complexity(0, 6), 0.0);
    }

    #[test]
    fn test_ratio_complexity_minor_second() {
        // Step 1 = minor 2nd = most dissonant small step
        assert_eq!(ratio_complexity(1, 5), 0.83);
        assert_eq!(ratio_complexity(1, 6), 0.83);
    }

    #[test]
    fn test_ratio_complexity_major_third() {
        // Step 2 = major 3rd = consonant
        assert_eq!(ratio_complexity(2, 5), 0.43);
        assert_eq!(ratio_complexity(2, 6), 0.43);
    }

    #[test]
    fn test_ratio_complexity_perfect_fourth() {
        // Step 3 = perfect 4th = very consonant
        assert_eq!(ratio_complexity(3, 5), 0.33);
        assert_eq!(ratio_complexity(3, 6), 0.33);
    }

    #[test]
    fn test_ratio_complexity_wraps() {
        // Step 5 on a 5-value enum wraps to 0 (unison)
        assert_eq!(ratio_complexity(5, 5), 0.0);
        // Step 6 on a 6-value enum wraps to 0 (unison)
        assert_eq!(ratio_complexity(6, 6), 0.0);
    }

    #[test]
    fn test_ratio_complexity_minor_second_higher_than_major_third() {
        // The key Pythagorean insight: 1-step is MORE jarring than 2-step
        let step1 = ratio_complexity(1, 5);
        let step2 = ratio_complexity(2, 5);
        assert!(step1 > step2, "minor 2nd should be more complex than major 3rd");
    }

    #[test]
    fn test_harmonic_distance_no_change() {
        let state = FacesState::neutral();
        let transition = TransitionVector::between(&state, &state);
        assert_eq!(transition.harmonic_distance(), 0.0);
    }

    #[test]
    fn test_harmonic_distance_one_step_more_jarring_than_two() {
        // A 1-step Container shift should have higher harmonic distance
        // than a 2-step Container shift (minor 2nd vs major 3rd)
        let neutral = FacesState::neutral();

        let one_step = FacesState::new(
            Aura::NEUTRAL,
            Container::Rigid,    // 1 step from Neutral
            Focus::Neutral,
            Action::Withheld,
        );

        let two_step = FacesState::new(
            Aura::NEUTRAL,
            Container::Fluid,    // 2 steps from Neutral
            Focus::Neutral,
            Action::Withheld,
        );

        let t1 = TransitionVector::between(&neutral, &one_step);
        let t2 = TransitionVector::between(&neutral, &two_step);

        assert!(
            t1.harmonic_distance() > t2.harmonic_distance(),
            "1-step shift should have higher harmonic distance than 2-step"
        );
    }

    #[test]
    fn test_harmonic_distance_with_aura_change() {
        let from = FacesState::neutral();
        let to = FacesState::new(
            Aura::URGENT,
            Container::Neutral,
            Focus::Neutral,
            Action::Withheld,
        );
        let transition = TransitionVector::between(&from, &to);
        let hd = transition.harmonic_distance();
        // Aura change from 245 to 160 = |85|/255 * 3.32 ≈ 1.107
        assert!(hd > 1.0, "Aura change should contribute to harmonic distance");
    }

    #[test]
    fn test_harmonic_distance_all_dimensions() {
        let from = FacesState::neutral();
        let to = FacesState::new(
            Aura::URGENT,
            Container::Sharp,    // 4 steps
            Focus::Tired,        // 5 steps (wraps to octave)
            Action::Assertive,   // 1 step
        );
        let transition = TransitionVector::between(&from, &to);
        let hd = transition.harmonic_distance();
        // Should be positive and significant
        assert!(hd > 2.0, "Multi-dimension change should have significant harmonic distance");
    }

    #[test]
    fn test_harmonic_distance_vs_magnitude_different_ordering() {
        // Demonstrate that harmonic_distance and magnitude can disagree
        // on which transition is "larger"
        let neutral = FacesState::neutral();

        // 1-step Container change: magnitude=1, harmonic=0.83*2.32≈1.93
        let one_step = FacesState::new(
            Aura::NEUTRAL,
            Container::Rigid,
            Focus::Neutral,
            Action::Withheld,
        );

        // 4-step Container change: magnitude=4, harmonic=0.50*2.32≈1.16
        let four_step = FacesState::new(
            Aura::NEUTRAL,
            Container::Sharp,   // 4 steps from Neutral
            Focus::Neutral,
            Action::Withheld,
        );

        let t1 = TransitionVector::between(&neutral, &one_step);
        let t4 = TransitionVector::between(&neutral, &four_step);

        // magnitude: 1-step (1) < 4-step (4)
        assert!(t1.magnitude() < t4.magnitude());

        // But harmonic_distance: 1-step (minor 2nd, 0.83) > 4-step (perfect 5th, 0.50)
        // Both have same Aura so aura contribution is 0 for both.
        // This is the key insight: L1 says 1-step is smaller,
        // but Pythagorean ratios say 1-step is more jarring
        assert!(
            t1.harmonic_distance() > t4.harmonic_distance(),
            "1-step (minor 2nd) should have higher harmonic distance than 4-step (perfect 5th): {} vs {}",
            t1.harmonic_distance(),
            t4.harmonic_distance(),
        );
    }

    // ── PhaseManager Tests ─────────────────────────────────────────────────

    fn urgent_state() -> FacesState {
        FacesState::new(Aura::URGENT, Container::Sharp, Focus::Intense, Action::Assertive)
    }

    fn creative_state() -> FacesState {
        FacesState::new(Aura::CREATIVE, Container::Fluid, Focus::Open, Action::Playful)
    }

    #[test]
    fn test_phase_manager_new_is_static_lock() {
        let pm = PhaseManager::new(FacesState::neutral());
        assert_eq!(pm.phase(), Phase::StaticLock);
        assert_eq!(pm.committed_state(), FacesState::neutral());
        assert_eq!(pm.current_state(), FacesState::neutral());
        assert_eq!(pm.friction(), 0.0);
        assert_eq!(pm.tick_count(), 0);
    }

    #[test]
    fn test_enter_static_lock() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.enter_static_lock(urgent_state());
        assert_eq!(pm.phase(), Phase::StaticLock);
        assert_eq!(pm.committed_state(), urgent_state());
        assert_eq!(pm.current_state(), urgent_state());
        assert_eq!(pm.friction(), 0.0);
        assert_eq!(pm.tick_count(), 0);
    }

    #[test]
    fn test_enter_dynamic_interaction_after_min_ticks() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        // Need min_lock_ticks (default 10) ticks before unlock
        for _ in 0..10 {
            pm.tick();
        }
        assert!(pm.enter_dynamic_interaction(0.5));
        assert_eq!(pm.phase(), Phase::DynamicInteraction);
        assert_eq!(pm.friction(), 0.5);
        assert_eq!(pm.tick_count(), 0);
    }

    #[test]
    fn test_enter_dynamic_interaction_blocked_by_min_ticks() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        for _ in 0..5 {
            pm.tick();
        }
        assert!(!pm.enter_dynamic_interaction(0.5));
        assert_eq!(pm.phase(), Phase::StaticLock);
    }

    #[test]
    fn test_force_unlock_ignores_min_ticks() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        // No ticks at all, but force_unlock should work
        pm.force_unlock(0.5);
        assert_eq!(pm.phase(), Phase::DynamicInteraction);
        assert_eq!(pm.friction(), 0.5);
    }

    #[test]
    fn test_tick_increments_count() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        assert_eq!(pm.tick_count(), 0);
        pm.tick();
        assert_eq!(pm.tick_count(), 1);
        pm.tick();
        assert_eq!(pm.tick_count(), 2);
    }

    #[test]
    fn test_tick_resets_on_phase_change() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        for _ in 0..10 {
            pm.tick();
        }
        assert_eq!(pm.tick_count(), 10);

        pm.enter_dynamic_interaction(0.0);
        assert_eq!(pm.tick_count(), 0);
    }

    // ── Keystroke Friction Tests ───────────────────────────────────────────

    #[test]
    fn test_apply_change_rejected_in_static_lock() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        let result = pm.apply_change(&urgent_state());
        assert_eq!(result, ChangeResult::Rejected);
        assert_eq!(pm.current_state(), FacesState::neutral());
    }

    #[test]
    fn test_apply_change_accepted_low_friction() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.force_unlock(0.1);
        let result = pm.apply_change(&urgent_state());
        assert!(matches!(result, ChangeResult::Accepted(_)));
        assert_eq!(pm.current_state(), urgent_state());
    }

    #[test]
    fn test_apply_change_rejected_high_friction() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.force_unlock(0.9);
        let result = pm.apply_change(&urgent_state());
        assert_eq!(result, ChangeResult::Rejected);
        assert_eq!(pm.current_state(), FacesState::neutral());
    }

    #[test]
    fn test_apply_change_partial_medium_friction() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.force_unlock(0.5);
        let result = pm.apply_change(&urgent_state());
        assert!(matches!(result, ChangeResult::Partial(_)));
        // Current state should be between neutral and urgent, not equal to either
        assert_ne!(pm.current_state(), FacesState::neutral());
        assert_ne!(pm.current_state(), urgent_state());
    }

    #[test]
    fn test_friction_decays_over_time() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.force_unlock(0.9);
        assert!((pm.friction() - 0.9).abs() < 0.001);

        for _ in 0..200 {
            pm.tick();
        }
        assert!(pm.friction() < 0.001, "Friction should decay to near-zero over time: {}", pm.friction());
    }

    #[test]
    fn test_friction_decay_allows_change_after_warming() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.force_unlock(0.9);

        // Initially rejected
        let r1 = pm.apply_change(&urgent_state());
        assert_eq!(r1, ChangeResult::Rejected);

        // Tick until friction drops below 0.8
        while pm.friction() > 0.8 {
            pm.tick();
        }

        // Now should be at least partial
        let r2 = pm.apply_change(&urgent_state());
        assert!(!matches!(r2, ChangeResult::Rejected), "Should not be rejected after friction decayed");
    }

    #[test]
    fn test_friction_does_not_decay_in_static_lock() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.force_unlock(0.5);
        pm.enter_static_lock(FacesState::neutral());
        for _ in 0..100 {
            pm.tick();
        }
        // Friction is reset to 0 on enter_static_lock, and doesn't decay (already 0)
        assert_eq!(pm.friction(), 0.0);
    }

    // ── Drift Detection Tests ──────────────────────────────────────────────

    #[test]
    fn test_drift_magnitude_zero_when_unchanged() {
        let pm = PhaseManager::new(FacesState::neutral());
        assert_eq!(pm.drift_magnitude(), 0);
    }

    #[test]
    fn test_drift_magnitude_nonzero_after_change() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.force_unlock(0.0);
        pm.apply_change(&urgent_state());
        assert!(pm.drift_magnitude() > 0);
    }

    #[test]
    fn test_drift_harmonic_zero_when_unchanged() {
        let pm = PhaseManager::new(FacesState::neutral());
        assert_eq!(pm.drift_harmonic(), 0.0);
    }

    #[test]
    fn test_drift_harmonic_nonzero_after_change() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.force_unlock(0.0);
        pm.apply_change(&urgent_state());
        assert!(pm.drift_harmonic() > 0.0);
    }

    #[test]
    fn test_is_drift_acceptable_within_threshold() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.force_unlock(0.0);

        // Apply a small change (1 step on container only)
        let slightly_different = FacesState::new(
            Aura::NEUTRAL,
            Container::Rigid,
            Focus::Neutral,
            Action::Withheld,
        );
        pm.apply_change(&slightly_different);
        assert!(pm.is_drift_acceptable(), "Small drift should be acceptable");
    }

    #[test]
    fn test_is_drift_acceptable_exceeds_threshold() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.force_unlock(0.0);
        pm.apply_change(&urgent_state());
        assert!(!pm.is_drift_acceptable(), "Large drift should not be acceptable");
    }

    #[test]
    fn test_set_drift_threshold() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.set_drift_threshold(100);
        pm.force_unlock(0.0);
        pm.apply_change(&urgent_state());
        assert!(pm.is_drift_acceptable(), "With high threshold, even large drift is acceptable");
    }

    // ── Recommit Tests ─────────────────────────────────────────────────────

    #[test]
    fn test_recommit_returns_to_static_lock() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.force_unlock(0.0);
        pm.apply_change(&urgent_state());

        let committed = pm.recommit();
        assert_eq!(committed, urgent_state());
        assert_eq!(pm.committed_state(), urgent_state());
        assert_eq!(pm.phase(), Phase::StaticLock);
        assert_eq!(pm.friction(), 0.0);
        assert_eq!(pm.tick_count(), 0);
    }

    #[test]
    fn test_recommit_drift_becomes_zero() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.force_unlock(0.0);
        pm.apply_change(&creative_state());
        assert!(pm.drift_magnitude() > 0);

        pm.recommit();
        assert_eq!(pm.drift_magnitude(), 0, "Drift should be zero after recommit");
    }

    // ── Full Cycle Tests ───────────────────────────────────────────────────

    #[test]
    fn test_full_cycle_static_dynamic_recommit() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        assert_eq!(pm.phase(), Phase::StaticLock);

        // Tick past min_lock_ticks
        for _ in 0..10 {
            pm.tick();
        }

        // Enter dynamic
        assert!(pm.enter_dynamic_interaction(0.0));
        assert_eq!(pm.phase(), Phase::DynamicInteraction);

        // Apply change
        pm.apply_change(&creative_state());
        assert_ne!(pm.current_state(), FacesState::neutral());

        // Recommit
        pm.recommit();
        assert_eq!(pm.phase(), Phase::StaticLock);
        assert_eq!(pm.committed_state(), creative_state());
    }

    #[test]
    fn test_multiple_cycles() {
        let mut pm = PhaseManager::new(FacesState::neutral());

        // First cycle
        for _ in 0..10 { pm.tick(); }
        pm.enter_dynamic_interaction(0.0);
        pm.apply_change(&urgent_state());
        pm.recommit();
        assert_eq!(pm.committed_state(), urgent_state());

        // Second cycle
        for _ in 0..10 { pm.tick(); }
        pm.enter_dynamic_interaction(0.0);
        pm.apply_change(&creative_state());
        pm.recommit();
        assert_eq!(pm.committed_state(), creative_state());
    }

    // ── Consent Gate Integration Tests ─────────────────────────────────────

    #[test]
    fn test_from_gate_locked() {
        let gate = ConsentGate::new(FacesState::neutral());
        let pm = PhaseManager::from_gate(&gate);
        assert_eq!(pm.phase(), Phase::StaticLock);
        assert_eq!(pm.committed_state(), FacesState::neutral());
    }

    #[test]
    fn test_from_gate_unlocked() {
        let mut gate = ConsentGate::new(FacesState::neutral());
        gate.unlock();
        let pm = PhaseManager::from_gate(&gate);
        assert_eq!(pm.phase(), Phase::DynamicInteraction);
    }

    #[test]
    fn test_from_gate_committed() {
        let mut gate = ConsentGate::new(FacesState::neutral());
        gate.unlock();
        gate.propose(urgent_state());
        gate.commit();
        let pm = PhaseManager::from_gate(&gate);
        assert_eq!(pm.phase(), Phase::StaticLock);
        assert_eq!(pm.committed_state(), urgent_state());
    }

    // ── Edge Cases ─────────────────────────────────────────────────────────

    #[test]
    fn test_set_min_lock_ticks() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.set_min_lock_ticks(3);
        for _ in 0..3 {
            pm.tick();
        }
        assert!(pm.enter_dynamic_interaction(0.0));
    }

    #[test]
    fn test_enter_dynamic_from_dynamic_succeeds() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.force_unlock(0.5);
        // Already in DynamicInteraction, so min_lock_ticks doesn't apply
        assert!(pm.enter_dynamic_interaction(0.3));
        assert_eq!(pm.friction(), 0.3);
    }

    #[test]
    fn test_apply_change_same_state_accepted() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.force_unlock(0.0);
        let result = pm.apply_change(&FacesState::neutral());
        assert!(matches!(result, ChangeResult::Accepted(_)));
    }

    #[test]
    fn test_partial_change_lerp_factor() {
        let mut pm = PhaseManager::new(FacesState::neutral());
        pm.force_unlock(0.6); // lerp_factor = 1.0 - 0.6 = 0.4
        let result = pm.apply_change(&urgent_state());
        if let ChangeResult::Partial(new_state) = result {
            // Should be 40% of the way from neutral to urgent
            let tv = TransitionVector::between(&FacesState::neutral(), &new_state);
            let full_tv = TransitionVector::between(&FacesState::neutral(), &urgent_state());
            // Aura is the main continuous dimension — check it moved
            assert!(tv.delta_aura != 0,
                "Partial change should move aura toward proposed");
            // Should not be the full distance
            assert!(tv.magnitude() < full_tv.magnitude(),
                "Partial change should be less than full change: {} vs {}",
                tv.magnitude(), full_tv.magnitude());
        } else {
            panic!("Expected ChangeResult::Partial");
        }
    }
}
