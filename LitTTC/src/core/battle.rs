// battle.rs — Turn-based synonym/antonym card combat against wild typos
use bevy::prelude::*;
use crate::components::*;
use crate::database::*;
use crate::quest;
use crate::deck;
use std::collections::HashMap;

const WAND_DUEL_DISTANCE_BASE_MULTIPLIER: f32 = 1.5;
const WAND_DUEL_DISTANCE_SCALE: f32 = 0.2;
const SYNONYM_BOOST_MULTIPLIER: f32 = 2.0;

// Combat constants
const BASE_DAMAGE: f32 = 25.0;
const TYPO_MAX_HEALTH: i32 = 100;
const PLAYER_MAX_HEALTH: i32 = 100;
const INEFFECTIVE_DAMAGE_PENALTY: i32 = 20;
const HYPERBOLE_RECOIL_DAMAGE: i32 = 10;
const PALINDROME_REFLECTION_PERCENT: f32 = 0.5;

// Pillar 3: Syntax Spell Crafting multipliers
const NOUN_SUMMON_MULTIPLIER: f32 = 1.0;
const ADJECTIVE_AURA_MULTIPLIER: f32 = 1.2;
const VERB_ACTION_MULTIPLIER: f32 = 1.3;

// Pillar 4: Literary Device Metamagic multipliers
const OXYMORON_ARMOR_PIERCING: f32 = 2.0; // Bypasses shields
const HYPERBOLE_OVERCHARGE: f32 = 3.0; // Triple damage with recoil

// VAAM (Vocabulary Acquisition Autonomous Meaning) Tracking
//
// VAAM is the pedagogical framework that measures how players acquire vocabulary
// through autonomous meaning-making rather than rote memorization. The four dimensions
// are:
//
// - **Vocabulary (V)**: The breadth of words the player has encountered
// - **Autonomy (A)**: How often the player uses words independently vs. prompted
// - **Acquisition (A)**: How successfully the player applies words in context (combat)
// - **Mastery (M)**: Depth of understanding through repeated effective use
//
// This tracking system provides quantitative data on educational progress without
// relying on traditional quiz-based assessment.

#[derive(Debug, Clone, Default, Resource)]
pub struct VaamMetrics {
    /// Vocabulary: Total unique words encountered
    ///
    /// Tracks the breadth of the player's vocabulary. Each unique word encountered
    /// (whether through spelling, card play, or NPC dialogue) increments this counter.
    pub vocabulary_count: usize,

    /// Autonomy: Words used independently (not prompted)
    ///
    /// Measures self-directed vocabulary use. Words chosen by the player without
    /// explicit prompting count toward autonomy, indicating independent decision-making
    /// in word selection.
    pub autonomous_usage: usize,

    /// Acquisition: Words successfully used in combat
    ///
    /// Tracks contextual application of vocabulary. Words that successfully damage
    /// enemies or achieve combat objectives demonstrate the player understands the
    /// word's meaning in context.
    pub successful_acquisitions: usize,

    /// Mastery: Words used multiple times with increasing effectiveness
    ///
    /// Maps each word to its usage count. Repeated effective use (5+ times) indicates
    /// mastery, suggesting the player has internalized the word's meaning and can
    /// apply it strategically.
    pub mastery_level: HashMap<String, u32>,

    /// Semantic depth: Average semantic distance of word choices
    ///
    /// Measures the conceptual breadth of vocabulary choices. High semantic distance
    /// between words indicates the player is exploring diverse conceptual spaces,
    /// while low distance suggests focused exploration of related concepts.
    pub semantic_depth: f32,

    /// Combat effectiveness: Average damage multiplier achieved
    ///
    /// Tracks the strategic effectiveness of word choices. Higher multipliers indicate
    /// the player is selecting words that leverage game mechanics (synonyms, antonyms,
    /// literary devices) effectively.
    pub combat_effectiveness: f32,

    /// Literary device usage: Count of metamagic triggers
    ///
    /// Tracks awareness and application of literary devices (oxymoron, hyperbole,
    /// palindrome). This measures higher-order linguistic understanding beyond basic
    /// vocabulary.
    pub literary_device_usage: HashMap<String, usize>,
}

impl VaamMetrics {
    /// Record that a word was encountered (regardless of context)
    ///
    /// This increments both the vocabulary count and the mastery level for the word.
    /// Use this when a player sees a word in any context (spelling, card play, dialogue).
    pub fn record_word_encounter(&mut self, word: &str) {
        self.vocabulary_count += 1;
        *self.mastery_level.entry(word.to_lowercase()).or_insert(0) += 1;
    }

    /// Record autonomous (self-directed) word usage
    ///
    /// Use this when a player chooses a word without explicit prompting or hints.
    /// This indicates independent decision-making and contributes to the autonomy score.
    pub fn record_autonomous_usage(&mut self, word: &str) {
        self.autonomous_usage += 1;
        self.record_word_encounter(word);
    }

    /// Record successful word application in combat
    ///
    /// Use this when a word is used effectively in combat (deals damage, achieves objective).
    /// The damage multiplier is tracked to measure combat effectiveness over time.
    pub fn record_successful_acquisition(&mut self, word: &str, damage_multiplier: f32) {
        self.successful_acquisitions += 1;
        self.record_word_encounter(word);

        // Update combat effectiveness (running average)
        let total_acquisitions = self.successful_acquisitions as f32;
        self.combat_effectiveness = (self.combat_effectiveness * (total_acquisitions - 1.0) + damage_multiplier) / total_acquisitions;
    }

