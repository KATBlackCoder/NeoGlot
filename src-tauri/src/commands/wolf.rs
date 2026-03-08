// Module Wolf RPG — extraction et réinjection via WolfTL + UberWolf (subprocess)
// Binaires : src-tauri/resources/WolfTL.exe + UberWolfCli.exe (bundlés via tauri.conf.json "resources")
// Implémenté en T09

#[tauri::command]
pub fn extract_wolf(_project_id: i64) -> Result<usize, String> {
    Err("Non implémenté — voir T09".into())
}

#[tauri::command]
pub fn inject_wolf(_project_id: i64) -> Result<(), String> {
    Err("Non implémenté — voir T09".into())
}
