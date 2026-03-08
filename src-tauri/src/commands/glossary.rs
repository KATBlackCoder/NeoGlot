// CRUD glossaire + import speakers + QC post-traduction
// Implémenté en T08

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct GlossaryEntry {
    pub id: i64,
    pub project_id: Option<i64>,
    pub term: String,
    pub translation: String,
    pub note: String,
    pub match_mode: String,
}

#[derive(Deserialize)]
pub struct NewGlossaryEntry {
    pub project_id: Option<i64>,
    pub term: String,
    pub translation: String,
    pub note: Option<String>,
    pub match_mode: String,
}

#[tauri::command]
pub fn list_glossary(_project_id: Option<i64>) -> Result<Vec<GlossaryEntry>, String> {
    Err("Non implémenté — voir T08".into())
}

#[tauri::command]
pub fn add_glossary_term(_entry: NewGlossaryEntry) -> Result<i64, String> {
    Err("Non implémenté — voir T08".into())
}

#[tauri::command]
pub fn update_glossary_term(_id: i64, _entry: NewGlossaryEntry) -> Result<(), String> {
    Err("Non implémenté — voir T08".into())
}

#[tauri::command]
pub fn delete_glossary_term(_id: i64) -> Result<(), String> {
    Err("Non implémenté — voir T08".into())
}

#[tauri::command]
pub fn import_speakers(_project_id: i64) -> Result<usize, String> {
    Err("Non implémenté — voir T08".into())
}
