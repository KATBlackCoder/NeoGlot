// Extraction de textes depuis les fichiers RPG Maker XP/VX/VXAce (Ruby Marshal)
// Implémenté en T10 — marshal-rs

#[tauri::command]
pub fn extract_rpgm_classic(
    _project_id: i64,
    _game_path: String,
    _work_path: String,
) -> Result<usize, String> {
    Err("Non implémenté — voir T10".into())
}
