// Déchiffrement des archives .rgss / .rgss2 / .rgss3 (RPG Maker XP/VX/VXAce)
// Implémenté en T10 — rpgmad-lib

#[tauri::command]
pub fn decrypt_rgss(_archive_path: String, _output_path: String) -> Result<(), String> {
    Err("Non implémenté — voir T10".into())
}
