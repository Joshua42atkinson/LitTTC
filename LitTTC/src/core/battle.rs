// battle.rs — Turn-based synonym/antonym card combat against wild typos
use bevy::prelude::*;
use crate::components::*;
use crate::database::*;
use crate::quest;
use crate::deck;
use faces_protocol::FacesState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Probability of drawing zero successes in a sample of size `k`
/// from a population of size `N` containing `K` successes.
/// Uses exact factorial-free arithmetic to avoid overflow.
fn hypergeometric_prob_zero(population: u32, successes: u32, sample: u32) -> f64 {
    if sample == 0 || successes == 0 || population == 0 {
        return 1.0;
    }
    if sample > population || successes >= population {
        return 0.0;
    }

    let failures = population - successes;
    if sample > failures {
        return 0.0;
    }
    // P(X = 0) = C(N-K, k) / C(N, k)
    // Compute as a running product to stay numerically stable.
    let mut prob = 1.0f64;
    for i in 0..sample {
        let num = (failures - i) as f64;
        let den = (population - i) as f64;
        if den == 0.0 {
            return 0.0;
        }
        prob *= num / den;
    }
    prob
}

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

// Stealth assessment constants
const LEXICAL_DIVERSITY_WINDOW: usize = 100;
const MTLD_FACTOR_SIZE: usize = 10;
const HD_D_SAMPLE_SIZE: usize = 42;

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