    /// Record usage of a literary device (metamagic)
    ///
    /// Use this when a player triggers literary device mechanics (oxymoron, hyperbole, palindrome).
    /// This tracks higher-order linguistic understanding beyond basic vocabulary.
    pub fn record_literary_device(&mut self, device: &str) {
        *self.literary_device_usage.entry(device.to_string()).or_insert(0) += 1;
    }

    /// Calculate overall VAAM score (0.0 to 1.0)
    ///
    /// Combines all four VAAM dimensions into a single score:
    /// - Vocabulary: capped at 100 words for full score
    /// - Autonomy: percentage of independent word usage
    /// - Acquisition: percentage of successful combat applications
    /// - Mastery: average usage count / 5.0 (5 uses = full mastery)
    ///
    /// Returns a value between 0.0 (no progress) and 1.0 (complete mastery).
    pub fn get_vaam_score(&self) -> f32 {
        // Calculate overall VAAM score (0.0 to 1.0)
        let vocabulary_score = (self.vocabulary_count as f32 / 100.0).min(1.0);
        let autonomy_score = if self.vocabulary_count > 0 {
            self.autonomous_usage as f32 / self.vocabulary_count as f32
        } else {
            0.0
        };
        let acquisition_score = if self.vocabulary_count > 0 {
            self.successful_acquisitions as f32 / self.vocabulary_count as f32
        } else {
            0.0
        };
        let mastery_score = if self.mastery_level.is_empty() {
            0.0
        } else {
            let avg_mastery = self.mastery_level.values().sum::<u32>() as f32 / self.mastery_level.len() as f32;
            (avg_mastery / 5.0).min(1.0) // Assume 5 uses = full mastery
        };

        (vocabulary_score + autonomy_score + acquisition_score + mastery_score) / 4.0
    }

    /// Generate a human-readable summary of VAAM progress
    ///
    /// Returns a formatted string showing the overall VAAM score and breakdown
    /// of each dimension for display to players or educators.
    pub fn get_summary(&self) -> String {
        format!(
            "VAAM Score: {:.2}\nVocabulary: {} unique words\nAutonomy: {}% independent usage\nAcquisition: {}% successful uses\nMastery: {} words mastered\nLiterary Devices: {} types used",
            self.get_vaam_score(),
            self.vocabulary_count,
            if self.vocabulary_count > 0 {
                self.autonomous_usage * 100 / self.vocabulary_count
            } else {
                0
            },
            if self.vocabulary_count > 0 {
                self.successful_acquisitions * 100 / self.vocabulary_count
            } else {
                0
            },
            self.mastery_level.values().filter(|&&m| m >= 5).count(),
            self.literary_device_usage.len()
        )
    }
}

#[derive(Resource, Debug, Clone)]
pub struct BattleSession {
    pub typo_word: String,
    pub typo_health: i32,
    pub player_health: i32,
    pub failed_word: Option<String>, // Track word that caused defeat for Tutor Loop
}

/// The player's sentence-in-progress during Thesaurus Dance combat.
/// A complete sentence is an adjective + noun + verb trio (max 3 cards).
#[derive(Resource, Debug, Clone, Default)]
pub struct Plot {
    pub cards: Vec<String>,
    pub max_size: usize,
}

impl Plot {
    pub fn new() -> Self {
        Self { cards: Vec::new(), max_size: 3 }
    }

    pub fn sentence_preview(&self) -> String {
        match self.cards.len() {
            0 => "[ ... ] [ ... ] [ ... ]".to_string(),
            1 => format!("[ {} ] [ ... ] [ ... ]", self.cards[0]),
            2 => format!("[ {} ] [ {} ] [ ... ]", self.cards[0], self.cards[1]),
            3 => format!("[ {} ] [ {} ] [ {} ]", self.cards[0], self.cards[1], self.cards[2]),
            _ => self.cards.join(" "),
        }
    }
}

#[derive(Component)]
pub struct CriticalHitTrigger;

pub fn semantic_distance(a: &WordStats, b: &WordStats) -> f32 {
    let dc = a.concreteness - b.concreteness;
    let dv = a.valence - b.valence;
    let dd = a.dominance - b.dominance;
    let da = a.intensity - b.intensity;
    (dc*dc + dv*dv + dd*dd + da*da).sqrt()
}

/// Check if two words form an oxymoron (high semantic distance, opposing concepts)
pub fn is_oxymoron(word1: &str, word2: &str, db: &GameDatabase) -> bool {
    let lower1 = word1.to_lowercase();
    let lower2 = word2.to_lowercase();

    if let (Some(stats1), Some(stats2)) = (db.words.get(&lower1), db.words.get(&lower2)) {
        let distance = semantic_distance(stats1, stats2);
        // Oxymorons have high semantic distance (opposing concepts)
        distance > 4.0
    } else {
        false
    }
}

/// Check if multiple words start with the same letter (alliteration)
pub fn is_alliteration(words: &[&str]) -> bool {
    if words.len() < 2 {
        return false;
    }

    let first_char = words[0].chars().next().map(|c| c.to_lowercase().next());
    if first_char.is_none() {
        return false;
    }

    let first_char = first_char.unwrap().unwrap();

    words.iter().all(|word| {
        word.chars()
            .next()
            .map(|c| c.to_lowercase().next() == Some(first_char))
            .unwrap_or(false)
    })
}

/// Check if a word is a palindrome (reads same forwards and backwards)
pub fn is_palindrome(word: &str) -> bool {
    let lower = word.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    let len = chars.len();

    if len < 2 {
        return false;
    }

    for i in 0..len / 2 {
        if chars[i] != chars[len - 1 - i] {
            return false;
        }
    }

    true
}

