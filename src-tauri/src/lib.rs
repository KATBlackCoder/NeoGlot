// Point d'entrée de la bibliothèque Tauri
// Initialise DB, AppState, plugins et enregistre toutes les commandes
mod db;
mod commands;

use std::sync::Mutex;

/// État global partagé entre toutes les commandes Tauri via State<'_, AppState>
pub struct AppState {
    /// Chemin vers neoglot.db (calculé une fois au démarrage)
    pub db_path: std::path::PathBuf,
    /// Flag d'annulation de la traduction en cours (T06)
    pub translation_running: Mutex<bool>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialiser la DB avant de démarrer Tauri
    let db_path = db::get_db_path();
    db::init_schema(&db_path).expect("Erreur initialisation schéma SQLite");

    tauri::Builder::default()
        .manage(AppState {
            db_path,
            translation_running: Mutex::new(false),
        })
        // ─── Plugins ──────────────────────────────────────────────────────────
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("neoglot".into()),
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                ])
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        // ─── Commandes IPC ────────────────────────────────────────────────────
        .invoke_handler(tauri::generate_handler![
            // T03 — Santé Ollama (check_ollama disponible dès T03 pour HomeView)
            commands::translate::check_ollama,
            commands::translate::list_ollama_models,
            commands::translate::cancel_translation,
            commands::translate::start_translation,
            // T03/T05 — CRUD projets / fichiers / strings
            commands::db_commands::list_projects,
            commands::db_commands::create_project,
            commands::db_commands::delete_project,
            commands::db_commands::list_project_files,
            commands::db_commands::store_strings,
            commands::db_commands::get_project_strings,
            commands::db_commands::get_project_progress,
            // T04 — Détection moteur
            commands::detect::detect_engine,
            // T05 — Extraction RPG Maker MV/MZ
            commands::engines::rpgmv::extract::extract_rpgmv,
            commands::engines::rpgmv::extract::extract_speakers,
            // T07 — Réinjection RPG Maker MV/MZ
            commands::engines::rpgmv::inject::write_rpgmv,
            // T10 — Extraction + déchiffrement RPG Maker XP/VX/VXAce
            commands::engines::rpgm_classic::extract::extract_rpgm_classic,
            commands::engines::rpgm_classic::decrypt::decrypt_rgss,
            // T08 — Glossaire
            commands::glossary::list_glossary,
            commands::glossary::add_glossary_term,
            commands::glossary::update_glossary_term,
            commands::glossary::delete_glossary_term,
            commands::glossary::import_speakers,
            // T09 — Wolf RPG Editor
            commands::engines::wolf::extract::extract_wolf,
            commands::engines::wolf::inject::inject_wolf,
        ])
        .run(tauri::generate_context!())
        .expect("Erreur au démarrage de NeoGlot");
}
