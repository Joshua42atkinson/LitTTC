// ═══════════════════════════════════════════════════════════════════════════════
// FACES PROTOCOL — faces-protocol
// FILE:        src/detect.rs
// PURPOSE:     Text-to-FACES detection — maps natural language to FacesState
// ═══════════════════════════════════════════════════════════════════════════════
//
// TEXT-TO-FACES DETECTION
//
// This module maps natural language text to a `FacesState`, replacing
// Trinity's legacy `detect_emotion()` function (which used simple
// keyword matching with only 6 states) with the full 38,400-state
// FACES protocol.
//
// The detection uses a rule-based approach with syntactic isomorphism
// — mapping parts of speech and semantic categories to the 4-byte
// payload. This is the "zero-compute" path that requires no LLM
// inference or neural network. It runs on CPU with near-zero latency.
//
// For higher accuracy, the FACES-Embed model (~66M param DistilBERT
// encoder, ONNX for NPU) can be used as a drop-in replacement. See the
// "npu" feature flag (future) for that integration.
//
// SYNTACTIC ISOMORPHISM — THE GRAMMAR OF PAREIDOLIA
//
// The FACES protocol defines a formal mapping between linguistic
// grammar (Parts of Speech) and the 4-byte FACES Matrix:
//
//   1. The NOUN (Subject)        ↔ Container (Byte 1)
//      — entity, boundary, structure, identity
//   2. The ADJECTIVE (Modifier)  ↔ Aura (Byte 0)
//      — qualitative state, tone, temperature
//   3. The VERB (Action)         ↔ Action (Byte 3)
//      — kinetic readiness, dynamic output
//   4. The ADVERB (Modifier)     ↔ Focus (Byte 2)
//      — intensity, directional focus, processing load
//
// EXAMPLE: "He quickly whispered"
//   Subject (He)         → Container: Neutral ()
//   Verb (whispered)     → Action: Hesitant (.)
//   Adverb (quickly)     → Focus: Intense (><)
//   Implicit Adj (secretive) → Aura: Cool Cyan (39)
//   Result: (><.) in cool cyan
//
// DETECTION STRATEGY
//
// The current implementation uses keyword and pattern matching across
// four dimensions:
//
//   1. ACTION detection — scans for verbs/action words
//   2. FOCUS detection — scans for adverbs/intensity markers
//   3. CONTAINER detection — scans for structural/formality markers
//   4. AURA detection — scans for emotional tone words
//
// Each dimension is scored independently, and the highest-scoring
// category wins. If no keywords match, the default (neutral) state
// is returned.
//
// CONGRUENCE AND INCONGRUENCE
//
// The protocol supports both congruent and incongruent states:
//   - Congruent: Text sentiment matches the FACES visual state
//     (e.g., positive text with a happy face)
//   - Incongruent: Visual state intentionally diverges from text
//     to indicate sarcasm, cognitive fatigue, or high thought-load
//     (e.g., high-energy text paired with a "tired" visual)
//
// The current detector produces congruent states. Incongruent
// detection (sarcasm, irony) requires the FACES-Embed model or
// explicit user override via the Consent Gate.
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::action::Action;
use crate::aura::Aura;
use crate::container::Container;
use crate::focus::Focus;
use crate::protocol::FacesState;

// ── Scored Detection Types ───────────────────────────────────────────────────

/// How a FACES state was detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionMethod {
    /// Keyword-based detection (current zero-compute path).
    Keyword,
    /// Rule-based congruence adjustment (heuristic layer).
    Heuristic,
    /// Neural detection via FACES-Embed on NPU (future).
    Neural,
}

/// Whether the detected dimensions agree or conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Congruence {
    /// All dimensions share the same emotional valence.
    Congruent,
    /// Dimensions conflict (e.g., happy aura but sharp container).
    Incongruent,
    /// Not enough signal to determine congruence.
    Neutral,
}

/// Result of scored text-to-FACES detection.
///
/// Contains the best-guess `FacesState` plus per-dimension confidence
/// scores (0.0 to 1.0), an overall confidence, congruence assessment,
/// and the detection method used.
#[derive(Debug, Clone, PartialEq)]
pub struct DetectionResult {
    /// The best-guess FACES state.
    pub state: FacesState,
    /// Confidence score for Aura detection (0.0 = no signal, 1.0 = certain).
    pub aura_confidence: f32,
    /// Confidence score for Container detection.
    pub container_confidence: f32,
    /// Confidence score for Focus detection.
    pub focus_confidence: f32,
    /// Confidence score for Action detection.
    pub action_confidence: f32,
    /// Weighted average confidence across all dimensions.
    pub overall_confidence: f32,
    /// Whether the detected dimensions are congruent or incongruent.
    pub congruence: Congruence,
    /// How this state was detected.
    pub method: DetectionMethod,
}