/// Check if a word uses hyperbole (exaggerated intensity)
pub fn is_hyperbole(word: &str, db: &GameDatabase) -> bool {
    let lower = word.to_lowercase();
    if let Some(stats) = db.words.get(&lower) {
        // High intensity indicates hyperbole
        stats.intensity > 4.0
    } else {
        false
    }
}

pub fn start_battle(
    commands: &mut Commands,
    db: &GameDatabase,
    grade_manager: &crate::quest::GradeManager,
    next_state: &mut NextState<GameState>,
    state: &State<GameState>,
) {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let valid_grades = grade_manager.get_valid_grade_levels();

    let valid_words: Vec<&String> = db.words.iter()
        .filter(|(_, stats)| valid_grades.contains(&stats.grade_level.as_str()))
        .map(|(word, _)| word)
        .collect();

    let mut typo_word = "inferno".to_string(); // fallback
    if let Some(&word) = valid_words.choose(&mut thread_rng()) {
        typo_word = word.clone();
    }

    commands.insert_resource(BattleSession {
        typo_word: typo_word.clone(),
        typo_health: 50,
        player_health: 100,
        failed_word: None,
    });
    commands.insert_resource(Plot::new());

    info!("A wild Typo ({}) emerges! Deduce its semantic weakness based on its stats!", typo_word.to_uppercase());
    crate::commands::log_state_transition(state.get(), GameState::Battling);
    next_state.set(GameState::Battling);
}

/// Start Tutor Loop quest after battle defeat
pub fn start_tutor_loop(
    commands: &mut Commands,
    db: &GameDatabase,
    grade_manager: &crate::quest::GradeManager,
    battle_session: &BattleSession,
    next_state: &mut NextState<GameState>,
    state: &State<GameState>,
) {
    let _sheet = &CharacterSheet::default(); // Unused in MVP
    if let Some(failed_word) = &battle_session.failed_word {
        let tutor_npc = quest::route_to_tutor_npc(failed_word, db);
        info!("Tutor Loop: Routing to {} for practice on '{}'", tutor_npc, failed_word);
        
        quest::start_quest(&tutor_npc, db, grade_manager, commands, next_state, state);
    } else {
        // Fallback if no failed word tracked
        quest::start_quest("Barnaby", db, grade_manager, commands, next_state, state);
    }
}

pub struct BattleResult {
    pub is_effective: bool,
    pub is_counter: bool, // High distance = antonym/counter
    pub is_synonym: bool, // Low distance = synonym/heavy attack
}

