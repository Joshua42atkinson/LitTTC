// integration_tests.rs — End-to-End Integration and Pedagogical Tests
#![allow(unused)]
use communication_class::components::*;
use communication_class::database::GameDatabase;
use communication_class::quest::{self, QuestSession};
use communication_class::battle::{self, BattleSession};
use communication_class::save::{self, SaveData};
use bevy::prelude::*;
use std::collections::HashMap;

#[test]
fn test_database_loading() {
    let db = GameDatabase::load_from_embedded();
    assert!(db.is_ok(), "Embedded JSON database should load successfully");
    let database = db.unwrap();
    
    assert!(database.words.contains_key("abandoned"), "Database should contain 'abandoned'");
    assert!(database.synonyms.contains_key("abandoned"), "Synonyms should contain 'abandoned'");
    assert!(database.etymology.roots.contains_key("Ignis"), "Etymology roots should contain 'Ignis'");
    assert!(database.quests.npc_chains.contains_key("Barnaby"), "Quests should contain 'Barnaby' chain");
}

#[test]
fn test_quest_progression() {
    let mut db = GameDatabase::load_from_embedded().unwrap();
    let mut queue1 = bevy::ecs::world::CommandQueue::default();
    let mut world1 = World::new();
    let mut commands = Commands::new(&mut queue1, &world1);
    
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.insert_resource(db);
    app.insert_resource(CharacterSheet::default());
    app.insert_resource(SpellBook::default());
    
    let mut next_state = app.world_mut().resource_mut::<NextState<GameState>>();
    
    // Simulate starting quest
    let npc_quest = app.world().resource::<GameDatabase>().quests.npc_chains.get("Barnaby").unwrap().first().unwrap().clone();
    
    let mut session = QuestSession {
        npc_name: "Barnaby".to_string(),
        title: npc_quest.title.clone(),
        template: npc_quest.template.clone(),
        slots: vec!["ADJECTIVE".to_string()],
        filled_slots: HashMap::new(),
        xp_reward: npc_quest.rewards.xp,
        evolution_reward: npc_quest.rewards.evolution_points,
    };
    
    // Fill slot and complete
    quest::fill_slot(0, "patience", None, &mut session);
    assert_eq!(session.filled_slots.get(&0).unwrap().0, "patience");
    
    let mut sheet = CharacterSheet::default();
    let mut spellbook = SpellBook::default();
    let mut test_next_state = NextState::default();
    let mut queue2 = bevy::ecs::world::CommandQueue::default();
    let mut world2 = World::new();
    let mut test_commands = Commands::new(&mut queue2, &world2);
    
    quest::complete_quest(
        &session,
        &mut sheet,
        &mut spellbook,
        &mut test_next_state,
        &mut test_commands,
    );
    
    assert_eq!(sheet.total_xp, npc_quest.rewards.xp as u64);
    assert_eq!(sheet.words_encountered, 1);
}

#[test]
fn test_battle_combat_mechanics() {
    let db = GameDatabase::load_from_embedded().unwrap();
    let mut session = BattleSession {
        typo_word: "abandoned".to_string(),
        typo_health: 100,
        player_health: 100,
    };
    
    let mut spellbook = SpellBook::default();
    let mut next_state = NextState::default();
    
    // Play a word with high semantic distance (effective): "abc" vs "abandoned"
    let is_effective_1 = battle::play_battle_card("abc", &mut session, &db, &mut spellbook, &mut next_state);
    assert!(is_effective_1, "Playing abc should be semantically effective");
    assert_eq!(session.typo_health, 48, "Effective card should damage typo");
    assert_eq!(session.player_health, 100, "Effective card should not damage player");
    
    // Play a word with low semantic distance (ineffective): "abandoned" vs "abandoned"
    let is_effective_2 = battle::play_battle_card("abandoned", &mut session, &db, &mut spellbook, &mut next_state);
    assert!(!is_effective_2, "Playing identical word should be ineffective");
    assert_eq!(session.typo_health, 36, "Ineffective card should do minor damage");
    assert_eq!(session.player_health, 80, "Ineffective card should result in typo counter-attack");
}

#[test]
fn test_local_save_system() {
    let sheet = CharacterSheet {
        mind_attunement: 0.55,
        heart_attunement: 0.2,
        body_attunement: 0.1,
        action_attunement: 0.05,
        emergent_class: "The Oracle".to_string(),
        words_encountered: 12,
        total_deeper_swipes: 4,
        total_xp: 450,
    };
    
    let mut spellbook = SpellBook::default();
    spellbook.record_encounter("clarity", Channel::Mind);
    spellbook.upgrade_mastery("clarity", MasteryLevel::Mastered);
    
    let trail = StudentTrail {
        visited_words: vec!["clarity".to_string(), "abandoned".to_string()],
        swipe_history: vec![],
        current_word: Some("clarity".to_string()),
    };
    
    // Test saving
    let save_res = save::save_game(&sheet, &spellbook, &trail);
    assert!(save_res.is_ok(), "Saving progress to disk should succeed");
    
    // Test loading
    let load_res = save::load_game();
    assert!(load_res.is_ok(), "Loading progress from disk should succeed");
    
    let loaded = load_res.unwrap();
    assert_eq!(loaded.character_sheet.emergent_class, "The Oracle");
    assert_eq!(loaded.character_sheet.total_xp, 450);
    assert_eq!(loaded.student_trail.visited_words.len(), 2);
    
    // Clean up temporary save file
    let _ = std::fs::remove_file("save.json");
}

#[test]
fn test_pet_chat_taming() {
    use communication_class::chat::{self, ChatLog};
    use faces_protocol::{Focus, Action};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ChatLog>();

    let mut chat_log = app.world().resource::<ChatLog>().clone();
    assert_eq!(chat_log.messages.len(), 0);

    // Simulate taming actions directly
    let focus = Focus::Happy;
    let action = Action::Playful;
    let element = Element::Normal;

    let dialogue = chat::get_pet_dialogue("wisdom", focus, action, element);
    chat_log.add_message("wisdom", &dialogue, 0.0);

    assert_eq!(chat_log.messages.len(), 1);
    assert!(chat_log.messages[0].text.contains("wisdom"));
    assert!(chat_log.messages[0].text.contains("bounce-bounce-bouncy"));
}

#[test]
fn test_curriculum_and_dialogue() {
    use communication_class::quest::{self, CurriculumManager};

    let db = GameDatabase::load_from_embedded().unwrap();
    
    // 1. Verify dynamic NPC dialogue matching
    let dialogue_dawn = quest::get_npc_dialogue("Peter", &db, "Dawn");
    let dialogue_night = quest::get_npc_dialogue("Peter", &db, "Night");
    
    assert!(!dialogue_dawn.is_empty());
    assert!(!dialogue_night.is_empty());

    // 2. Verify CurriculumManager grade mapping
    let mut curriculum = CurriculumManager {
        active_grade: 1,
        progress_xp: 0,
    };
    
    assert_eq!(curriculum.get_grade_level(), "Grade 1");
    
    // Gain 1200 XP (threshold for Grade 2 is 1000 XP)
    let graded_up = curriculum.check_grade_up(1200);
    assert!(graded_up);
    assert_eq!(curriculum.get_grade_level(), "Grade 2");
}

