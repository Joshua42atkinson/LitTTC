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
    pub companion: bool,
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
    ) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.word == word) {
            entry.times_encountered += 1;
            // Fill in missing pet metadata if now known.
            if entry.element.is_none() { entry.element = element; }
            if entry.role.is_none() { entry.role = role; }
            if entry.stats.is_none() { entry.stats = stats; }
        } else {
            self.entries.push(SpellBookEntry {
                word: word.to_string(),
                channel,
                mastery: MasteryLevel::Encountered,
                times_encountered: 1,
                element,
                role,
                stats,
                companion: false,
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
        book.record_encounter("clarity", Channel::Mind, None, None, None);
        book.record_encounter("clarity", Channel::Mind, None, None, None);
        assert_eq!(book.entries.len(), 1);
        assert_eq!(book.entries[0].times_encountered, 2);
    }

    #[test]
    fn spellbook_upgrade_mastery_only_increases() {
        let mut book = SpellBook::default();
        book.record_encounter("clarity", Channel::Mind, None, None, None);
        book.upgrade_mastery("clarity", MasteryLevel::Owned);
        assert_eq!(book.entries[0].mastery, MasteryLevel::Owned);
        book.upgrade_mastery("clarity", MasteryLevel::Encountered);
        assert_eq!(book.entries[0].mastery, MasteryLevel::Owned);
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

/// Slime face/emotion states
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SlimeFace {
    #[default]
    Fierce,
    Joyful,
    Calm,
    Angry,
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

/// Active Slime face for combat
#[derive(Resource, Default)]
pub struct ActiveFace {
    pub face: SlimeFace,
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
    pub face: SlimeFace,
}

/// Event when face is changed
#[derive(Event)]
pub struct FaceChanged {
    pub new_face: SlimeFace,
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