#[allow(clippy::too_many_arguments)]
pub fn play_battle_card(
    played_word: &str,
    session: &mut BattleSession,
    db: &GameDatabase,
    spellbook: &mut SpellBook,
    next_state: &mut NextState<GameState>,
    _sheet: &CharacterSheet,
    state: &State<GameState>,
    active_face: Option<&ActiveFace>,
    mut vaam_metrics: Option<&mut VaamMetrics>,
) -> BattleResult {
    let lower_typo = session.typo_word.to_lowercase();
    let lower_played = played_word.to_lowercase();

    let mut damage_multiplier = 1.0;
    let mut is_effective = false;
    let mut is_counter = false;
    let mut is_synonym = false;

    if let (Some(typo_stats), Some(played_stats)) = (db.words.get(&lower_typo), db.words.get(&lower_played)) {
        let distance = semantic_distance(typo_stats, played_stats);

        // Wand Duel: High distance = antonym/counter, Low distance = synonym/heavy attack
        if distance > 4.0 {
            // Counter/Block: High semantic distance = opposing concepts
            damage_multiplier = WAND_DUEL_DISTANCE_BASE_MULTIPLIER + (distance - 4.0) * WAND_DUEL_DISTANCE_SCALE;
            is_effective = true;
            is_counter = true;
        } else if distance < 2.0 {
            // Synonym/Heavy Attack: Low semantic distance = similar concepts
            damage_multiplier = SYNONYM_BOOST_MULTIPLIER;
            is_effective = true;
            is_synonym = true;
        } else {
            // Mid-range: normal damage
            damage_multiplier = 1.0;
            is_effective = true;
        }

        // Pillar 3: Syntax Spell Crafting - Apply part-of-speech multipliers
        let played_pos = played_stats.part_of_speech.to_lowercase();
        match played_pos.as_str() {
            "noun" => {
                damage_multiplier *= NOUN_SUMMON_MULTIPLIER;
                info!("Noun: Summon/Target base damage");
            }
            "adjective" => {
                damage_multiplier *= ADJECTIVE_AURA_MULTIPLIER;
                info!("Adjective: Aura/Element multiplier applied");
            }
            "verb" => {
                damage_multiplier *= VERB_ACTION_MULTIPLIER;
                info!("Verb: Action multiplier applied");
            }
            _ => {
                // Unknown or no POS data - no multiplier
            }
        }
    }

    let base_damage = BASE_DAMAGE;

    // Pillar 4: Literary Device Metamagic (applied after base damage calculation)
    if db.words.contains_key(&lower_typo) && db.words.contains_key(&lower_played) {
        // Check for oxymoron (opposing concepts)
        if is_oxymoron(&session.typo_word, played_word, db) {
            damage_multiplier *= OXYMORON_ARMOR_PIERCING;
            info!("OXYMORON! Armor piercing applied (2.0x)");
            if let Some(ref mut metrics) = vaam_metrics {
                metrics.record_literary_device("oxymoron");
            }
        }

        // Check for hyperbole (exaggerated intensity)
        if is_hyperbole(played_word, db) {
            damage_multiplier *= HYPERBOLE_OVERCHARGE;
            info!("HYPERBOLE! Overcharge applied (3.0x) - potential recoil");
            // Hyperbole recoil: player takes damage
            session.player_health -= HYPERBOLE_RECOIL_DAMAGE;
            if let Some(ref mut metrics) = vaam_metrics {
                metrics.record_literary_device("hyperbole");
            }
        }

        // Check for palindrome (defensive reflection)
        if is_palindrome(played_word) {
            info!("PALINDROME! Defensive reflection active");
            // Palindromes reflect damage back to enemy
            let reflected_damage = (base_damage * damage_multiplier * PALINDROME_REFLECTION_PERCENT) as i32;
            session.typo_health -= reflected_damage;
            if let Some(ref mut metrics) = vaam_metrics {
                metrics.record_literary_device("palindrome");
            }
        }
    }

    // Pillar 2: Apply FACES emotional modifiers
    if let Some(face) = active_face {
        match face.face {
            SlimeFace::Fierce => {
                damage_multiplier *= 1.2; // High damage/intensity
                info!("Fierce Face: Damage increased by 20%");
            }
            SlimeFace::Joyful => {
                damage_multiplier *= 1.1; // Slight boost, potential healing
                info!("Joyful Face: Damage increased by 10%");
            }
            SlimeFace::Calm => {
                damage_multiplier *= 1.0; // Neutral, precise
                info!("Calm Face: Standard damage");
            }
            SlimeFace::Angry => {
                damage_multiplier *= 1.3; // High damage, potential recoil
                info!("Angry Face: Damage increased by 30%");
            }
        }
    }

    let base_damage = BASE_DAMAGE;
    let final_damage = (base_damage * damage_multiplier) as i32;

    if is_effective {
        session.typo_health -= final_damage;
        if is_counter {
            info!("COUNTER! '{}' (antonym) blocks '{}'. Damage: {:.2}x → {}. Typo health: {}/{}",
                played_word, session.typo_word, damage_multiplier, final_damage, session.typo_health, TYPO_MAX_HEALTH);
        } else if is_synonym {
            info!("HEAVY ATTACK! '{}' (synonym) overwhelms '{}'. Damage: {:.2}x → {}. Typo health: {}/{}",
                played_word, session.typo_word, damage_multiplier, final_damage, session.typo_health, TYPO_MAX_HEALTH);
        } else {
            info!("HIT! '{}' deals {} damage to '{}'. Typo health: {}/{}",
                played_word, final_damage, session.typo_word, session.typo_health, TYPO_MAX_HEALTH);
        }
        spellbook.upgrade_mastery(played_word, MasteryLevel::Owned);

        // Record VAAM metrics for successful word usage
        if let Some(ref mut metrics) = vaam_metrics {
            metrics.record_successful_acquisition(played_word, damage_multiplier);
        }
    } else {
        session.typo_health -= final_damage;
        session.player_health -= INEFFECTIVE_DAMAGE_PENALTY;
        warn!("INEFFECTIVE! '{}' fails against '{}'. Damage: {:.2}x → {}. Typo counters for {} damage! Player health: {}/{}",
            played_word, session.typo_word, damage_multiplier, final_damage, INEFFECTIVE_DAMAGE_PENALTY, session.player_health, PLAYER_MAX_HEALTH);
    }

    if session.typo_health <= 0 {
        info!("VICTORY! '{}' defeated '{}'. The verse is clean.", played_word, session.typo_word);
        spellbook.upgrade_mastery(played_word, MasteryLevel::Mastered);
        crate::commands::log_state_transition(state.get(), GameState::Reviewing);
        next_state.set(GameState::Reviewing);
    } else if session.player_health <= 0 {
        warn!("DEFEAT! '{}' overrode your verse. Entering Tutor Loop for remedial practice.", session.typo_word);
        session.failed_word = Some(session.typo_word.clone()); // Track failed word for NPC routing
        // Note: The actual Tutor Loop quest start will be handled by the command handler
        // which has access to all needed resources. We just transition state here.
        crate::commands::log_state_transition(state.get(), GameState::Questing);
        next_state.set(GameState::Questing);
    }

    BattleResult {
        is_effective,
        is_counter,
        is_synonym,
    }
}

