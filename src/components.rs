// components.rs — ECS Components and Resources for Communication Class
#![allow(dead_code)]
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use faces_protocol::FacesState;

// ─── THE FOUR CHANNELS (Card Element Types) ─────────────────────

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Channel {
    Mind,
    Heart,
    Body,
    Action,
}

impl Channel {
    pub fn color(&self) -> Color {
        match self {
            Channel::Mind   => Color::srgba(0.29, 0.62, 0.43, 1.0),
            Channel::Heart  => Color::srgba(0.83, 0.47, 0.24, 1.0),
            Channel::Body   => Color::srgba(0.29, 0.49, 0.71, 1.0),
            Channel::Action => Color::srgba(0.77, 0.64, 0.24, 1.0),
        }
    }

    pub fn background_color(&self) -> Color {
        match self {
            Channel::Mind   => Color::srgba(0.04, 0.09, 0.06, 1.0),
            Channel::Heart  => Color::srgba(0.12, 0.07, 0.04, 1.0),
            Channel::Body   => Color::srgba(0.04, 0.07, 0.11, 1.0),
            Channel::Action => Color::srgba(0.12, 0.10, 0.04, 1.0),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Channel::Mind   => "Mind",
            Channel::Heart  => "Heart",
            Channel::Body   => "Body",
            Channel::Action => "Action",
        }
    }
}

// ─── SPELL POWER (Word Mastery Tracking) ────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MasteryLevel {
    Encountered,
    Experienced,
    Owned,
    Mastered,
}

impl MasteryLevel {
    pub fn icon(&self) -> &'static str {
        match self {
            MasteryLevel::Encountered => "🔮",
            MasteryLevel::Experienced => "⚡",
            MasteryLevel::Owned       => "🌟",
            MasteryLevel::Mastered    => "👑",
        }
    }
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
        }
    }
}

impl CharacterSheet {
    pub fn engage_channel(&mut self, channel: &Channel) {
        let bump = 0.05;
        match channel {
            Channel::Mind   => self.mind_attunement = (self.mind_attunement + bump).min(1.0),
            Channel::Heart  => self.heart_attunement = (self.heart_attunement + bump).min(1.0),
            Channel::Body   => self.body_attunement = (self.body_attunement + bump).min(1.0),
            Channel::Action => self.action_attunement = (self.action_attunement + bump).min(1.0),
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
}

#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpellBook {
    pub entries: Vec<SpellBookEntry>,
}

// ─── FACES PROTOCOL COMPONENT WRAPPER ────────────────────────────

#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PetFacesState(pub FacesState);

impl SpellBook {
    pub fn record_encounter(&mut self, word: &str, channel: Channel) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.word == word) {
            entry.times_encountered += 1;
        } else {
            self.entries.push(SpellBookEntry {
                word: word.to_string(),
                channel,
                mastery: MasteryLevel::Encountered,
                times_encountered: 1,
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

// ─── STUDENT TRAIL ───────────────────────────────────────────────

#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize)]
pub struct StudentTrail {
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
    pub story_text: String,
    pub setting_mood: String,
    pub ready_for_input: bool,
    pub depth_showing: bool,
}

// ─── TCG DECK / HAND / DISCARD ──────────────────────────────

#[derive(Resource, Debug, Default)]
pub struct Deck {
    pub cards: Vec<String>, // Words in the deck
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
pub struct DiscardPile {
    pub cards: Vec<String>,
}

// ─── PET COMPONENTS ──────────────────────────────────────────────

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummonClass {
    SemanticSlime,
    GrammarGolem,
    RhetoricRobot,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summon(pub SummonClass);

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub enum Morpheme {
    Prefix(String),
    Root(String),
    Suffix(String),
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

impl Element {
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
}

#[derive(Component)]
pub struct AvatarAnimation {
    pub time: f32,
    pub base_y: f32,
    pub state_transition: f32,
    pub previous_state: PetVisualState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
pub enum PetVisualState {
    #[default]
    Idle,
    Alert,
    Battle,
    Happy,
    Sleeping,
}

#[derive(Component)]
pub struct OrbitalRing {
    pub speed: f32,
}

// ─── GAME STATE ──────────────────────────────────────────────────

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Collecting,
    Constructing,
    Playing,
    Questing,
    Battling,
    Reviewing,
    Paywall,
}