#[derive(Debug, Clone, Default, Resource, Serialize, Deserialize)]
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

    /// Subject mastery: Progress per NPC scenario subject
    ///
    /// Tracks repeated success on language subjects such as "simple-past", "negation",
    /// or "tone". Each successful quest completion or effective cast on a subject
    /// increments its counter, showing growing pragmatic competence.
    pub subject_mastery: HashMap<String, u32>,

    /// Temporal evidence series for institutional dashboards and psychometric reports.
    #[serde(default)]
    pub telemetry: crate::components::TelemetrySeries,

    /// CCSS standard coverage: counts of demonstrated exposures per standard code.
    #[serde(default)]
    pub ccss_coverage: HashMap<String, u32>,

    /// Rolling token window for HD-D / MTLD lexical diversity calculations.
    /// Stores lowercased words in cast order; capped to `LEXICAL_DIVERSITY_WINDOW`.
    pub token_window: Vec<String>,
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

    /// Record progress on a language subject (NPC scenario training)
    ///
    /// Use this when a player completes a quest or successfully casts a word tied to a
    /// subject such as "simple-past", "negation", or "tone".
    pub fn record_subject_mastery(&mut self, subject: &str) {
        if !subject.is_empty() {
            *self.subject_mastery.entry(subject.to_string()).or_insert(0) += 1;
        }
    }

    /// Append a telemetry event and recalculate all temporal metrics.
    ///
    /// This is the main entry point for stealth assessment evidence collection.
    pub fn record_cast_telemetry(&mut self, event: crate::components::CastTelemetry) {
        let word = event.word.clone();
        self.telemetry.cast_log.push(event.clone());
        self.telemetry.syntax_series.push(event.grades.syntax);
        self.telemetry.semantics_series.push(event.grades.semantics);
        self.telemetry.pragmatics_series.push(event.faces_resonance);

        // Update rolling token window for lexical diversity.
        self.token_window.push(word);
        if self.token_window.len() > LEXICAL_DIVERSITY_WINDOW {
            self.token_window.remove(0);
        }

        // Update CCSS coverage counts.
        for tag in &event.ccss_tags {
            *self.ccss_coverage.entry(tag.clone()).or_insert(0) += 1;
        }

        // Recalculate lexical diversity and syntactic complexity.
        let snapshot = self.compute_lexical_diversity();
        self.telemetry.lexical_diversity_series.push(snapshot);
        self.telemetry.syntactic_complexity_series.push(self.compute_syntactic_complexity_ratio());
    }

    /// Compute HD-D and MTLD over the rolling token window.
    fn compute_lexical_diversity(&self) -> crate::components::LexicalDiversitySnapshot {
        let tokens = &self.token_window;
        let token_count = tokens.len() as u32;
        if token_count == 0 {
            return crate::components::LexicalDiversitySnapshot::default();
        }

        // Frequency map over the window.
        let mut freq: HashMap<String, u32> = HashMap::new();
        for token in tokens {
            *freq.entry(token.clone()).or_insert(0) += 1;
        }

        let hd_d = Self::compute_hd_d(tokens, &freq);
        let mtld = Self::compute_mtld(tokens);

        crate::components::LexicalDiversitySnapshot { hd_d, mtld, token_count }
    }

    /// Hypergeometric Distribution D (HD-D).
    ///
    /// For each word type in the window, computes the probability that a random sample of
    /// `sample_size` tokens contains *at least one* occurrence of that type. The HD-D score
    /// is the average of these probabilities across all types, which mitigates text-length
    /// bias better than raw TTR.
    fn compute_hd_d(tokens: &[String], freq: &HashMap<String, u32>) -> f32 {
        let n = tokens.len() as u32;
        if n == 0 {
            return 0.0;
        }
        let sample_size = (HD_D_SAMPLE_SIZE as u32).min(n);

        let mut sum = 0.0f64;
        for count in freq.values() {
            let count = *count as u32;
            // Probability of drawing zero of this type in a sample of size `sample_size`.
            let prob_zero = hypergeometric_prob_zero(n, count, sample_size);
            // We want probability of at least one.
            sum += 1.0 - prob_zero;
        }

        let types = freq.len() as f64;
        if types > 0.0 {
            (sum / types) as f32
        } else {
            0.0
        }
    }

    /// Measure of Textual Lexical Diversity (MTLD) using a factor-size procedure.
    ///
    /// Iterates over the token stream, accumulating tokens into factors of `MTLD_FACTOR_SIZE`
    /// unique types. The MTLD is the average number of tokens required to reach each factor.
    fn compute_mtld(tokens: &[String]) -> f32 {
        if tokens.len() < MTLD_FACTOR_SIZE {
            return 0.0;
        }

        let mut total_factors = 0usize;
        let mut total_tokens_in_factors = 0usize;
        let mut idx = 0usize;

        while idx < tokens.len() {
            let mut seen = std::collections::HashSet::new();
            let mut factor_tokens = 0usize;
            while seen.len() < MTLD_FACTOR_SIZE && idx < tokens.len() {
                seen.insert(tokens[idx].clone());
                factor_tokens += 1;
                idx += 1;
            }
            if seen.len() >= MTLD_FACTOR_SIZE {
                total_factors += 1;
                total_tokens_in_factors += factor_tokens;
            }
        }

        if total_factors > 0 {
            (total_tokens_in_factors as f32) / (total_factors as f32)
        } else {
            0.0
        }
    }

    /// Syntactic complexity ratio: combo casts / total casts.
    fn compute_syntactic_complexity_ratio(&self) -> f32 {
        let total = self.telemetry.cast_log.len() as f32;
        if total == 0.0 {
            return 0.0;
        }
        let combos = self.telemetry.cast_log.iter().filter(|e| e.combo).count() as f32;
        combos / total
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
    pub grades: GradeScores,
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
    slime_level: &mut SlimeLevel,
) -> BattleResult {
    let lower_typo = session.typo_word.to_lowercase();
    let lower_played = played_word.to_lowercase();

    let mut damage_multiplier = 1.0;
    let mut is_effective = false;
    let mut is_counter = false;
    let mut is_synonym = false;

    // Grade-tracking state.
    let mut semantic_distance_value: Option<f32> = None;
    let mut typo_pos: Option<String> = None;
    let mut played_pos: Option<String> = None;
    let mut contextual_resonance: Option<f32> = None;
    let mut cast_device: Option<String> = None;

    if let (Some(typo_stats), Some(played_stats)) = (db.words.get(&lower_typo), db.words.get(&lower_played)) {
        let distance = semantic_distance(typo_stats, played_stats);
        semantic_distance_value = Some(distance);
        typo_pos = Some(typo_stats.part_of_speech.to_lowercase());
        played_pos = Some(played_stats.part_of_speech.to_lowercase());

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
        match played_pos.as_deref() {
            Some("noun") => {
                damage_multiplier *= NOUN_SUMMON_MULTIPLIER;
                info!("Noun: Summon/Target base damage");
            }
            Some("adjective") => {
                damage_multiplier *= ADJECTIVE_AURA_MULTIPLIER;
                info!("Adjective: Aura/Element multiplier applied");
            }
            Some("verb") => {
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
            cast_device = Some("oxymoron".to_string());
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
            cast_device = Some("hyperbole".to_string());
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
            cast_device = Some("palindrome".to_string());
            if let Some(ref mut metrics) = vaam_metrics {
                metrics.record_literary_device("palindrome");
            }
        }
    }

    // Pillar 2: Apply FACES emotional resonance.
    // The word's intrinsic FACES is compared to the Slime's contextual FACES.
    // High alignment = resonant cast (bonus damage). Low alignment = dissonant cast (penalty).
    if let Some(face) = active_face {
        if let Some(entry) = spellbook.entries.iter().find(|e| e.word == lower_played) {
            if let Some(intrinsic_faces) = entry.faces {
                let resonance = compute_resonance(intrinsic_faces.0, face.faces);
                contextual_resonance = Some(resonance);
                let res_mult = resonance_multiplier(resonance);
                damage_multiplier *= res_mult;
                info!("FACES resonance: {:.2} → multiplier {:.2}", resonance, res_mult);
            } else {
                // Word has no cached FACES; fall back to nearest preset modifier.
                let preset = nearest_slime_face_preset(face.faces);
                let preset_mult = match preset {
                    SlimeFace::Fierce => 1.2,
                    SlimeFace::Joyful => 1.1,
                    SlimeFace::Calm => 1.0,
                    SlimeFace::Angry => 1.3,
                };
                damage_multiplier *= preset_mult;
                info!("No intrinsic FACES for '{}'; using preset {:?} modifier {:.2}", played_word, preset, preset_mult);
            }
        } else {
            // Word not in spellbook: neutral FACES contribution.
            info!("Word '{}' not in SpellBook; neutral FACES modifier", played_word);
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

        // Award card XP and global Slime XP for effective casts.
        let xp_amount = (final_damage.max(1) as u32) + 2;
        spellbook.add_card_xp(played_word, xp_amount);
        slime_level.add_xp(xp_amount);

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

    // Compute three-axis grades for this cast.
    let syntax_score = match (typo_pos.as_deref(), played_pos.as_deref()) {
        (Some(t), Some(p)) if t == p => 0.7, // same POS
        (Some("noun"), Some("verb")) | (Some("verb"), Some("noun")) => 1.0, // complementary
        (Some("adjective"), Some("noun")) | (Some("noun"), Some("adjective")) => 1.0,
        (Some(_), Some(_)) => 0.4,
        _ => 0.0,
    };

    let semantics_score = match semantic_distance_value {
        Some(d) if d < 2.0 => 1.0,  // synonym
        Some(d) if d > 4.0 => 0.9,  // strong antonym counter
        Some(_) => 0.5,             // mid-range
        None => 0.0,
    };

    let pragmatics_score = contextual_resonance.unwrap_or(0.0);

    let grades = GradeScores {
        syntax: syntax_score,
        semantics: semantics_score,
        pragmatics: pragmatics_score,
    };

    info!("Grade scores — syntax: {:.2}, semantics: {:.2}, pragmatics: {:.2}", grades.syntax, grades.semantics, grades.pragmatics);

    // Record stealth-assessment telemetry for this cast.
    if let Some(ref mut metrics) = vaam_metrics {
        let mut ccss_tags = db.word_ccss_tags(played_word);
        if contextual_resonance.is_some() {
            ccss_tags.push(crate::components::ccss::L_9_10_5.to_string());
        }
        if cast_device.is_some() {
            ccss_tags.push(crate::components::ccss::L_11_12_3.to_string());
        }
        // Deduplicate while preserving order.
        let mut unique_tags = Vec::new();
        for tag in ccss_tags {
            if !unique_tags.contains(&tag) {
                unique_tags.push(tag);
            }
        }
        let event = CastTelemetry {
            word: lower_played,
            pos: played_pos.clone(),
            grades,
            faces_resonance: contextual_resonance.unwrap_or(0.0),
            effective: is_effective,
            combo: cast_device.is_some(),
            device: cast_device.clone(),
            ccss_tags: unique_tags,
            subject: None,
            sequence: metrics.telemetry.cast_log.len() as u64,
        };
        metrics.record_cast_telemetry(event);
    }

    BattleResult {
        is_effective,
        is_counter,
        is_synonym,
        grades,
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
    slime_level: &mut SlimeLevel,
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
    let mut sentence_devices: Vec<String> = Vec::new();

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
            sentence_devices.push("oxymoron".to_string());
            if let Some(ref mut metrics) = vaam_metrics {
                metrics.record_literary_device("oxymoron");
            }
        }
        if is_hyperbole(word, db) {
            sentence_multiplier *= HYPERBOLE_OVERCHARGE;
            recoil += HYPERBOLE_RECOIL_DAMAGE;
            sentence_devices.push("hyperbole".to_string());
            if let Some(ref mut metrics) = vaam_metrics {
                metrics.record_literary_device("hyperbole");
            }
        }
        if is_palindrome(word) {
            reflection += (BASE_DAMAGE * card_multiplier * PALINDROME_REFLECTION_PERCENT) as i32;
            sentence_devices.push("palindrome".to_string());
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
        sentence_devices.push("alliteration".to_string());
        if let Some(ref mut metrics) = vaam_metrics {
            metrics.record_literary_device("alliteration");
        }
    }

    // Apply FACES emotional resonance once to the whole sentence.
    // The average resonance across all played words sets the sentence-level mood multiplier.
    if let Some(face) = active_face {
        let mut total_resonance = 0.0;
        let mut words_with_faces = 0usize;
        for word in &plot.cards {
            if let Some(entry) = spellbook.entries.iter().find(|e| e.word == word.to_lowercase()) {
                if let Some(intrinsic_faces) = entry.faces {
                    total_resonance += compute_resonance(intrinsic_faces.0, face.faces);
                    words_with_faces += 1;
                }
            }
        }
        if words_with_faces > 0 {
            let avg_resonance = total_resonance / words_with_faces as f32;
            sentence_multiplier *= resonance_multiplier(avg_resonance);
            info!("Sentence FACES resonance: {:.2}", avg_resonance);
        } else {
            // Fallback to nearest preset if no words have cached FACES.
            let preset = nearest_slime_face_preset(face.faces);
            let preset_mult = match preset {
                SlimeFace::Fierce => 1.2,
                SlimeFace::Joyful => 1.1,
                SlimeFace::Calm => 1.0,
                SlimeFace::Angry => 1.3,
            };
            sentence_multiplier *= preset_mult;
            info!("No intrinsic FACES in sentence; using preset {:?} modifier {:.2}", preset, preset_mult);
        }
    }

    let final_damage = (total_damage as f32 * sentence_multiplier) as i32 + reflection;
    session.typo_health -= final_damage;
    session.player_health -= recoil;

    info!("CAST SENTENCE: '{}' deals {} damage. Typo health: {}/{}",
        plot.cards.join(" "), final_damage, session.typo_health, TYPO_MAX_HEALTH);

    // Award card XP for each word in the sentence plus global Slime XP.
    let sentence_xp = (final_damage.max(1) as u32) + 5;
    let sentence_combo = !sentence_devices.is_empty();
    for word in &plot.cards {
        spellbook.add_card_xp(word, 3);
    }
    slime_level.add_xp(sentence_xp);

    // Record stealth-assessment telemetry for each word in the sentence.
    let avg_resonance = if let Some(face) = active_face {
        let mut total = 0.0f32;
        let mut count = 0usize;
        for word in &plot.cards {
            if let Some(entry) = spellbook.entries.iter().find(|e| e.word == word.to_lowercase()) {
                if let Some(intrinsic_faces) = entry.faces {
                    total += compute_resonance(intrinsic_faces.0, face.faces);
                    count += 1;
                }
            }
        }
        if count > 0 { total / count as f32 } else { 0.0 }
    } else {
        0.0
    };

    if let Some(ref mut metrics) = vaam_metrics {
        let mut seq = metrics.telemetry.cast_log.len() as u64;
        for word in &plot.cards {
            let pos = db.words.get(&word.to_lowercase()).map(|s| s.part_of_speech.to_lowercase());
            let mut ccss_tags = db.word_ccss_tags(word);
            ccss_tags.push(crate::components::ccss::L_9_10_5.to_string());
            if sentence_combo {
                ccss_tags.push(crate::components::ccss::L_11_12_3.to_string());
            }
            let mut unique_tags = Vec::new();
            for tag in ccss_tags {
                if !unique_tags.contains(&tag) {
                    unique_tags.push(tag);
                }
            }
            let event = CastTelemetry {
                word: word.to_lowercase(),
                pos,
                grades: GradeScores::default(),
                faces_resonance: avg_resonance,
                effective: true,
                combo: sentence_combo,
                device: sentence_devices.first().cloned(),
                ccss_tags: unique_tags,
                subject: None,
                sequence: seq,
            };
            seq += 1;
            metrics.record_cast_telemetry(event);
        }
    }

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
            grades: GradeScores::default(),
        };
        assert!(!result.is_effective);
        assert!(!result.is_counter);
        assert!(!result.is_synonym);
        assert_eq!(result.grades, GradeScores::default());
    }

    #[test]
    fn nearest_preset_round_trips_slime_face_presets() {
        use crate::components::SlimeFace;

        assert_eq!(nearest_slime_face_preset(SlimeFace::Fierce.to_faces_state()), SlimeFace::Fierce);
        assert_eq!(nearest_slime_face_preset(SlimeFace::Joyful.to_faces_state()), SlimeFace::Joyful);
        assert_eq!(nearest_slime_face_preset(SlimeFace::Calm.to_faces_state()), SlimeFace::Calm);
        assert_eq!(nearest_slime_face_preset(SlimeFace::Angry.to_faces_state()), SlimeFace::Angry);
    }

    #[test]
    fn identical_faces_states_have_perfect_resonance() {
        let state = FacesState::default();
        let resonance = compute_resonance(state, state);
        assert!((resonance - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn orthogonal_faces_states_have_low_resonance() {
        use faces_protocol::{Action, Aura, Container, Focus};

        let calm = FacesState::new(Aura::CALM, Container::Neutral, Focus::Neutral, Action::Thoughtful);
        let fierce = FacesState::new(Aura::URGENT, Container::Sharp, Focus::Intense, Action::Assertive);
        let resonance = compute_resonance(calm, fierce);
        assert!(resonance < 0.5);
    }

    #[test]
    fn resonance_multiplier_scales_with_alignment() {
        assert_eq!(resonance_multiplier(1.0), 1.5);
        assert_eq!(resonance_multiplier(0.6), 1.1);
        assert_eq!(resonance_multiplier(0.3), 1.0);
        assert_eq!(resonance_multiplier(0.0), 0.7);
    }

    #[test]
    fn play_battle_card_returns_three_axis_grades() {
        let db = GameDatabase::load_from_embedded().unwrap();
        let mut session = BattleSession {
            typo_word: "abandoned".to_string(),
            typo_health: 100,
            player_health: 100,
            failed_word: None,
        };
        let mut spellbook = SpellBook::default();
        spellbook.record_encounter("left", Channel::Mind, None, None, None, None);
        let mut next_state = NextState::default();
        let sheet = CharacterSheet::default();
        let state = State::new(GameState::Battling);
        let mut vaam = VaamMetrics::default();
        let active_face = ActiveFace {
            face: SlimeFace::Angry,
            faces: SlimeFace::Angry.to_faces_state(),
        };

        let mut slime_level = SlimeLevel::default();
        let result = play_battle_card("left", &mut session, &db, &mut spellbook, &mut next_state, &sheet, &state, Some(&active_face), Some(&mut vaam), &mut slime_level);

        assert!(result.grades.syntax >= 0.0 && result.grades.syntax <= 1.0);
        assert!(result.grades.semantics >= 0.0 && result.grades.semantics <= 1.0);
        assert!(result.grades.pragmatics >= 0.0 && result.grades.pragmatics <= 1.0);
        assert!(slime_level.xp > 0, "slime should gain XP from an effective cast");
        let entry = spellbook.entries.iter().find(|e| e.word == "left").unwrap();
        assert!(entry.card_xp > 0, "word card should gain XP from an effective cast");
    }

    #[test]
    fn hypergeometric_prob_zero_is_one_when_no_successes() {
        assert!((hypergeometric_prob_zero(10, 0, 5) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hypergeometric_prob_zero_decreases_with_more_successes() {
        let p1 = hypergeometric_prob_zero(20, 2, 5);
        let p2 = hypergeometric_prob_zero(20, 10, 5);
        assert!(p2 < p1, "higher success count should reduce zero-draw probability");
    }

    #[test]
    fn mtld_is_zero_for_short_input() {
        let tokens: Vec<String> = vec!["a".to_string(), "b".to_string()];
        assert_eq!(VaamMetrics::compute_mtld(&tokens), 0.0);
    }

    #[test]
    fn mtld_grows_with_repeated_diverse_tokens() {
        // 10 unique tokens should form exactly one factor of size 10.
        let tokens: Vec<String> = (0..10).map(|i| format!("word{}", i)).collect();
        let mtld = VaamMetrics::compute_mtld(&tokens);
        assert_eq!(mtld, 10.0, "10 unique tokens should require 10 tokens per factor");
    }

    #[test]
    fn hd_d_is_perfect_when_population_is_fully_sampled() {
        // If every token in the population is unique, drawing the full population
        // guarantees at least one of every type, so HD-D equals 1.0.
        let tokens: Vec<String> = (0..20).map(|i| format!("word{}", i)).collect();
        let mut freq = HashMap::new();
        for i in 0..20 {
            freq.insert(format!("word{}", i), 1);
        }
        let hd_d = VaamMetrics::compute_hd_d(&tokens, &freq);
        assert!((hd_d - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hd_d_matches_known_hypergeometric_property() {
        // Population 100, one rare type appearing once, sample 42.
        // P(at least one) = 1 - C(99, 42) / C(100, 42) = 1 - 58/100 = 0.42.
        let tokens: Vec<String> = (0..100).map(|i| if i == 0 { "rare".to_string() } else { "common".to_string() }).collect();
        let mut freq = HashMap::new();
        freq.insert("rare".to_string(), 1);
        freq.insert("common".to_string(), 99);
        let hd_d = VaamMetrics::compute_hd_d(&tokens, &freq);
        // Two types. Rare type ~0.42, common type ~1.0. Average ~0.71.
        assert!(hd_d > 0.6 && hd_d < 0.85, "HD-D should be the average of the two type probabilities");
    }

    #[test]
    fn cast_telemetry_updates_series() {
        let mut vaam = VaamMetrics::default();
        let event = CastTelemetry {
            word: "brave".to_string(),
            pos: Some("adjective".to_string()),
            grades: GradeScores { syntax: 0.8, semantics: 0.9, pragmatics: 0.7 },
            faces_resonance: 0.75,
            effective: true,
            combo: true,
            device: Some("echo".to_string()),
            ccss_tags: vec!["L.9-10.5".to_string()],
            subject: Some("tone".to_string()),
            sequence: 1,
        };
        vaam.record_cast_telemetry(event);

        assert_eq!(vaam.telemetry.cast_log.len(), 1);
        assert_eq!(vaam.telemetry.syntax_series, vec![0.8]);
        assert_eq!(vaam.telemetry.pragmatics_series, vec![0.75]);
        assert_eq!(vaam.telemetry.syntactic_complexity_series, vec![1.0]);
        assert_eq!(vaam.ccss_coverage.get("L.9-10.5"), Some(&1));
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

        #[cfg(feature = "flat2d")]
        app.add_systems(Update, handle_critical_hit_effects_2d);

        app.add_systems(Update, play_battle_sfx);

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
               update_graybox_feedback_ui,
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

pub fn play_battle_sfx(
    trigger_query: Query<Entity, With<CriticalHitTrigger>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for _ in trigger_query.iter() {
        commands.spawn((
            AudioPlayer::<AudioSource>(asset_server.load(crate::asset_catalog::SOUND_ATTUNE)),
            PlaybackSettings::DESPAWN,
        ));
    }
}

#[cfg(feature = "flat2d")]
pub fn handle_critical_hit_effects_2d(
    trigger_query: Query<Entity, With<CriticalHitTrigger>>,
    mut commands: Commands,
    camera_query: Query<(Entity, &Transform), With<Camera2d>>,
) {
    for trigger_entity in trigger_query.iter() {
        commands.entity(trigger_entity).despawn();

        for (entity, tf) in &camera_query {
            commands.entity(entity).insert(crate::render::ScreenShake {
                timer: 0.3,
                intensity: 8.0,
                base_translation: tf.translation,
            });
        }

        use rand::Rng;
        let mut rng = rand::thread_rng();
        for _ in 0..24 {
            let vx = rng.gen_range(-120.0..120.0);
            let vy = rng.gen_range(-120.0..120.0);
            commands.spawn((
                Sprite {
                    color: Color::srgb(rng.gen(), rng.gen(), 1.0),
                    custom_size: Some(Vec2::splat(rng.gen_range(4.0..10.0))),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 10.0),
                crate::render::BurstParticle {
                    velocity: Vec2::new(vx, vy),
                    timer: rng.gen_range(0.4..0.8),
                },
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

        // FACES emotion buttons
        parent.spawn((
            Node {
                margin: UiRect::top(Val::Px(12.0)),
                justify_content: JustifyContent::SpaceEvenly,
                width: Val::Px(360.0),
                ..default()
            },
        )).with_children(|face_parent| {
            let faces = [SlimeFace::Fierce, SlimeFace::Joyful, SlimeFace::Calm, SlimeFace::Angry];
            for face in faces {
                face_parent.spawn((
                    Button,
                    FaceButton { face },
                    Node {
                        width: Val::Px(80.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.5, 0.3)),
                    BattleUiMarker,
                )).with_children(|btn| {
                    btn.spawn((
                        Text::new(format!("{:?}", face)),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });

        // Clear Plot button
        parent.spawn((
            Button,
            crate::components::ClearPlotButton,
            BattleUiMarker,
            Node {
                margin: UiRect::top(Val::Px(10.0)),
                width: Val::Px(120.0),
                height: Val::Px(32.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.4, 0.2, 0.2)),
        )).with_children(|btn| {
            btn.spawn((
                Text::new("Clear Plot"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });

        // Hint line
        parent.spawn((
            Text::new("Click cards to build a 3-word sentence, then Cast Spell."),
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

#[derive(Component)]
struct GrayBoxFeedbackText;

#[derive(Resource, Default)]
pub(crate) struct GrayBoxFeedback {
    last_cast_summary: Option<String>,
}

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
        parent.spawn((
            Text::new("Slime Face: Calm\nCast a spell to see grade breakdown."),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
            GrayBoxFeedbackText,
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
    commands.insert_resource(GrayBoxFeedback::default());

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
            active_face.faces = face_button.face.to_faces_state();
            info!("Face changed to: {:?} ({})", face_button.face, active_face.faces);
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
    mut feedback: ResMut<GrayBoxFeedback>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            if let Ok(altar) = altar_query.single() {
                if let Some(ref word) = altar.active_card {
                    let (damage, math) = calculate_spell_damage(
                        word,
                        &battle_session.prompt_word,
                        active_face.faces,
                    );
                    battle_session.dummy_hp -= damage;

                    // Simple three-axis grade estimate for the gray-box demo.
                    let semantics = if is_synonym(word, &battle_session.prompt_word) {
                        1.0
                    } else if is_antonym(word, &battle_session.prompt_word) {
                        0.9
                    } else {
                        0.5
                    };
                    let face = nearest_slime_face_preset(active_face.faces);
                    let face_modifier = match face {
                        SlimeFace::Fierce => 1.2,
                        SlimeFace::Joyful => 1.1,
                        SlimeFace::Calm => 1.0,
                        SlimeFace::Angry => 1.3,
                    };
                    let pragmatics = ((face_modifier - 0.7f32) / 0.8f32).clamp(0.0f32, 1.0f32);
                    let syntax = 0.7f32;

                    feedback.last_cast_summary = Some(format!(
                        "Cast: {}\nSlime Face: {:?}\nDamage: {}\nMath: {}\nGrades — Syntax: {:.2}, Semantics: {:.2}, Pragmatics: {:.2}",
                        word, face, damage, math, syntax, semantics, pragmatics
                    ));
                    info!("Spell cast: {} with faces {} - Damage: {} ({})", word, active_face.faces, damage, math);
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

/// Compute FACES resonance between a word's intrinsic FACES and the Slime's
/// contextual FACES.
///
/// Returns a score from 0.0 (complete dissonance) to 1.0 (perfect resonance).
/// The calculation uses byte-level comparison: exact matches on the structured
/// bytes (container, focus, action) score highest; the aura byte uses inverse
/// distance on the ANSI-256 color wheel.
pub fn compute_resonance(intrinsic: FacesState, contextual: FacesState) -> f32 {
    let ib = intrinsic.to_bytes();
    let cb = contextual.to_bytes();

    // Aura (byte 0): color distance on a 0-255 scale, inverted.
    let aura_distance = (ib[0] as f32 - cb[0] as f32).abs();
    let aura_score = 1.0 - (aura_distance / 255.0);

    // Container (byte 1), Focus (byte 2), Action (byte 3): exact matches.
    let container_score = if ib[1] == cb[1] { 1.0 } else { 0.0 };
    let focus_score = if ib[2] == cb[2] { 1.0 } else { 0.0 };
    let action_score = if ib[3] == cb[3] { 1.0 } else { 0.0 };

    // Weighted blend. Aura provides subtle shading; container/focus/action
    // carry the grammatical-emotional shape of the word.
    aura_score * 0.20 + container_score * 0.30 + focus_score * 0.25 + action_score * 0.25
}

/// Map a resonance score to a damage multiplier.
pub fn resonance_multiplier(resonance: f32) -> f32 {
    if resonance > 0.85 {
        1.5 // resonant cast
    } else if resonance > 0.55 {
        1.1 // partial alignment
    } else if resonance > 0.25 {
        1.0 // neutral
    } else {
        0.7 // dissonant cast
    }
}

/// Calculate spell damage with face modifiers.
///
/// The face modifier is derived from the Slime's contextual FACES register.
/// In the 2D demo this maps back to the nearest preset; in the full game it
/// will be computed from byte-level resonance.
pub fn calculate_spell_damage(
    played_word: &str,
    prompt_word: &str,
    faces: FacesState,
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

    let face = nearest_slime_face_preset(faces);
    let face_modifier = match face {
        SlimeFace::Fierce => 1.2,
        SlimeFace::Joyful => 1.1,
        SlimeFace::Calm => 1.0,
        SlimeFace::Angry => 1.3,
    };

    let final_damage = (base_damage as f32 * multiplier * face_modifier) as i32;

    let math_breakdown = format!(
        "Base: {} × DistMult: {:.2} × FaceMod({:?}): {:.2} = {}",
        base_damage, multiplier, face, face_modifier, final_damage
    );

    (final_damage, math_breakdown)
}

/// Maps a full FACES register to the nearest 2D demo preset.
///
/// This is a temporary compatibility helper. The full game will use the raw
/// FACES bytes for resonance instead of collapsing back to a preset.
pub fn nearest_slime_face_preset(faces: FacesState) -> SlimeFace {
    use faces_protocol::{Action, Aura, Container, Focus};

    match (faces.aura, faces.container, faces.focus, faces.action) {
        // Calm preset: blue/calm aura, neutral boundary, neutral attention, thoughtful output.
        (Aura::CALM, Container::Neutral, Focus::Neutral, Action::Thoughtful) => SlimeFace::Calm,
        // Joyful preset: happy/yellow aura, fluid boundary, happy focus, playful output.
        (Aura::HAPPY, Container::Fluid, Focus::Happy, Action::Playful) => SlimeFace::Joyful,
        // Angry preset: urgent/red aura, sharp boundary, intense focus, assertive output.
        (Aura::URGENT, Container::Sharp, Focus::Intense, Action::Assertive) => SlimeFace::Angry,
        // Fierce preset: energetic/orange aura, sharp boundary, intense focus, assertive output.
        (Aura::ENERGETIC, Container::Sharp, Focus::Intense, Action::Assertive) => SlimeFace::Fierce,
        // Fallback: choose by aura temperature.
        _ => {
            let bytes = faces.to_bytes();
            match bytes[0] {
                0..=60 | 200..=220 => SlimeFace::Joyful, // warm yellow / bright
                61..=140 => SlimeFace::Angry,            // warm red / urgent
                141..=200 => SlimeFace::Calm,            // cool blue / calm
                _ => SlimeFace::Fierce,                  // energetic / intense
            }
        }
    }
}

/// Apply damage to target dummy (no longer needed with direct spell casting)
pub fn apply_damage_to_dummy(
    _battle_session: Res<GrayBoxBattleSession>,
) {
    // Damage is now applied directly in handle_cast_spell_click
    // This function is kept for future event-based integration
}

/// Update the 2D gray-box combat log with the current Slime face and latest cast feedback.
#[cfg(feature = "flat2d")]
fn update_graybox_feedback_ui(
    active_face: Res<ActiveFace>,
    feedback: Res<GrayBoxFeedback>,
    mut text_query: Query<&mut Text, With<GrayBoxFeedbackText>>,
) {
    let face = nearest_slime_face_preset(active_face.faces);
    let summary = feedback.last_cast_summary.as_deref().unwrap_or("Cast a spell to see grade breakdown.");
    for mut text in &mut text_query {
        text.0 = format!("Slime Face: {:?}\n{}", face, summary);
    }
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

