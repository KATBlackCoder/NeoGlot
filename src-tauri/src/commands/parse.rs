// Extraction de textes depuis les fichiers de jeu RPG Maker
// Implémenté en T05 — rvpacker-txt-rs-lib + sha2

#[tauri::command]
pub fn extract_rpgmv(_project_id: i64, _game_path: String, _work_path: String) -> Result<usize, String> {
    Err("Non implémenté — voir T05".into())
}

#[tauri::command]
pub fn extract_speakers(_project_id: i64) -> Result<Vec<String>, String> {
    Err("Non implémenté — voir T05".into())
}

#[tauri::command]
pub fn extract_rpgm_classic(_project_id: i64, _game_path: String, _work_path: String) -> Result<usize, String> {
    Err("Non implémenté — voir T10".into())
}
