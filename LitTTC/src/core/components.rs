// components.rs — ECS Components and Resources for LitTCG
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use faces_protocol::FacesState;

// ─── SHARED WORLD CONSTANTS ─────────────────────────────────────

/// Where pets (and reveal cards) appear in front of the player.
pub const PET_SPAWN_POSITION: Vec3 = Vec3::new(0.0, 1.5, -2.0);

// ─── THE FOUR CHANNELS (Card Element Types) ─────────────────────

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Channel {
    Mind,
    Heart,
    Body,
    Action,
}


// ─── SPELL POWER (Word Mastery Tracking) ────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MasteryLevel {
    Encountered,
    Experienced,
    Owned,
    Mastered,
}


// ─── CHARACTER SHEET ─────────────────────────────────────────────

/// Global Slime progression resource.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default)]
pub struct SlimeLevel {
    /// Total XP accumulated across battles and quests.
    pub xp: u32,
    /// Current level derived from xp.
    pub level: u32,
    /// Visual evolution stage (0–3).
    pub evolution_stage: u32,
}

impl SlimeLevel {
    /// XP required to reach the next level follows a gentle curve.
    pub fn xp_for_level(level: u32) -> u32 {
        100 * level * level
    }

    /// Recompute `self.level` and `self.evolution_stage` from `self.xp`.
    pub fn recalc(&mut self) {
        let mut level = 1u32;
        loop {
            let needed = Self::xp_for_level(level);
            if self.xp >= needed {
                level += 1;
            } else {
                break;
            }
        }
        self.level = level;
        self.evolution_stage = ((self.level - 1) / 3).min(3);
    }

    /// Add XP and recalculate level/stage.
    pub fn add_xp(&mut self, amount: u32) {
        self.xp = self.xp.saturating_add(amount);
        self.recalc();
    }
}

/// Three-axis grade for a cast or quest completion.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct GradeScores {
    /// Part-of-speech / structural fit (0.0–1.0).
    pub syntax: f32,
    /// Synonym/antonym semantic distance (0.0–1.0).
    pub semantics: f32,
    /// FACES emotional resonance (0.0–1.0).
    pub pragmatics: f32,
}

/// A single evidence event recorded when a word is cast in battle or placed in a quest slot.
///
/// This is the atomic unit of the Evidence-Centered Design (ECD) evidence model. It carries
/// everything needed to recompute lexical diversity, syntactic complexity, and pragmatic
/// flexibility without storing the full game state.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CastTelemetry {
    /// Lowercased word that was played.
    pub word: String,
    /// Part of speech for the word, if known.
    pub pos: Option<String>,
    /// Three-axis grade produced by this cast.
    pub grades: GradeScores,
    /// FACES resonance between the word's intrinsic face and the active/environmental face.
    pub faces_resonance: f32,
    /// Whether the cast was effective (dealt damage / filled a valid slot).
    pub effective: bool,
    /// Whether this cast used a literary-device combo (Echo, Armor Piercing, Overcharge).
    pub combo: bool,
    /// Name of the literary device, if any.
    pub device: Option<String>,
    /// CCSS standard codes demonstrated by this cast, if any.
    pub ccss_tags: Vec<String>,
    /// Language subject demonstrated, if this cast was tied to an NPC scenario.
    pub subject: Option<String>,
    /// UTC timestamp placeholder; serializable without chrono dependency.
    pub sequence: u64,
}

/// Common Core State Standards ELA codes mapped by the stealth assessment system.
pub mod ccss {
    /// Figurative language, word relationships, and nuances in word meanings.
    pub const L_9_10_5: &str = "L.9-10.5";
    /// Apply knowledge of language to understand how language functions in different contexts.
    pub const L_9_10_3: &str = "L.9-10.3";
    /// Determine or clarify the meaning of unknown and multiple-meaning words and phrases.
    pub const L_9_10_4: &str = "L.9-10.4";
    /// Apply knowledge of language to make effective choices for meaning or style.
    pub const L_11_12_3: &str = "L.11-12.3";
}