/// Resolve the Thesaurus Dance Plot as one cohesive sentence attack.
///
/// Damage is summed from each card, then amplified by sentence-level combos
/// (alliteration, palindrome, oxymoron, hyperbole) and the active FACES stance.
#[allow(clippy::too_many_arguments)]
pub fn cast_sentence(
    session: &mut BattleSession,
    plot: Option<&mut Plot>,
    db: &GameDatabase,
    spellbook: &mut SpellBook,
    next_state: &mut NextState<GameState>,
    _sheet: &CharacterSheet,
    state: &State<GameState>,
    active_face: Option<&ActiveFace>,
    mut vaam_metrics: Option<&mut VaamMetrics>,
) {
    let plot = match plot {
        Some(p) => p,
        None => {
            warn!("CastSentence called with no Plot resource");
            return;
        }
    };

    if plot.cards.is_empty() {
        warn!("Cannot cast an empty sentence. Add words to the Plot first.");
        return;
    }

    let lower_typo = session.typo_word.to_lowercase();
    let typo_stats = db.words.get(&lower_typo);

    let mut total_damage: i32 = 0;
    let mut sentence_multiplier = 1.0;
    let mut recoil = 0;
    let mut reflection = 0;
    let mut logged_words: Vec<String> = Vec::new();

    for word in &plot.cards {
        let lower_word = word.to_lowercase();
        logged_words.push(word.clone());

        let mut card_multiplier = 1.0;
        if let (Some(t_stats), Some(w_stats)) = (typo_stats, db.words.get(&lower_word)) {
            let distance = semantic_distance(t_stats, w_stats);
            if distance > 4.0 {
                card_multiplier *= WAND_DUEL_DISTANCE_BASE_MULTIPLIER + (distance - 4.0) * WAND_DUEL_DISTANCE_SCALE;
            } else if distance < 2.0 {
                card_multiplier *= SYNONYM_BOOST_MULTIPLIER;
            }

            match w_stats.part_of_speech.to_lowercase().as_str() {
                "noun" => card_multiplier *= NOUN_SUMMON_MULTIPLIER,
                "adjective" => card_multiplier *= ADJECTIVE_AURA_MULTIPLIER,
                "verb" => card_multiplier *= VERB_ACTION_MULTIPLIER,
                _ => {}
            }
        }

        if is_oxymoron(&session.typo_word, word, db) {
            sentence_multiplier *= OXYMORON_ARMOR_PIERCING;
            if let Some(ref mut metrics) = vaam_metrics {
                metrics.record_literary_device("oxymoron");
            }
        }
        if is_hyperbole(word, db) {
            sentence_multiplier *= HYPERBOLE_OVERCHARGE;
            recoil += HYPERBOLE_RECOIL_DAMAGE;
            if let Some(ref mut metrics) = vaam_metrics {
                metrics.record_literary_device("hyperbole");
            }
        }
        if is_palindrome(word) {
            reflection += (BASE_DAMAGE * card_multiplier * PALINDROME_REFLECTION_PERCENT) as i32;
            if let Some(ref mut metrics) = vaam_metrics {
                metrics.record_literary_device("palindrome");
            }
        }

        total_damage += (BASE_DAMAGE * card_multiplier) as i32;
        spellbook.upgrade_mastery(word, MasteryLevel::Owned);
        if let Some(ref mut metrics) = vaam_metrics {
            metrics.record_successful_acquisition(word, card_multiplier);
        }
    }

    // Sentence-level alliteration bonus.
    let refs: Vec<&str> = plot.cards.iter().map(|s| s.as_str()).collect();
    if is_alliteration(&refs) {
        sentence_multiplier *= 1.2;
        info!("ALLITERATION! The verse flows with matching sounds (+20%)");
        if let Some(ref mut metrics) = vaam_metrics {
            metrics.record_literary_device("alliteration");
        }
    }

    // Apply FACES emotional stance once to the whole sentence.
    if let Some(face) = active_face {
        match face.face {
            SlimeFace::Fierce => sentence_multiplier *= 1.2,
            SlimeFace::Joyful => sentence_multiplier *= 1.1,
            SlimeFace::Calm => sentence_multiplier *= 1.0,
            SlimeFace::Angry => sentence_multiplier *= 1.3,
        }
    }

    let final_damage = (total_damage as f32 * sentence_multiplier) as i32 + reflection;
    session.typo_health -= final_damage;
    session.player_health -= recoil;

    info!("CAST SENTENCE: '{}' deals {} damage. Typo health: {}/{}",
        plot.cards.join(" "), final_damage, session.typo_health, TYPO_MAX_HEALTH);

    plot.cards.clear();

    if session.typo_health <= 0 {
        info!("VICTORY! The crafted sentence purified '{}'.", session.typo_word);
        for word in &logged_words {
            spellbook.upgrade_mastery(word, MasteryLevel::Mastered);
        }
        crate::commands::log_state_transition(state.get(), GameState::Reviewing);
        next_state.set(GameState::Reviewing);
    } else if session.player_health <= 0 {
        warn!("DEFEAT! '{}' corrupted the verse. Entering Tutor Loop.", session.typo_word);
        session.failed_word = Some(session.typo_word.clone());
        crate::commands::log_state_transition(state.get(), GameState::Questing);
        next_state.set(GameState::Questing);
    }
}

#[derive(Component)]
pub struct PlayerHealthBar;

#[derive(Component)]
pub struct EnemyHealthBar;

#[derive(Component)]
pub struct PlotPreviewText;

