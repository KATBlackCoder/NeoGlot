// Réinjection des traductions dans les fichiers RPG Maker MV/MZ
// Implémenté en T07 — rvpacker-txt-rs-lib Writer

#[tauri::command]
pub fn write_rpgmv(_project_id: i64) -> Result<(), String> {
    Err("Non implémenté — voir T07".into())
}