/// Snapshot of lexical diversity at a point in time.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct LexicalDiversitySnapshot {
    /// Hypergeometric Distribution D score.
    pub hd_d: f32,
    /// Measure of Textual Lexical Diversity score.
    pub mtld: f32,
    /// Number of tokens in the rolling window used for the calculation.
    pub token_count: u32,
}

/// Temporal evidence series used for institutional dashboard reports.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetrySeries {
    /// Per-cast three-axis grades over time.
    pub syntax_series: Vec<f32>,
    pub semantics_series: Vec<f32>,
    pub pragmatics_series: Vec<f32>,
    /// Lexical diversity (HD-D / MTLD) over time.
    pub lexical_diversity_series: Vec<LexicalDiversitySnapshot>,
    /// Syntactic complexity ratio over time (combo casts / total casts).
    pub syntactic_complexity_series: Vec<f32>,
    /// Raw cast events for recomputation or audit.
    pub cast_log: Vec<CastTelemetry>,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSheet {
    pub mind_attunement: f32,
    pub heart_attunement: f32,
    pub body_attunement: f32,
    pub action_attunement: f32,
    pub emergent_class: String,
    pub words_encountered: u32,
    pub total_deeper_swipes: u32,
    pub total_xp: u64,
    #[serde(default = "default_summon_class")]
    pub active_summon_class: SummonClass,
    #[serde(default = "default_arm_length")]
    pub arm_length: f32,
    #[serde(default)]
    pub last_grades: GradeScores,
    #[serde(default)]
    pub telemetry: TelemetrySeries,
}

fn default_arm_length() -> f32 {
    0.65
}

fn default_summon_class() -> SummonClass {
    SummonClass::SemanticSlime
}

impl Default for CharacterSheet {
    fn default() -> Self {
        Self {
            mind_attunement: 0.0,
            heart_attunement: 0.0,
            body_attunement: 0.0,
            action_attunement: 0.0,
            emergent_class: "Newcomer".to_string(),
            words_encountered: 0,
            total_deeper_swipes: 0,
            total_xp: 0,
            active_summon_class: SummonClass::SemanticSlime,
            arm_length: 0.65,
            last_grades: GradeScores::default(),
            telemetry: TelemetrySeries::default(),
        }
    }
}

impl CharacterSheet {
    pub fn engage_channel(&mut self, channel: &Channel) {
        let bump = 0.1; // Asymptotic bump multiplier
        match channel {
            Channel::Mind   => self.mind_attunement += (1.0 - self.mind_attunement) * bump,
            Channel::Heart  => self.heart_attunement += (1.0 - self.heart_attunement) * bump,
            Channel::Body   => self.body_attunement += (1.0 - self.body_attunement) * bump,
            Channel::Action => self.action_attunement += (1.0 - self.action_attunement) * bump,
        }
        self.update_class();
    }

    fn update_class(&mut self) {
        let scores = [
            (self.mind_attunement,   "Mind"),
            (self.heart_attunement,  "Heart"),
            (self.body_attunement,   "Body"),
            (self.action_attunement, "Action"),
        ];

        let max_score = scores.iter().map(|s| s.0).fold(0.0, f32::max);
        
        // Require a minimum attunement threshold before manifesting an emergent class
        if max_score < 0.2 {
            self.emergent_class = "Newcomer".to_string();
            return;
        }

        let dominant = scores.iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|s| s.1)
            .unwrap_or("Mind");

        self.emergent_class = match dominant {
            "Mind"   => "The Oracle".to_string(),
            "Heart"  => "The Bard".to_string(),
            "Body"   => "The Cultivator".to_string(),
            "Action" => "The Templar".to_string(),
            _        => "The Architect".to_string(),
        };
    }
}