#[derive(Component)]
pub struct BattleUiMarker;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_distance_is_zero_for_identical_stats() {
        let stats = WordStats {
            concreteness: 2.0,
            valence: 3.0,
            intensity: 4.0,
            dominance: 5.0,
            grade_level: "6-8".to_string(),
            part_of_speech: "noun".to_string(),
        };
        assert!((semantic_distance(&stats, &stats) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn semantic_distance_increases_with_divergent_stats() {
        let a = WordStats {
            concreteness: 1.0,
            valence: 1.0,
            intensity: 1.0,
            dominance: 1.0,
            grade_level: "K-2".to_string(),
            part_of_speech: "noun".to_string(),
        };
        let b = WordStats {
            concreteness: 4.0,
            valence: 4.0,
            intensity: 4.0,
            dominance: 4.0,
            grade_level: "6-8".to_string(),
            part_of_speech: "adjective".to_string(),
        };
        let dist = semantic_distance(&a, &b);
        assert!(dist > 5.0);
    }

    #[test]
    fn battle_result_defaults_are_false() {
        let result = BattleResult {
            is_effective: false,
            is_counter: false,
            is_synonym: false,
        };
        assert!(!result.is_effective);
        assert!(!result.is_counter);
        assert!(!result.is_synonym);
    }
}

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "xr")]
        app.add_systems(OnEnter(GameState::Battling), (spawn_battle_ui_xr, set_pet_battle_state))
           .add_systems(Update, update_battle_hp_bars_xr.run_if(in_state(GameState::Battling)))
           .add_systems(OnExit(GameState::Battling), (cleanup_battle_ui_xr, set_pet_idle_state));

        #[cfg(not(feature = "flat2d"))]
        app.add_systems(Update, handle_critical_hit_effects);

        #[cfg(not(feature = "xr"))]
        app.add_systems(OnEnter(GameState::Battling), (spawn_battle_ui_2d, set_pet_battle_state))
           .add_systems(Update, update_battle_hp_bars_2d.run_if(in_state(GameState::Battling)))
           .add_systems(OnExit(GameState::Battling), (cleanup_battle_ui_2d, set_pet_idle_state));

        // Phase 1: Gray-box combat systems (2D only, feature-gated for testing)
        #[cfg(all(not(feature = "xr"), feature = "graybox"))]
        app.add_systems(OnEnter(GameState::Battling), start_graybox_battle)
           .add_systems(Update, (
               handle_hand_card_click,
               handle_face_button_click,
               handle_cast_spell_click,
               apply_damage_to_dummy,
               enemy_turn_ai,
               check_battle_end_conditions,
               debug_trigger_graybox_battle,
           ).run_if(in_state(GameState::Battling)));
    }
}

#[cfg(not(feature = "flat2d"))]
pub fn handle_critical_hit_effects(
    trigger_query: Query<Entity, With<CriticalHitTrigger>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<Entity, With<Camera>>,
) {
    for trigger_entity in trigger_query.iter() {
        commands.entity(trigger_entity).despawn();
        
        for entity in &camera_query {
            commands.entity(entity).insert(crate::render::ScreenShake { timer: 0.3, intensity: 0.2 });
        }
        
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for _ in 0..30 {
            let vx = rng.gen_range(-4.0..4.0);
            let vy = rng.gen_range(2.0..6.0);
            let vz = rng.gen_range(-4.0..4.0);
            
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.06).mesh().ico(1).unwrap())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.9, 0.1),
                    emissive: Color::srgb(2.0, 1.8, 0.2).into(),
                    ..default()
                })),
                Transform::from_xyz(0.0, 1.5, -2.0),
                crate::render::BurstParticle {
                    velocity: Vec3::new(vx, vy, vz),
                    timer: 1.5,
                }
            ));
        }
    }
}

fn set_pet_battle_state(
    mut query: Query<&mut PetVisualState, With<PetAvatar>>,
) {
    for mut state in &mut query {
        *state = PetVisualState::Battle;
    }
}

fn set_pet_idle_state(
    mut query: Query<&mut PetVisualState, With<PetAvatar>>,
) {
    for mut state in &mut query {
        *state = PetVisualState::Idle;
    }
}

#[cfg(feature = "xr")]
fn spawn_battle_ui_xr(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    session: Res<BattleSession>,
) {
    let instruction_text = format!("WILD TYPO: {}", session.typo_word.to_uppercase());
    commands.spawn((
        BattleUiMarker,
        Text2d::new(instruction_text),
        TextFont { font_size: 36.0, ..default() },
        TextColor(Color::srgb(0.9, 0.9, 0.2)),
        Transform::from_xyz(0.0, 2.5, -2.0),
    ));
    // Player HP bar
    let player_bar = commands.spawn((
        PlayerHealthBar,
        BattleUiMarker,
        Mesh3d(meshes.add(Cuboid::new(1.0, 0.1, 0.02))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.8, 0.2),
            ..default()
        })),
        Transform::from_xyz(-1.5, 1.8, -2.0),
    )).id();

    let player_text = commands.spawn((
        BattleUiMarker,
        Text2d::new("Player HP: 100"),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 0.15, 0.02),
    )).id();
    commands.entity(player_bar).add_child(player_text);

    // Enemy HP bar
    let enemy_bar = commands.spawn((
        EnemyHealthBar,
        BattleUiMarker,
        Mesh3d(meshes.add(Cuboid::new(1.0, 0.1, 0.02))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            ..default()
        })),
        Transform::from_xyz(1.5, 1.8, -2.0),
    )).id();

    let enemy_text = commands.spawn((
        BattleUiMarker,
        Text2d::new("Typo HP: 50"),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 0.15, 0.02),
    )).id();
    commands.entity(enemy_bar).add_child(enemy_text);
}

#[cfg(feature = "xr")]
fn update_battle_hp_bars_xr(
    session: Option<Res<BattleSession>>,
    mut player_bar: Query<(&mut Transform, &Children), (With<PlayerHealthBar>, Without<EnemyHealthBar>)>,
    mut enemy_bar: Query<(&mut Transform, &Children), (With<EnemyHealthBar>, Without<PlayerHealthBar>)>,
    mut text_query: Query<&mut Text2d>,
) {
    let session = match session {
        Some(s) => s,
        None => return,
    };

    for (mut transform, children) in &mut player_bar {
        let ratio = (session.player_health as f32 / 100.0).clamp(0.0, 1.0);
        transform.scale.x = ratio;
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = format!("Player HP: {}", session.player_health);
            }
        }
    }

    for (mut transform, children) in &mut enemy_bar {
        let ratio = (session.typo_health as f32 / 50.0).clamp(0.0, 1.0);
        transform.scale.x = ratio;
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = format!("Typo HP: {}", session.typo_health);
            }
        }
    }
}

