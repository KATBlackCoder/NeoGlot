// Réinjection des traductions dans les fichiers de jeu RPG Maker
// Implémenté en T07 — rvpacker-txt-rs-lib

#[tauri::command]
pub fn write_rpgmv(_project_id: i64) -> Result<(), String> {
    Err("Non implémenté — voir T07".into())
}