impl DetectionResult {
    /// Build a DetectionResult from per-dimension scores.
    fn assemble(
        state: FacesState,
        aura_conf: f32,
        container_conf: f32,
        focus_conf: f32,
        action_conf: f32,
    ) -> Self {
        // Weighted average: Aura has the most semantic weight (10 options),
        // then Focus (6), then Container and Action (5 each).
        let overall = (aura_conf * 3.32 + container_conf * 2.32 + focus_conf * 2.58 + action_conf * 2.32)
            / (3.32 + 2.32 + 2.58 + 2.32);

        let congruence = detect_congruence(&state);

        Self {
            state,
            aura_confidence: aura_conf,
            container_confidence: container_conf,
            focus_confidence: focus_conf,
            action_confidence: action_conf,
            overall_confidence: overall,
            congruence,
            method: DetectionMethod::Keyword,
        }
    }
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Detect a FACES state from natural language text with confidence scores.
///
/// This is the primary entry point for scored text-to-FACES mapping.
/// Unlike `detect_faces()` which returns only the state, this function
/// returns a `DetectionResult` with per-dimension confidence scores,
/// congruence assessment, and detection method.
///
/// # Scoring Algorithm
///
/// For each dimension (Aura, Container, Focus, Action), the function:
/// 1. Scans ALL keyword sets (not just first match)
/// 2. Counts keyword hits per candidate variant
/// 3. The variant with the most hits wins
/// 4. Confidence = winner_hits / total_hits (softmax-like normalization)
/// 5. If no keywords match, returns neutral with confidence 0.0
///
/// Intensity modifiers ("very", "extremely") boost confidence.
/// Negation ("not urgent") reduces confidence for negated keywords.
///
/// # Arguments
///
/// * `text` — The input text to analyze (case-insensitive)
///
/// # Example
///
/// ```
/// use faces_protocol::detect::detect_scored;
///
/// let result = detect_scored("Critical error: system crash!");
/// assert!(result.aura_confidence > 0.0);
/// assert_eq!(result.congruence, faces_protocol::detect::Congruence::Congruent);
/// ```
pub fn detect_scored(text: &str) -> DetectionResult {
    let lower = text.to_lowercase();

    let (action, action_conf) = detect_action_scored(&lower);
    let (focus, focus_conf) = detect_focus_scored(&lower);
    let (container, container_conf) = detect_container_scored(&lower);
    let (aura, aura_conf) = detect_aura_scored(&lower);

    let state = FacesState::new(aura, container, focus, action);
    DetectionResult::assemble(state, aura_conf, container_conf, focus_conf, action_conf)
}

/// Detect a FACES state from natural language text.
///
/// Backward-compatible wrapper around `detect_scored()` that returns
/// only the `FacesState` without confidence information.
///
/// # Example
///
/// ```
/// use faces_protocol::detect::detect_faces;
/// use faces_protocol::Container;
/// use faces_protocol::Focus;
/// use faces_protocol::Action;
///
/// let state = detect_faces("Congratulations! Quest complete!");
/// assert_eq!(state.focus, Focus::Happy);
/// assert_eq!(state.action, Action::Assertive);
/// ```
pub fn detect_faces(text: &str) -> FacesState {
    detect_scored(text).state
}

// ── Multi-Sentence Detection ─────────────────────────────────────────────────

/// Detect FACES states for each sentence in a multi-sentence text.
///
/// Splits the input into sentences using `segment_sentences()`, then
/// runs `detect_scored()` on each independently. Returns one
/// `DetectionResult` per sentence.
///
/// # Arguments
///
/// * `text` — Multi-sentence input text
///
/// # Example
///
/// ```
/// use faces_protocol::detect::detect_multi;
///
/// let results = detect_multi("Critical error! Let's brainstorm a fix.");
/// assert_eq!(results.len(), 2);
/// ```
pub fn detect_multi(text: &str) -> Vec<DetectionResult> {
    crate::segment::segment_sentences(text)
        .iter()
        .map(|s| detect_scored(s))
        .collect()
}

/// Detect a single aggregate FACES state from multi-sentence text.
///
/// Runs `detect_multi()` then aggregates the results into a single
/// `DetectionResult`. Later sentences are weighted higher (recency bias),
/// and confidence is boosted when multiple sentences agree on a dimension.
///
/// # Aggregation Algorithm
///
/// 1. Each sentence gets a weight: `w_i = (i + 1) / sum(1..n)` (recency bias)
/// 2. Per-dimension: weighted average of confidence scores
/// 3. State: the state from the highest-weighted (last) sentence with
///    confidence > 0.1, or the highest-confidence sentence otherwise
/// 4. Agreement boost: if >60% of sentences agree on a dimension's variant,
///    confidence is boosted by 0.1 (capped at 1.0)
///
/// # Arguments
///
/// * `text` — Multi-sentence input text
///
/// # Example
///
/// ```
/// use faces_protocol::detect::detect_aggregate;
///
/// let result = detect_aggregate("Critical error! We must fix this. Deploy now!");
/// assert!(result.overall_confidence > 0.0);
/// ```
pub fn detect_aggregate(text: &str) -> DetectionResult {
    let results = detect_multi(text);

    if results.is_empty() {
        return DetectionResult::assemble(
            FacesState::neutral(),
            0.0, 0.0, 0.0, 0.0,
        );
    }

    if results.len() == 1 {
        return results.into_iter().next().unwrap();
    }

    let n = results.len() as f32;
    let total_weight: f32 = (1..=results.len()).map(|i| i as f32).sum();
    let weights: Vec<f32> = (1..=results.len())
        .map(|i| i as f32 / total_weight)
        .collect();

    let mut aura_conf = 0.0;
    let mut container_conf = 0.0;
    let mut focus_conf = 0.0;
    let mut action_conf = 0.0;

    for (i, r) in results.iter().enumerate() {
        let w = weights[i];
        aura_conf += r.aura_confidence * w;
        container_conf += r.container_confidence * w;
        focus_conf += r.focus_confidence * w;
        action_conf += r.action_confidence * w;
    }

    let best_idx = results
        .iter()
        .enumerate()
        .max_by(|(i, a), (j, b)| {
            let wa = a.overall_confidence * weights[*i];
            let wb = b.overall_confidence * weights[*j];
            wa.partial_cmp(&wb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(results.len() - 1);

    let state = results[best_idx].state;

    let mut aura_variants = [0usize; 10];
    let mut container_variants = [0usize; 5];
    let mut focus_variants = [0usize; 6];
    let mut action_variants = [0usize; 5];

    for r in &results {
        if r.aura_confidence > 0.0 {
            let idx = r.state.aura.index() % 10;
            aura_variants[idx as usize] += 1;
        }
        if r.container_confidence > 0.0 {
            container_variants[r.state.container as usize] += 1;
        }
        if r.focus_confidence > 0.0 {
            focus_variants[r.state.focus as usize] += 1;
        }
        if r.action_confidence > 0.0 {
            action_variants[r.state.action as usize] += 1;
        }
    }

    let threshold = (n * 0.6).ceil() as usize;
    if *aura_variants.iter().max().unwrap_or(&0) >= threshold && aura_conf > 0.0 {
        aura_conf = (aura_conf + 0.1).min(1.0);
    }
    if *container_variants.iter().max().unwrap_or(&0) >= threshold && container_conf > 0.0 {
        container_conf = (container_conf + 0.1).min(1.0);
    }
    if *focus_variants.iter().max().unwrap_or(&0) >= threshold && focus_conf > 0.0 {
        focus_conf = (focus_conf + 0.1).min(1.0);
    }
    if *action_variants.iter().max().unwrap_or(&0) >= threshold && action_conf > 0.0 {
        action_conf = (action_conf + 0.1).min(1.0);
    }

    DetectionResult::assemble(state, aura_conf, container_conf, focus_conf, action_conf)
}

// ── Keyword Tables ───────────────────────────────────────────────────────────

// Action keywords (Byte 3 — verb/mouth)
const ACTION_ASSERTIVE: &[&str] = &[
    "must", "need to", "required", "do this", "execute", "deploy", "build it",
    "ship it", "assert", "confirm", "congratulations", "quest complete",
    "milestone", "leveled up", "xp awarded", "excellent", "success", "achieved",
    "done", "complete", "finished", "delivered", "launched", "committed",
    "go ahead", "proceed", "make it happen",
];
const ACTION_PLAYFUL: &[&str] = &[
    "what if", "imagine", "playful", "creative", "irony", "joke", "fun",
    "explore", "brainstorm", "maybe we could", "oh really", "interesting choice",
    "bold move", "wild", "crazy idea", "let's try", "experiment", "tinker",
    "toy with", "doodle", "sketch",
];
const ACTION_THOUGHTFUL: &[&str] = &[
    "have you considered", "perhaps", "thought about", "evaluate", "analyze",
    "consider", "reflect", "think about", "assess", "review", "concern",
    "wonder", "question", "why", "how might", "what might", "ponder",
    "deliberate", "weigh", "contemplate", "examine",
];
const ACTION_HESITANT: &[&str] = &[
    "maybe", "might", "not sure", "uncertain", "could be wrong", "i think",
    "possibly", "error", "failed", "retry", "hmm", "unsure", "approximate",
    "roughly", "sort of", "kind of", "i guess", "i suppose", "if i'm not mistaken",
];

// Focus keywords (Byte 2 — adverb/eyes)
const FOCUS_INTENSE: &[&str] = &[
    "critical", "urgent", "error", "failed", "breakpoint", "compile error",
    "crash", "warning", "deadline", "cannot be undone", "immediately", "asap",
    "high priority", "danger", "pressure", "now", "right away", "no time",
    "red alert", "all hands",
];
const FOCUS_OPEN: &[&str] = &[
    "wow", "unexpected", "surprising", "amazing", "shocking", "whoa",
    "incredible", "didn't expect", "new", "discovered", "breakthrough",
    "never seen", "fascinating", "remarkable", "astonishing", "revelation",
];
const FOCUS_HAPPY: &[&str] = &[
    "great job", "well done", "keep going", "making progress", "congratulations",
    "excellent", "perfect", "beautiful", "wonderful", "fantastic", "quest complete",
    "success", "milestone", "nailed it", "proud", "thrilled", "delighted",
];
const FOCUS_DISTANT: &[&str] = &[
    "waiting", "idle", "background", "pending", "queued", "loading", "standby",
    "bored", "nothing to do", "zzz", "sleeping", "paused", "suspended", "stalled",
];
const FOCUS_TIRED: &[&str] = &[
    "tired", "exhausted", "depleted", "low battery", "low memory", "overloaded",
    "slow", "timeout", "rate limited", "quota", "drained", "burned out", "fatigue",
    "weary", "spent",
];

// Container keywords (Byte 1 — noun/head shape)
const CONTAINER_SHARP: &[&str] = &[
    "critical", "emergency", "danger", "threat", "attack", "breach",
    "high priority", "asap", "immediately", "aggressive", "urgent", "red line",
    "deadline", "now", "force",
];
const CONTAINER_DEFENSIVE: &[&str] = &[
    "security", "protected", "caution", "guard", "shield", "defensive",
    "boundary", "restrict", "permission denied", "unauthorized", "firewall",
    "encrypted", "secure", "lock down", "quarantine",
];
const CONTAINER_RIGID: &[&str] = &[
    "protocol", "formal", "rule", "standard", "specification", "requirement",
    "constraint", "schema", "compliance", "must", "required", "regulation",
    "policy", "guideline", "framework",
];
const CONTAINER_FLUID: &[&str] = &[
    "creative", "adaptive", "flexible", "flow", "imagine", "brainstorm",
    "explore", "what if", "maybe we could", "novel", "unconventional",
    "vulnerable", "open", "organic", "evolving", "emergent",
];

// Aura keywords (Byte 0 — adjective/color)
const AURA_URGENT: &[&str] = &[
    "critical", "urgent", "error", "failed", "crash", "danger", "warning",
    "emergency", "alarm", "alert", "red", "fatal", "severe",
];
const AURA_CREATIVE: &[&str] = &[
    "creative", "growth", "learn", "build", "develop", "imagine", "explore",
    "brainstorm", "novel", "innovative", "invent", "design", "craft", "green",
    "spring",
];
const AURA_HAPPY: &[&str] = &[
    "congratulations", "excellent", "success", "milestone", "great job",
    "well done", "beautiful", "wonderful", "happy", "quest complete", "joy",
    "celebrate", "triumph", "victory", "cheer",
];
const AURA_CONTEMPLATIVE: &[&str] = &[
    "have you considered", "perhaps", "reflect", "think about", "wonder",
    "why", "philosophical", "socratic", "question", "evaluate", "analyze",
    "meaning", "deep", "profound", "meditate",
];
const AURA_ANALYTICAL: &[&str] = &[
    "analyze", "debug", "inspect", "trace", "log", "audit", "verify", "test",
    "compile", "benchmark", "profile", "measure", "quantify", "systematic",
    "methodical",
];
const AURA_TIRED: &[&str] = &[
    "tired", "exhausted", "depleted", "slow", "timeout", "rate limited",
    "low battery", "low memory", "drained", "burned out", "fatigue", "weary",
    "spent", "gray",
];
const AURA_CALM: &[&str] = &[
    "calm", "peaceful", "steady", "stable", "consistent", "reliable", "serene",
    "tranquil", "composed", "grounded", "centered", "blue", "ocean",
];
const AURA_ENERGETIC: &[&str] = &[
    "energetic", "enthusiastic", "excited", "pumped", "ready to go", "fired up",
    "motivated", "driven", "passionate", "orange", "dynamic", "vibrant",
];
const AURA_UNCONVENTIONAL: &[&str] = &[
    "weird", "strange", "unexpected", "unconventional", "odd", "peculiar",
    "bizarre", "quirky", "unusual", "magenta", "eccentric", "avant-garde",
];

// ── Intensity & Negation ─────────────────────────────────────────────────────

const INTENSITY_BOOSTERS: &[&str] = &[
    "very", "extremely", "highly", "super", "really", "incredibly",
    "absolutely", "utterly", "profoundly", "deeply",
];
const INTENSITY_DIMINISHERS: &[&str] = &[
    "slightly", "somewhat", "kind of", "a bit", "sort of", "mildly",
    "rather", "fairly",
];
const NEGATION_WORDS: &[&str] = &[
    "not ", "no ", "never ", "don't ", "doesn't ", "isn't ", "wasn't ",
    "aren't ", "cannot ", "won't ",
];

#[allow(dead_code)]
fn count_hits(lower: &str, keywords: &[&str]) -> u16 {
    keywords.iter().filter(|kw| lower.contains(*kw)).count() as u16
}

fn is_negated(lower: &str, keyword: &str) -> bool {
    if let Some(pos) = lower.find(keyword) {
        let start = pos.saturating_sub(15);
        let preceding = &lower[start..pos];
        NEGATION_WORDS.iter().any(|neg| preceding.contains(neg))
    } else {
        false
    }
}

fn count_hits_filtered(lower: &str, keywords: &[&str]) -> u16 {
    keywords.iter().filter(|kw| lower.contains(*kw) && !is_negated(lower, kw)).count() as u16
}

fn intensity_multiplier(lower: &str) -> f32 {
    let has_booster = INTENSITY_BOOSTERS.iter().any(|w| lower.contains(w));
    let has_diminisher = INTENSITY_DIMINISHERS.iter().any(|w| lower.contains(w));
    if has_booster && !has_diminisher { 1.15 } else if has_diminisher && !has_booster { 0.70 } else { 1.0 }
}

// ── Scored Dimension Detection ───────────────────────────────────────────────

fn detect_action_scored(lower: &str) -> (Action, f32) {
    let assertive = count_hits_filtered(lower, ACTION_ASSERTIVE);
    let playful = count_hits_filtered(lower, ACTION_PLAYFUL);
    let thoughtful = count_hits_filtered(lower, ACTION_THOUGHTFUL);
    let hesitant = count_hits_filtered(lower, ACTION_HESITANT);
    let total = assertive + playful + thoughtful + hesitant;
    if total == 0 { return (Action::Withheld, 0.0); }
    let intensity = intensity_multiplier(lower);
    let (winner, hits) = [
        (Action::Assertive, assertive), (Action::Playful, playful),
        (Action::Thoughtful, thoughtful), (Action::Hesitant, hesitant),
    ].iter().copied().max_by_key(|(_, h)| *h).unwrap();
    let confidence = (hits as f32 / total as f32 * intensity).min(1.0);
    (winner, confidence)
}

fn detect_focus_scored(lower: &str) -> (Focus, f32) {
    let intense = count_hits_filtered(lower, FOCUS_INTENSE);
    let open = count_hits_filtered(lower, FOCUS_OPEN);
    let happy = count_hits_filtered(lower, FOCUS_HAPPY);
    let distant = count_hits_filtered(lower, FOCUS_DISTANT);
    let tired = count_hits_filtered(lower, FOCUS_TIRED);
    let total = intense + open + happy + distant + tired;
    if total == 0 { return (Focus::Neutral, 0.0); }
    let intensity = intensity_multiplier(lower);
    let (winner, hits) = [
        (Focus::Intense, intense), (Focus::Open, open), (Focus::Happy, happy),
        (Focus::Distant, distant), (Focus::Tired, tired),
    ].iter().copied().max_by_key(|(_, h)| *h).unwrap();
    let confidence = (hits as f32 / total as f32 * intensity).min(1.0);
    (winner, confidence)
}

fn detect_container_scored(lower: &str) -> (Container, f32) {
    let sharp = count_hits_filtered(lower, CONTAINER_SHARP);
    let defensive = count_hits_filtered(lower, CONTAINER_DEFENSIVE);
    let rigid = count_hits_filtered(lower, CONTAINER_RIGID);
    let fluid = count_hits_filtered(lower, CONTAINER_FLUID);
    let total = sharp + defensive + rigid + fluid;
    if total == 0 { return (Container::Neutral, 0.0); }
    let intensity = intensity_multiplier(lower);
    let (winner, hits) = [
        (Container::Sharp, sharp), (Container::Defensive, defensive),
        (Container::Rigid, rigid), (Container::Fluid, fluid),
    ].iter().copied().max_by_key(|(_, h)| *h).unwrap();
    let confidence = (hits as f32 / total as f32 * intensity).min(1.0);
    (winner, confidence)
}

fn detect_aura_scored(lower: &str) -> (Aura, f32) {
    let urgent = count_hits_filtered(lower, AURA_URGENT);
    let creative = count_hits_filtered(lower, AURA_CREATIVE);
    let happy = count_hits_filtered(lower, AURA_HAPPY);
    let contemplative = count_hits_filtered(lower, AURA_CONTEMPLATIVE);
    let analytical = count_hits_filtered(lower, AURA_ANALYTICAL);
    let tired = count_hits_filtered(lower, AURA_TIRED);
    let calm = count_hits_filtered(lower, AURA_CALM);
    let energetic = count_hits_filtered(lower, AURA_ENERGETIC);
    let unconventional = count_hits_filtered(lower, AURA_UNCONVENTIONAL);
    let total = urgent + creative + happy + contemplative + analytical + tired + calm + energetic + unconventional;
    if total == 0 { return (Aura::NEUTRAL, 0.0); }
    let intensity = intensity_multiplier(lower);
    let (winner, hits) = [
        (Aura::URGENT, urgent), (Aura::CREATIVE, creative), (Aura::HAPPY, happy),
        (Aura::CONTEMPLATIVE, contemplative), (Aura::ANALYTICAL, analytical),
        (Aura::TIRED, tired), (Aura::CALM, calm), (Aura::ENERGETIC, energetic),
        (Aura::UNCONVENTIONAL, unconventional),
    ].iter().copied().max_by_key(|(_, h)| *h).unwrap();
    let confidence = (hits as f32 / total as f32 * intensity).min(1.0);
    (winner, confidence)
}

// ── Congruence Detection ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Valence { Positive, Negative, Neutral }

fn aura_valence(aura: &Aura) -> Valence {
    match aura.index() {
        160 => Valence::Negative, // Urgent
        238 => Valence::Negative, // Tired
        120 => Valence::Positive, // Creative
        220 => Valence::Positive, // Happy
        208 => Valence::Positive, // Energetic
        27  => Valence::Positive, // Calm
        _   => Valence::Neutral,
    }
}

fn container_valence(c: &Container) -> Valence {
    match c {
        Container::Sharp | Container::Defensive => Valence::Negative,
        Container::Fluid => Valence::Positive,
        _ => Valence::Neutral,
    }
}

fn focus_valence(f: &Focus) -> Valence {
    match f {
        Focus::Intense | Focus::Distant | Focus::Tired => Valence::Negative,
        Focus::Happy | Focus::Open => Valence::Positive,
        Focus::Neutral => Valence::Neutral,
    }
}

fn action_valence(a: &Action) -> Valence {
    match a {
        Action::Assertive | Action::Playful => Valence::Positive,
        Action::Hesitant => Valence::Negative,
        _ => Valence::Neutral,
    }
}

fn detect_congruence(state: &FacesState) -> Congruence {
    let valences = [
        aura_valence(&state.aura),
        container_valence(&state.container),
        focus_valence(&state.focus),
        action_valence(&state.action),
    ];
    let has_positive = valences.contains(&Valence::Positive);
    let has_negative = valences.contains(&Valence::Negative);
    if has_positive && has_negative {
        Congruence::Incongruent
    } else if has_positive || has_negative {
        Congruence::Congruent
    } else {
        Congruence::Neutral
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Action Detection Tests ────────────────────────────────────────────

    #[test]
    fn test_action_assertive_congratulations() {
        let state = detect_faces("Congratulations! Quest complete!");
        assert_eq!(state.action, Action::Assertive);
    }

    #[test]
    fn test_action_assertive_must() {
        let state = detect_faces("You must deploy this now.");
        assert_eq!(state.action, Action::Assertive);
    }

    #[test]
    fn test_action_playful_what_if() {
        let state = detect_faces("What if we tried something different?");
        assert_eq!(state.action, Action::Playful);
    }

    #[test]
    fn test_action_playful_creative() {
        let state = detect_faces("Let's brainstorm a creative approach.");
        assert_eq!(state.action, Action::Playful);
    }

    #[test]
    fn test_action_thoughtful_consider() {
        let state = detect_faces("Have you considered another approach?");
        assert_eq!(state.action, Action::Thoughtful);
    }

    #[test]
    fn test_action_thoughtful_reflect() {
        let state = detect_faces("Let's reflect on what we learned.");
        assert_eq!(state.action, Action::Thoughtful);
    }

    #[test]
    fn test_action_hesitant_maybe() {
        let state = detect_faces("Maybe this could work, but I'm not sure.");
        assert_eq!(state.action, Action::Hesitant);
    }

    #[test]
    fn test_action_hesitant_error() {
        let state = detect_faces("Error: failed to compile, retry needed.");
        assert_eq!(state.action, Action::Hesitant);
    }

    #[test]
    fn test_action_withheld_default() {
        let state = detect_faces("The lesson plan has three sections.");
        assert_eq!(state.action, Action::Withheld);
    }

    // ── Focus Detection Tests ─────────────────────────────────────────────

    #[test]
    fn test_focus_intense_critical() {
        let state = detect_faces("Critical: system failure detected!");
        assert_eq!(state.focus, Focus::Intense);
    }

    #[test]
    fn test_focus_open_surprising() {
        let state = detect_faces("Wow, that's an unexpected discovery!");
        assert_eq!(state.focus, Focus::Open);
    }

    #[test]
    fn test_focus_happy_success() {
        let state = detect_faces("Great job! Well done on the milestone!");
        assert_eq!(state.focus, Focus::Happy);
    }

    #[test]
    fn test_focus_distant_waiting() {
        let state = detect_faces("Waiting for background task to complete.");
        assert_eq!(state.focus, Focus::Distant);
    }

    #[test]
    fn test_focus_tired_exhausted() {
        let state = detect_faces("System exhausted, low memory, rate limited.");
        assert_eq!(state.focus, Focus::Tired);
    }

    #[test]
    fn test_focus_neutral_default() {
        let state = detect_faces("The document has been updated.");
        assert_eq!(state.focus, Focus::Neutral);
    }

    // ── Container Detection Tests ─────────────────────────────────────────

    #[test]
    fn test_container_sharp_emergency() {
        let state = detect_faces("Emergency: danger detected, high priority!");
        assert_eq!(state.container, Container::Sharp);
    }

    #[test]
    fn test_container_defensive_security() {
        let state = detect_faces("Security boundary: permission denied, unauthorized.");
        assert_eq!(state.container, Container::Defensive);
    }

    #[test]
    fn test_container_rigid_protocol() {
        let state = detect_faces("Protocol requires compliance with the standard.");
        assert_eq!(state.container, Container::Rigid);
    }

    #[test]
    fn test_container_fluid_creative() {
        let state = detect_faces("Let's explore a creative, flexible approach.");
        assert_eq!(state.container, Container::Fluid);
    }

    #[test]
    fn test_container_neutral_default() {
        let state = detect_faces("The file has been saved.");
        assert_eq!(state.container, Container::Neutral);
    }

    // ── Aura Detection Tests ──────────────────────────────────────────────

    #[test]
    fn test_aura_urgent_error() {
        let state = detect_faces("Critical error: system crash!");
        assert_eq!(state.aura, Aura::URGENT);
    }

    #[test]
    fn test_aura_creative_explore() {
        let state = detect_faces("Let's explore and build something novel.");
        assert_eq!(state.aura, Aura::CREATIVE);
    }

    #[test]
    fn test_aura_happy_success() {
        let state = detect_faces("Congratulations! Excellent success!");
        assert_eq!(state.aura, Aura::HAPPY);
    }

    #[test]
    fn test_aura_contemplative_reflect() {
        let state = detect_faces("Have you considered why this matters? Let's reflect.");
        assert_eq!(state.aura, Aura::CONTEMPLATIVE);
    }

    #[test]
    fn test_aura_neutral_default() {
        let state = detect_faces("The file has been saved.");
        assert_eq!(state.aura, Aura::NEUTRAL);
    }

    // ── Combined State Tests ──────────────────────────────────────────────

    #[test]
    fn test_combined_socratic_question() {
        // Socratic questioning should produce: contemplative + fluid + neutral + thoughtful
        let state = detect_faces("Have you considered perhaps there's a better way? Let's reflect.");
        assert_eq!(state.aura, Aura::CONTEMPLATIVE);
        assert_eq!(state.action, Action::Thoughtful);
    }

    #[test]
    fn test_combined_critical_error() {
        // Critical error should produce: urgent + sharp + intense + hesitant
        let state = detect_faces("Critical error: system crash, retry failed!");
        assert_eq!(state.aura, Aura::URGENT);
        assert_eq!(state.container, Container::Sharp);
        assert_eq!(state.focus, Focus::Intense);
        assert_eq!(state.action, Action::Hesitant);
    }

    #[test]
    fn test_combined_creative_brainstorm() {
        // Creative brainstorm should produce: creative + fluid + open + playful
        let state = detect_faces("What if we explore a creative, novel approach? Imagine!");
        assert_eq!(state.aura, Aura::CREATIVE);
        assert_eq!(state.container, Container::Fluid);
        assert_eq!(state.action, Action::Playful);
    }

    #[test]
    fn test_case_insensitive() {
        let upper = detect_faces("CONGRATULATIONS! QUEST COMPLETE!");
        let lower = detect_faces("congratulations! quest complete!");
        assert_eq!(upper, lower);
    }

    #[test]
    fn test_empty_string_returns_neutral() {
        let state = detect_faces("");
        assert_eq!(state, FacesState::neutral());
    }

    // ── Scored Detection Tests ────────────────────────────────────────────

    #[test]
    fn test_scored_returns_detection_result() {
        let result = detect_scored("Critical error: system crash!");
        assert_eq!(result.state.aura, Aura::URGENT);
        assert!(result.aura_confidence > 0.0);
        assert!(result.container_confidence > 0.0);
        assert!(result.focus_confidence > 0.0);
        assert!(result.action_confidence > 0.0);
        assert!(result.overall_confidence > 0.0);
        assert_eq!(result.method, DetectionMethod::Keyword);
    }

    #[test]
    fn test_scored_empty_string_zero_confidence() {
        let result = detect_scored("");
        assert_eq!(result.state, FacesState::neutral());
        assert_eq!(result.aura_confidence, 0.0);
        assert_eq!(result.container_confidence, 0.0);
        assert_eq!(result.focus_confidence, 0.0);
        assert_eq!(result.action_confidence, 0.0);
        assert_eq!(result.overall_confidence, 0.0);
        assert_eq!(result.congruence, Congruence::Neutral);
    }

    #[test]
    fn test_scored_confidence_range() {
        let texts = [
            "Critical error crash!",
            "Congratulations success milestone!",
            "Let's explore and build something novel.",
            "Waiting for background task.",
        ];
        for text in &texts {
            let result = detect_scored(text);
            assert!(result.aura_confidence >= 0.0 && result.aura_confidence <= 1.0,
                "aura_confidence out of range for '{}': {}", text, result.aura_confidence);
            assert!(result.container_confidence >= 0.0 && result.container_confidence <= 1.0,
                "container_confidence out of range for '{}': {}", text, result.container_confidence);
            assert!(result.focus_confidence >= 0.0 && result.focus_confidence <= 1.0,
                "focus_confidence out of range for '{}': {}", text, result.focus_confidence);
            assert!(result.action_confidence >= 0.0 && result.action_confidence <= 1.0,
                "action_confidence out of range for '{}': {}", text, result.action_confidence);
        }
    }

    #[test]
    fn test_scored_multi_keyword_higher_confidence() {
        // Use competing aura categories so confidence < 1.0
        // Single hit in urgent vs one in happy → 0.5 confidence
        let single = detect_scored("critical happy");
        // Multiple hits in urgent vs one in happy → 0.75 confidence
        let multi = detect_scored("critical error crash happy");
        assert!(multi.aura_confidence > single.aura_confidence,
            "Multiple keywords should yield higher confidence: {} vs {}",
            multi.aura_confidence, single.aura_confidence);
    }

    // ── Congruence Tests ──────────────────────────────────────────────────

    #[test]
    fn test_congruence_congruent_critical_error() {
        // Urgent + Sharp + Intense + Hesitant = all negative valence = Congruent
        let result = detect_scored("Critical error: system crash, retry failed!");
        assert_eq!(result.congruence, Congruence::Congruent);
    }

    #[test]
    fn test_congruence_congruent_creative_brainstorm() {
        // Creative + Fluid + (Open or Neutral) + Playful = positive/neutral = Congruent
        let result = detect_scored("What if we explore a creative, novel approach? Imagine!");
        assert_eq!(result.congruence, Congruence::Congruent);
    }

    #[test]
    fn test_congruence_neutral_default() {
        let result = detect_scored("The file has been saved.");
        assert_eq!(result.congruence, Congruence::Neutral);
    }

    #[test]
    fn test_congruence_incongruent_mixed_signals() {
        // Construct a state with mixed valences: Happy (positive) + Sharp (negative)
        let state = FacesState::new(Aura::HAPPY, Container::Sharp, Focus::Neutral, Action::Withheld);
        let congruence = detect_congruence(&state);
        assert_eq!(congruence, Congruence::Incongruent);
    }

    #[test]
    fn test_congruence_incongruent_tired_but_assertive() {
        // Tired (negative) + Assertive (positive) = Incongruent
        let state = FacesState::new(Aura::TIRED, Container::Neutral, Focus::Neutral, Action::Assertive);
        let congruence = detect_congruence(&state);
        assert_eq!(congruence, Congruence::Incongruent);
    }

    // ── Intensity Modifier Tests ──────────────────────────────────────────

    #[test]
    fn test_intensity_booster_increases_confidence() {
        // Use competing categories so confidence < 1.0, allowing booster to increase it
        let plain = detect_scored("critical happy");
        let boosted = detect_scored("very critical happy");
        assert!(boosted.aura_confidence > plain.aura_confidence,
            "Booster should increase confidence: {} vs {}",
            boosted.aura_confidence, plain.aura_confidence);
    }

    #[test]
    fn test_intensity_diminisher_decreases_confidence() {
        let plain = detect_scored("critical error");
        let diminished = detect_scored("slightly critical error");
        assert!(diminished.aura_confidence < plain.aura_confidence,
            "Diminisher should decrease confidence: {} vs {}",
            diminished.aura_confidence, plain.aura_confidence);
    }

    #[test]
    fn test_intensity_booster_caps_at_one() {
        let result = detect_scored("extremely critical urgent error crash danger");
        assert!(result.aura_confidence <= 1.0,
            "Confidence should cap at 1.0: {}", result.aura_confidence);
    }

    // ── Negation Tests ────────────────────────────────────────────────────

    #[test]
    fn test_negation_reduces_hits() {
        // "not critical" should not count "critical" as a hit
        let negated = detect_scored("This is not critical at all.");
        let plain = detect_scored("This is critical.");
        // With negation, "critical" should be filtered out, reducing confidence
        assert!(negated.aura_confidence < plain.aura_confidence,
            "Negated keyword should have lower confidence: {} vs {}",
            negated.aura_confidence, plain.aura_confidence);
    }

    #[test]
    fn test_negation_no_signal_when_all_negated() {
        // If all keywords are negated, should return neutral with 0.0 confidence
        let result = detect_scored("This is not critical, not urgent, not an error.");
        assert_eq!(result.aura_confidence, 0.0,
            "All negated keywords should yield 0.0 confidence");
    }

    // ── Backward Compatibility Tests ──────────────────────────────────────

    #[test]
    fn test_detect_faces_wrapper_matches_scored_state() {
        let texts = [
            "Congratulations! Quest complete!",
            "Critical error: system crash!",
            "What if we explore a creative approach?",
            "The file has been saved.",
        ];
        for text in &texts {
            let faces_state = detect_faces(text);
            let scored_state = detect_scored(text).state;
            assert_eq!(faces_state, scored_state,
                "detect_faces() should match detect_scored().state for '{}'", text);
        }
    }

    // ── Expanded Keyword Coverage Tests ───────────────────────────────────

    #[test]
    fn test_aura_energetic_detection() {
        let state = detect_faces("I'm feeling energetic and enthusiastic today!");
        assert_eq!(state.aura, Aura::ENERGETIC);
    }

    #[test]
    fn test_aura_unconventional_detection() {
        let state = detect_faces("That's a weird and bizarre approach.");
        assert_eq!(state.aura, Aura::UNCONVENTIONAL);
    }

    #[test]
    fn test_aura_calm_detection() {
        let state = detect_faces("Everything is calm and peaceful, steady and stable.");
        assert_eq!(state.aura, Aura::CALM);
    }

    #[test]
    fn test_focus_open_breakthrough() {
        let result = detect_scored("Breakthrough! We discovered something fascinating!");
        assert_eq!(result.state.focus, Focus::Open);
        assert!(result.focus_confidence > 0.0);
    }

    // ── Multi-Sentence Detection Tests ───────────────────────────────────

    #[test]
    fn test_detect_multi_two_sentences() {
        let results = detect_multi("Critical error! Let's brainstorm a fix.");
        assert_eq!(results.len(), 2);
        assert!(results[0].aura_confidence > 0.0);
        assert!(results[1].aura_confidence > 0.0);
    }

    #[test]
    fn test_detect_multi_single_sentence() {
        let results = detect_multi("Just one sentence here");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_detect_multi_empty() {
        let results = detect_multi("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_detect_multi_no_delimiters() {
        let results = detect_multi("no punctuation at all here");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_detect_multi_each_sentence_has_result() {
        let results = detect_multi("Critical crash! Build it now. Maybe later?");
        assert_eq!(results.len(), 3);
        for r in &results {
            assert!(r.overall_confidence >= 0.0);
        }
    }

    #[test]
    fn test_detect_aggregate_single_sentence() {
        let single = detect_scored("Critical error crash!");
        let agg = detect_aggregate("Critical error crash!");
        assert_eq!(agg.state, single.state);
    }

    #[test]
    fn test_detect_aggregate_empty() {
        let result = detect_aggregate("");
        assert_eq!(result.state, FacesState::neutral());
        assert_eq!(result.overall_confidence, 0.0);
    }

    #[test]
    fn test_detect_aggregate_multiple_sentences() {
        let result = detect_aggregate("Critical error! We must fix this. Deploy now!");
        assert!(result.overall_confidence > 0.0);
    }

    #[test]
    fn test_detect_aggregate_recency_bias() {
        // Last sentence should have more influence on the final state
        let result = detect_aggregate("Everything is calm and peaceful. Critical error crash!");
        // The urgent sentence is last (higher weight), so aura should lean urgent
        assert_eq!(result.state.aura, Aura::URGENT);
    }

    #[test]
    fn test_detect_aggregate_agreement_boost() {
        // All three sentences have urgent keywords → agreement boost
        let result = detect_aggregate("Critical error! System crash! Danger alert!");
        let single = detect_scored("Critical error crash danger alert");
        assert!(result.aura_confidence >= single.aura_confidence,
            "Agreement across sentences should boost confidence: {} vs {}",
            result.aura_confidence, single.aura_confidence);
    }

    #[test]
    fn test_detect_aggregate_mixed_sentences() {
        let result = detect_aggregate("Everything is calm. What if we explore creatively?");
        // Should produce a valid state, not crash
        assert!(result.overall_confidence >= 0.0);
    }

    #[test]
    fn test_detect_multi_preserves_per_sentence_confidence() {
        let results = detect_multi("Critical error crash! The file is saved.");
        assert!(results[0].aura_confidence > 0.0, "First sentence should have signal");
        assert_eq!(results[1].aura_confidence, 0.0, "Second sentence should have no aura signal");
    }
}