#[cfg(feature = "xr")]
fn cleanup_battle_ui_xr(
    mut commands: Commands,
    query: Query<Entity, With<BattleUiMarker>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[cfg(not(feature = "xr"))]
fn spawn_battle_ui_2d(
    mut commands: Commands,
    session: Res<BattleSession>,
) {
    let instruction_text = format!("WILD TYPO: {}", session.typo_word.to_uppercase());
    
    commands.spawn((
        BattleUiMarker,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn((
            Text::new(instruction_text),
            TextFont { font_size: 36.0, ..default() },
            TextColor(Color::srgb(0.9, 0.9, 0.2)),
        ));

        // Container for HP Bars
        parent.spawn((
            Node {
                margin: UiRect::top(Val::Px(20.0)),
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Px(400.0),
                ..default()
            },
        )).with_children(|bars| {
            bars.spawn((
                PlayerHealthBar,
                Text::new("Player HP: 100"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::srgb(0.2, 0.8, 0.2)),
            ));

            bars.spawn((
                EnemyHealthBar,
                Text::new("Typo HP: 50"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::srgb(0.8, 0.2, 0.2)),
            ));
        });

        // Plot / sentence preview
        parent.spawn((
            PlotPreviewText,
            Text::new("[ ... ] [ ... ] [ ... ]"),
            TextFont { font_size: 28.0, ..default() },
            TextColor(Color::srgb(0.6, 0.8, 1.0)),
        ));

        // Hint line
        parent.spawn((
            Text::new("Weak to antonyms (far meaning) and verbs."),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
    });
}

#[cfg(not(feature = "xr"))]
fn update_battle_hp_bars_2d(
    session: Option<Res<BattleSession>>,
    plot: Option<Res<Plot>>,
    mut player_bar: Query<&mut Text, (With<PlayerHealthBar>, Without<EnemyHealthBar>, Without<PlotPreviewText>)>,
    mut enemy_bar: Query<&mut Text, (With<EnemyHealthBar>, Without<PlayerHealthBar>, Without<PlotPreviewText>)>,
    mut plot_text: Query<&mut Text, With<PlotPreviewText>>,
) {
    let session = match session {
        Some(s) => s,
        None => return,
    };

    if let Some(mut text) = player_bar.iter_mut().next() {
        text.0 = format!("Player HP: {}", session.player_health);
    }

    if let Some(mut text) = enemy_bar.iter_mut().next() {
        text.0 = format!("Typo HP: {}", session.typo_health);
    }

    if let Some(p) = plot {
        if let Some(mut text) = plot_text.iter_mut().next() {
            text.0 = p.sentence_preview();
        }
    }
}

#[cfg(not(feature = "xr"))]
fn cleanup_battle_ui_2d(
    mut commands: Commands,
    query: Query<Entity, With<BattleUiMarker>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// ─── PHASE 1: 2D GRAY-BOX COMBAT SYSTEMS ───────────────────────────

/// Spawn the 2D gray-box battle UI for Phase 1
pub fn spawn_battle_ui_2d_graybox(
    mut commands: Commands,
    micro_deck_words: Vec<String>,
) {
    // Target Dummy UI at top center
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Px(50.0),
            right: Val::Px(50.0),
            height: Val::Px(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        TargetDummy {
            hp: 100,
            max_hp: 100,
            prompt_word: micro_deck_words.first().cloned().unwrap_or_else(|| "TEST".to_string()),
        },
    )).with_children(|parent| {
        parent.spawn((
            Text::new("PROMPT WORD: TEST"),
            TextFont { font_size: 32.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });

    // Player Hand UI at bottom (5 card buttons)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Px(20.0),
            right: Val::Px(20.0),
            height: Val::Px(120.0),
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            ..default()
        },
    )).with_children(|parent| {
        for (i, word) in micro_deck_words.iter().take(5).enumerate() {
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(100.0),
                    height: Val::Px(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                HandCardButton {
                    word: word.clone(),
                    part_of_speech: deck::PartOfSpeech::Noun,
                    index: i,
                },
            )).with_children(|parent| {
                parent.spawn((
                    Text::new(word.to_uppercase()),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        }
    });

    // Altar Drop-Zone in center
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(200.0),
            left: Val::Px(50.0),
            right: Val::Px(50.0),
            height: Val::Px(150.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.4)),
        AltarDropZone {
            active_card: None,
        },
    )).with_children(|parent| {
        parent.spawn((
            Text::new("Active Card: None"),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::WHITE),
        ));
        parent.spawn((
            Button,
            Node {
                width: Val::Px(150.0),
                height: Val::Px(50.0),
                margin: UiRect::top(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.5, 0.3, 0.3)),
            CastSpellButton,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("CAST SPELL"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });

    // Slime Face UI (4 emotion buttons)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(380.0),
            left: Val::Px(50.0),
            right: Val::Px(50.0),
            height: Val::Px(80.0),
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            ..default()
        },
    )).with_children(|parent| {
        let faces = [SlimeFace::Fierce, SlimeFace::Joyful, SlimeFace::Calm, SlimeFace::Angry];
        for face in faces {
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(60.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.5, 0.3)),
                FaceButton { face },
            )).with_children(|parent| {
                parent.spawn((
                    Text::new(format!("{:?}", face)),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        }
    });

    // Combat Log UI on right side
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            right: Val::Px(20.0),
            width: Val::Px(300.0),
            height: Val::Px(400.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
    )).with_children(|parent| {
        parent.spawn((
            Text::new("COMBAT LOG"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(1.0, 0.84, 0.0)),
        ));
    });
}

/// Start a gray-box battle with the micro-deck
pub fn start_graybox_battle(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
) {
    let micro_deck = deck::get_micro_deck_words();
    
    commands.insert_resource(GrayBoxBattleSession {
        dummy_hp: 100,
        player_hp: 100,
        prompt_word: micro_deck.first().cloned().unwrap_or_else(|| "TEST".to_string()),
        turn: BattleTurn::Player,
    });

    commands.insert_resource(ActiveFace::default());

    spawn_battle_ui_2d_graybox(commands, micro_deck);

    info!("Gray-box battle started!");
    crate::commands::log_state_transition(state.get(), GameState::Battling);
    next_state.set(GameState::Battling);
}

/// Handle hand card clicks (play to altar)
#[allow(clippy::type_complexity)]
pub fn handle_hand_card_click(
    mut interaction_query: Query<
        (&Interaction, &HandCardButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut altar_query: Query<&mut AltarDropZone>,
) {
    for (interaction, card_button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            if let Ok(mut altar) = altar_query.single_mut() {
                altar.active_card = Some(card_button.word.clone());
                info!("Card played to altar: {}", card_button.word);
            }
        }
    }
}

/// Handle face button clicks
#[allow(clippy::type_complexity)]
pub fn handle_face_button_click(
    mut interaction_query: Query<
        (&Interaction, &FaceButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut active_face: ResMut<ActiveFace>,
) {
    for (interaction, face_button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            active_face.face = face_button.face;
            info!("Face changed to: {:?}", face_button.face);
        }
    }
}

/// Handle cast spell button
pub fn handle_cast_spell_click(
    mut interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<CastSpellButton>),
    >,
    altar_query: Query<&AltarDropZone>,
    active_face: Res<ActiveFace>,
    mut battle_session: ResMut<GrayBoxBattleSession>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            if let Ok(altar) = altar_query.single() {
                if let Some(ref word) = altar.active_card {
                    let (damage, math) = calculate_spell_damage(
                        word,
                        &battle_session.prompt_word,
                        active_face.face,
                    );
                    battle_session.dummy_hp -= damage;
                    info!("Spell cast: {} with face {:?} - Damage: {} ({})", word, active_face.face, damage, math);
                }
            }
        }
    }
}

