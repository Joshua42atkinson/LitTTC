// save.rs — Simple local JSON saving and loading of student progress
#![allow(dead_code)]
use std::fs::File;
use std::io::{Read, Write};
use bevy::prelude::*;
use crate::components::*;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SaveData {
    pub character_sheet: CharacterSheet,
    pub spellbook: SpellBook,
    pub student_trail: StudentTrail,
}

pub fn save_game(
    sheet: &CharacterSheet,
    spellbook: &SpellBook,
    trail: &StudentTrail,
) -> Result<(), std::io::Error> {
    let data = SaveData {
        character_sheet: sheet.clone(),
        spellbook: SpellBook { entries: spellbook.entries.clone() },
        student_trail: trail.clone(),
    };

    let serialized = serde_json::to_string_pretty(&data)?;
    
    let mut file = File::create("save.json")?;
    file.write_all(serialized.as_bytes())?;
    
    Ok(())
}

pub fn load_game() -> Result<SaveData, std::io::Error> {
    let mut file = File::open("save.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let data: SaveData = serde_json::from_str(&contents)?;
    Ok(data)
}

pub fn auto_save_system(
    sheet: Res<CharacterSheet>,
    spellbook: Res<SpellBook>,
    trail: Res<StudentTrail>,
    demo: Res<crate::paywall::DemoSettings>,
) {
    if demo.is_demo {
        return; // Disable saving in demo mode
    }

    if let Err(e) = save_game(&sheet, &spellbook, &trail) {
        warn!("Failed to auto-save: {}", e);
    } else {
        info!("Auto-saved progress.");
    }
}