// ─── SPELL BOOK (Word Collection) ────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpellBookEntry {
    pub word: String,
    pub channel: Channel,
    pub mastery: MasteryLevel,
    pub times_encountered: u32,
    pub element: Option<Element>,
    pub role: Option<Role>,
    pub stats: Option<PetStats>,
    pub faces: Option<PetFacesState>,
    pub companion: bool,
    /// XP earned by successfully casting this word in battle or quests.
    #[serde(default)]
    pub card_xp: u32,
}

impl SpellBookEntry {
    /// Map accumulated card XP to a mastery tier.
    pub fn mastery_from_xp(xp: u32) -> MasteryLevel {
        match xp {
            0..=9 => MasteryLevel::Encountered,
            10..=49 => MasteryLevel::Experienced,
            50..=99 => MasteryLevel::Owned,
            _ => MasteryLevel::Mastered,
        }
    }

    /// Recompute this entry's mastery tier from its card_xp.
    pub fn recalc_mastery(&mut self) {
        self.mastery = Self::mastery_from_xp(self.card_xp);
    }

    /// Add card XP and update the mastery tier.
    pub fn add_card_xp(&mut self, amount: u32) {
        self.card_xp = self.card_xp.saturating_add(amount);
        self.recalc_mastery();
    }
}

#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpellBook {
    pub entries: Vec<SpellBookEntry>,
}

// ─── FACES PROTOCOL COMPONENT WRAPPER ────────────────────────────

#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PetFacesState(pub FacesState);

impl SpellBook {
    pub fn record_encounter(
        &mut self,
        word: &str,
        channel: Channel,
        element: Option<Element>,
        role: Option<Role>,
        stats: Option<PetStats>,
        faces: Option<PetFacesState>,
    ) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.word == word) {
            entry.times_encountered += 1;
            // Fill in missing pet metadata if now known.
            if entry.element.is_none() { entry.element = element; }
            if entry.role.is_none() { entry.role = role; }
            if entry.stats.is_none() { entry.stats = stats; }
            if entry.faces.is_none() { entry.faces = faces; }
        } else {
            self.entries.push(SpellBookEntry {
                word: word.to_string(),
                channel,
                mastery: MasteryLevel::Encountered,
                times_encountered: 1,
                element,
                role,
                stats,
                faces,
                companion: false,
                card_xp: 0,
            });
        }
    }

    pub fn upgrade_mastery(&mut self, word: &str, new_level: MasteryLevel) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.word == word) {
            if new_level > entry.mastery {
                entry.mastery = new_level;
            }
        }
    }

    /// Award XP to a word card and recalculate its mastery tier.
    pub fn add_card_xp(&mut self, word: &str, amount: u32) {
        let lower = word.to_lowercase();
        if let Some(entry) = self.entries.iter_mut().find(|e| e.word == lower) {
            entry.add_card_xp(amount);
            info!("Card XP for '{}': {} -> mastery {:?}", word, entry.card_xp, entry.mastery);
        } else {
            warn!("Cannot award card XP: '{}' is not in the SpellBook", word);
        }
    }
}

// ─── WORD TRAIL ───────────────────────────────────────────────