/// Calculate synonym distance using micro-deck
pub fn calculate_synonym_distance(word1: &str, word2: &str) -> f32 {
    let micro_deck = deck::initialize_micro_deck();
    for micro_word in micro_deck {
        if micro_word.word == word1 {
            if micro_word.synonyms.contains(&word2.to_string()) {
                return 1.0;
            }
            if micro_word.antonyms.contains(&word2.to_string()) {
                return 5.0;
            }
        }
    }
    3.0
}

/// Check if words are synonyms
pub fn is_synonym(word1: &str, word2: &str) -> bool {
    calculate_synonym_distance(word1, word2) < 2.0
}

/// Check if words are antonyms
pub fn is_antonym(word1: &str, word2: &str) -> bool {
    calculate_synonym_distance(word1, word2) > 4.0
}

/// Calculate spell damage with face modifiers
pub fn calculate_spell_damage(
    played_word: &str,
    prompt_word: &str,
    face: SlimeFace,
) -> (i32, String) {
    let base_damage = 25;
    let distance = calculate_synonym_distance(played_word, prompt_word);
    
    let (multiplier, _action_type) = if is_synonym(played_word, prompt_word) {
        (2.0, BattleActionType::SynonymAttack)
    } else if is_antonym(played_word, prompt_word) {
        let extra = (distance - 4.0) * 0.2;
        (1.5 + extra, BattleActionType::AntonymBlock)
    } else {
        (1.0, BattleActionType::NormalAttack)
    };

    let face_modifier = match face {
        SlimeFace::Fierce => 1.2,
        SlimeFace::Joyful => 1.1,
        SlimeFace::Calm => 1.0,
        SlimeFace::Angry => 1.3,
    };

    let final_damage = (base_damage as f32 * multiplier * face_modifier) as i32;
    
    let math_breakdown = format!(
        "Base: {} × DistMult: {:.2} × FaceMod: {:.2} = {}",
        base_damage, multiplier, face_modifier, final_damage
    );

    (final_damage, math_breakdown)
}

/// Apply damage to target dummy (no longer needed with direct spell casting)
pub fn apply_damage_to_dummy(
    _battle_session: Res<GrayBoxBattleSession>,
) {
    // Damage is now applied directly in handle_cast_spell_click
    // This function is kept for future event-based integration
}

/// Enemy turn AI (dummy attacks with random word)
pub fn enemy_turn_ai(
    mut battle_session: ResMut<GrayBoxBattleSession>,
) {
    if battle_session.turn == BattleTurn::Enemy {
        let damage = 15;
        battle_session.player_hp -= damage;
        battle_session.turn = BattleTurn::Player;
        info!("Enemy attacked! Player HP: {}", battle_session.player_hp);
    }
}

/// Check battle end conditions
pub fn check_battle_end_conditions(
    battle_session: Res<GrayBoxBattleSession>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if battle_session.dummy_hp <= 0 {
        info!("Victory! Dummy defeated!");
        next_state.set(GameState::Playing);
    } else if battle_session.player_hp <= 0 {
        info!("Defeat! Player defeated!");
        next_state.set(GameState::Questing);
    }
}

/// Debug trigger: Press 'G' to start gray-box battle
pub fn debug_trigger_graybox_battle(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyG) && *state.get() != GameState::Battling {
        info!("Debug: Starting gray-box battle (G key pressed)");
        next_state.set(GameState::Battling);
    }
}