#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize)]
pub struct WordTrail {
    pub visited_words: Vec<String>,
    pub swipe_history: Vec<SwipeChoice>,
    pub current_word: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum SwipeChoice {
    Yes,
    No,
    Deeper,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct CurrentSlide {
    pub ready_for_input: bool,
    pub depth_showing: bool,
}

// ─── TCG DECK / HAND / DISCARD ──────────────────────────────

/// The Grimoire — SemanticSlime acts as the player's physical inventory/deck
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Grimoire {
    pub words: Vec<String>, // Words stored in the Slime's grimoire
    pub max_capacity: usize,
}

impl Default for Grimoire {
    fn default() -> Self {
        Self {
            words: Vec::new(),
            max_capacity: 50,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct Deck {
    pub cards: Vec<String>, // Words in the deck
    pub active_summon_class: Option<SummonClass>,
}

#[derive(Resource, Debug)]
pub struct Hand {
    pub cards: Vec<String>,
    pub max_size: usize,
    pub selected: Option<usize>,
}

impl Default for Hand {
    fn default() -> Self {
        Self {
            cards: Vec::new(),
            max_size: 3,
            selected: None,
        }
    }
}

#[derive(Resource, Default, Debug)]
pub struct DiscardPile;

// ─── PET COMPONENTS ──────────────────────────────────────────────

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummonClass {
    SemanticSlime,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct TimeScale(pub f32);

impl Default for TimeScale {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct UnstableWord {
    pub health: f32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Element {
    Fire,
    Water,
    Earth,
    Air,
    Shadow,
    Light,
    Normal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_sheet_default_is_newcomer() {
        let sheet = CharacterSheet::default();
        assert_eq!(sheet.emergent_class, "Newcomer");
        assert_eq!(sheet.total_xp, 0);
        assert_eq!(sheet.arm_length, 0.65);
    }

    #[test]
    fn engage_channel_bumps_attunement_asymptotically() {
        let mut sheet = CharacterSheet::default();
        let before = sheet.mind_attunement;
        sheet.engage_channel(&Channel::Mind);
        assert!(sheet.mind_attunement > before);
        assert!(sheet.mind_attunement < 1.0);
    }

    #[test]
    fn repeated_mind_channel_manifests_oracle() {
        let mut sheet = CharacterSheet::default();
        for _ in 0..10 {
            sheet.engage_channel(&Channel::Mind);
        }
        assert_eq!(sheet.emergent_class, "The Oracle");
    }

    #[test]
    fn spellbook_records_encounter_once() {
        let mut book = SpellBook::default();
        book.record_encounter("clarity", Channel::Mind, None, None, None, None);
        book.record_encounter("clarity", Channel::Mind, None, None, None, None);
        assert_eq!(book.entries.len(), 1);
        assert_eq!(book.entries[0].times_encountered, 2);
    }

    #[test]
    fn spellbook_upgrade_mastery_only_increases() {
        let mut book = SpellBook::default();
        book.record_encounter("clarity", Channel::Mind, None, None, None, None);
        book.upgrade_mastery("clarity", MasteryLevel::Owned);
        assert_eq!(book.entries[0].mastery, MasteryLevel::Owned);
        book.upgrade_mastery("clarity", MasteryLevel::Encountered);
        assert_eq!(book.entries[0].mastery, MasteryLevel::Owned);
    }

    #[test]
    fn spellbook_records_faces() {
        let faces = PetFacesState(FacesState::default());
        let mut book = SpellBook::default();
        book.record_encounter("clarity", Channel::Mind, None, None, None, Some(faces));
        assert!(book.entries[0].faces.is_some());
    }

    #[test]
    fn slime_face_presets_map_to_distinct_faces_states() {
        let fierce = SlimeFace::Fierce.to_faces_state();
        let joyful = SlimeFace::Joyful.to_faces_state();
        let calm = SlimeFace::Calm.to_faces_state();
        let angry = SlimeFace::Angry.to_faces_state();

        assert_ne!(fierce, joyful);
        assert_ne!(fierce, calm);
        assert_ne!(fierce, angry);
        assert_ne!(joyful, calm);
        assert_ne!(joyful, angry);
        assert_ne!(calm, angry);
    }

    #[test]
    fn word_trail_defaults_empty() {
        let trail = WordTrail::default();
        assert!(trail.visited_words.is_empty());
        assert!(trail.swipe_history.is_empty());
        assert!(trail.current_word.is_none());
    }
}

impl Element {
    #[cfg(feature = "flat2d")]
    pub fn color(&self) -> Color {
        // NES-style 8-bit palette for the 2D pixel-art look.
        match self {
            Element::Fire => Color::srgb(0.90, 0.36, 0.04),
            Element::Water => Color::srgb(0.24, 0.74, 0.99),
            Element::Earth => Color::srgb(0.42, 0.39, 0.31),
            Element::Air => Color::srgb(0.69, 0.94, 0.99),
            Element::Shadow => Color::srgb(0.27, 0.00, 0.63),
            Element::Light => Color::srgb(0.99, 0.90, 0.64),
            Element::Normal => Color::srgb(0.72, 0.72, 0.72),
        }
    }

    #[cfg(not(feature = "flat2d"))]
    pub fn color(&self) -> Color {
        match self {
            Element::Fire => Color::srgb(0.94, 0.27, 0.27),
            Element::Water => Color::srgb(0.23, 0.51, 0.96),
            Element::Earth => Color::srgb(0.13, 0.77, 0.37),
            Element::Air => Color::srgb(0.79, 0.54, 0.02),
            Element::Shadow => Color::srgb(0.42, 0.13, 0.66),
            Element::Light => Color::srgb(0.96, 0.62, 0.04),
            Element::Normal => Color::srgb(0.66, 0.64, 0.62),
        }
    }

    /// Returns a distinct PBR material preset for each element (3D only).
    #[cfg(not(feature = "flat2d"))]
    pub fn material(&self) -> StandardMaterial {
        let color = self.color();
        let (emissive_mult, metallic, roughness) = match self {
            Element::Fire => (1.5, 0.1, 0.9),    // glowing, matte
            Element::Water => (0.6, 0.9, 0.1),   // glossy, reflective
            Element::Earth => (0.2, 0.3, 0.95),  // dull stone
            Element::Air => (0.9, 0.2, 0.3),     // bright, silky
            Element::Shadow => (1.2, 0.8, 0.2),  // dark sheen
            Element::Light => (1.8, 0.1, 0.2),   // intense glow
            Element::Normal => (0.1, 0.5, 0.6),    // plain
        };
        StandardMaterial {
            base_color: color,
            emissive: (color.to_srgba() * emissive_mult).into(),
            metallic,
            perceptual_roughness: roughness,
            ..default()
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Tank,
    Bruiser,
    Striker,
    Assassin,
    Caster,
    Support,
    Buffer,
    Healer,
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PetStats {
    pub logos: f32, // Attack
    pub pathos: f32, // Health
    pub ethos: f32, // Defense
    pub speed: f32, // Speed/Intellect
}

/// Marks a 3D pet entity in Bevy
#[derive(Component)]
pub struct PetAvatar {
    pub word: String,
    pub pet_type: SummonClass,
}

#[cfg(not(feature = "flat2d"))]
#[derive(Component)]
pub struct AvatarAnimation {
    pub time: f32,
    pub base_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
pub enum PetVisualState {
    #[default]
    Idle,
    Alert,
    Battle,
    Happy,
}

#[cfg(not(feature = "flat2d"))]
#[derive(Component)]
pub struct OrbitalRing;

// ─── GAME STATE ──────────────────────────────────────────────────

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Collecting,
    Constructing,
    Playing,
    Exploring,
    Questing,
    Battling,
    Reviewing,
    Paywall,
    Settings,
    Difficulty,
    PetCollection,
    RevealingPet,
}

// ─── 2D PROTOTYPE STATE & RESOURCES ──────────────────────────────

#[derive(Resource, Default)]
pub struct ActiveGestures {
    pub traces: std::collections::HashMap<u64, Vec<bevy::math::Vec2>>,
}

#[derive(Component)]
pub struct DraggableCard {
    pub touch_id: Option<u64>,
}

// ─── PHASE 1: 2D GRAY-BOX COMBAT COMPONENTS ────────────────────────

/// Target Dummy for Phase 1 combat testing
#[derive(Component)]
pub struct TargetDummy {
    pub hp: i32,
    pub max_hp: i32,
    pub prompt_word: String,
}

/// Hand card button for 2D UI
#[derive(Component)]
pub struct HandCardButton {
    pub word: String,
    pub part_of_speech: crate::deck::PartOfSpeech,
    pub index: usize,
}

/// Altar drop-zone for spell casting
#[derive(Component)]
pub struct AltarDropZone {
    pub active_card: Option<String>,
}

/// Cast spell button
#[derive(Component)]
pub struct CastSpellButton;

/// Face/emotion button for Slime
#[derive(Component)]
pub struct FaceButton {
    pub face: SlimeFace,
}

/// Marker for a button that removes a card from the battle Plot.
#[derive(Component)]
pub struct PlotSlotButton {
    pub index: usize,
}

/// Marker for the 2D battle panel that shows the active Plot sentence.
#[derive(Component)]
pub struct PlotPanelText;

/// Button that removes the last card from the battle Plot.
#[derive(Component)]
pub struct ClearPlotButton;

/// Slime face/emotion states
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SlimeFace {
    #[default]
    Fierce,
    Joyful,
    Calm,
    Angry,
}

impl SlimeFace {
    /// Returns a UI color that represents this emotion.
    pub fn color(&self) -> Color {
        match self {
            SlimeFace::Fierce => Color::srgb(1.0, 0.55, 0.0),   // orange
            SlimeFace::Joyful => Color::srgb(1.0, 0.85, 0.2),   // yellow
            SlimeFace::Calm => Color::srgb(0.3, 0.7, 1.0),       // blue
            SlimeFace::Angry => Color::srgb(0.9, 0.2, 0.2),    // red
        }
    }

    /// Maps the 2D demo preset to a full 32-bit FACES register.
    pub fn to_faces_state(&self) -> FacesState {
        match self {
            SlimeFace::Fierce => FacesState::new(
                faces_protocol::Aura::ENERGETIC,
                faces_protocol::Container::Sharp,
                faces_protocol::Focus::Intense,
                faces_protocol::Action::Assertive,
            ),
            SlimeFace::Joyful => FacesState::new(
                faces_protocol::Aura::HAPPY,
                faces_protocol::Container::Fluid,
                faces_protocol::Focus::Happy,
                faces_protocol::Action::Playful,
            ),
            SlimeFace::Calm => FacesState::new(
                faces_protocol::Aura::CALM,
                faces_protocol::Container::Neutral,
                faces_protocol::Focus::Neutral,
                faces_protocol::Action::Thoughtful,
            ),
            SlimeFace::Angry => FacesState::new(
                faces_protocol::Aura::URGENT,
                faces_protocol::Container::Sharp,
                faces_protocol::Focus::Intense,
                faces_protocol::Action::Assertive,
            ),
        }
    }
}

/// Combat log entry
#[derive(Component)]
pub struct CombatLogEntry {
    pub text: String,
    pub log_type: LogType,
}

/// Log type for color coding
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum LogType {
    Effective,   // green
    Ineffective, // red
    Critical,    // gold
    Info,        // white
}

// ─── PHASE 1: RESOURCES ─────────────────────────────────────────────

/// Active Slime face for combat.
///
/// `face` is the 2D demo UI preset (Fierce, Joyful, Calm, Angry).
/// `faces` is the full 32-bit FACES register used for resonance math.
#[derive(Resource, Default)]
pub struct ActiveFace {
    pub face: SlimeFace,
    pub faces: FacesState,
}

/// Gray-box battle session
#[derive(Resource)]
pub struct GrayBoxBattleSession {
    pub dummy_hp: i32,
    pub player_hp: i32,
    pub prompt_word: String,
    pub turn: BattleTurn,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BattleTurn {
    Player,
    Enemy,
}

// ─── PHASE 1: EVENTS ───────────────────────────────────────────────

/// Event when card is played to altar
#[derive(Event)]
pub struct CardPlayedToAltar {
    pub word: String,
}

/// Event when spell is cast
#[derive(Event)]
pub struct SpellCast {
    pub word: String,
    pub faces: FacesState,
}

/// Event when face is changed
#[derive(Event)]
pub struct FaceChanged {
    pub new_face: SlimeFace,
    pub new_faces: FacesState,
}

/// Event for battle actions with damage math
#[derive(Event)]
pub struct BattleAction {
    pub action_type: BattleActionType,
    pub damage: i32,
    pub math_breakdown: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BattleActionType {
    SynonymAttack,
    AntonymBlock,
    NormalAttack,
    EnemyAttack,
}
