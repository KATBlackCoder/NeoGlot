# T03 — Scaffold Rust Commands (DB + State)

**Statut** : DONE
**Dépendances** : T01 (Cargo.toml prêt)

> Python supprimé. Toute la logique métier est en Rust. Le frontend parle uniquement via `invoke()`.

---

## Objectif

Mettre en place la structure des commandes Rust : état global (`AppState`), accès SQLite via `rusqlite`, initialisation du schéma, et helper DB partagé entre toutes les commandes.

---

## Structure à créer

```
src-tauri/src/
├── main.rs
├── lib.rs              # setup Tauri : plugins, AppState, commandes
├── db.rs               # get_db_path(), init_schema(), helpers rusqlite
└── commands/
    ├── mod.rs
    ├── detect.rs       # detect_engine
    ├── db_commands.rs  # CRUD projets, fichiers, strings (exposé via invoke)
    ├── parse.rs        # extraction rvpacker-txt-rs-lib
    ├── write.rs        # réinjection rvpacker-txt-rs-lib
    ├── decrypt.rs      # rpgmad-lib
    ├── translate.rs    # ollama-rs + Tauri Channel streaming
    ├── wolf.rs         # WolfTL/UberWolf subprocess
    └── glossary.rs     # glossaire CRUD
```

---

## Étapes

### 1. Mettre à jour Cargo.toml

```toml
[dependencies]
tauri = { version = "2", features = [] }

# Plugins Tauri
tauri-plugin-shell = "2"
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
tauri-plugin-store = "2"
tauri-plugin-log = "2"
tauri-plugin-window-state = "2"
tauri-plugin-notification = "2"
tauri-plugin-process = "2"
tauri-plugin-clipboard-manager = "2"
# Pas de tauri-plugin-sql (SQLx) — DB uniquement via rusqlite côté Rust
# Pas de tauri-plugin-http — appels Ollama via ollama-rs

# Logique métier
ollama-rs = { version = "0.2", features = ["stream"] }
rusqlite = { version = "0.32", features = ["bundled"] }
tokio = { version = "1", features = ["full"] }

# Parsing RPG Maker
rvpacker-txt-rs-lib = "11.1.2"
marshal-rs = "*"
rpgmad-lib = "4.0.0"

# Utilitaires
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
walkdir = "2"
log = "^0.4"
regex = "1"
```

### 2. Créer src-tauri/src/db.rs

```rust
// Gestion du chemin et du schéma SQLite (rusqlite)
use std::path::{Path, PathBuf};
use rusqlite::{Connection, Result};

/// Retourne le chemin du fichier neoglot.db selon l'OS
pub fn get_db_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    let base = PathBuf::from(std::env::var("APPDATA").unwrap_or_default());
    #[cfg(not(target_os = "windows"))]
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()))
        .join(".local/share");

    let dir = base.join("neoglot");
    std::fs::create_dir_all(&dir).ok();
    dir.join("neoglot.db")
}

/// Ouvre une connexion rusqlite
pub fn open(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    Ok(conn)
}

/// Initialise le schéma (CREATE TABLE IF NOT EXISTS)
pub fn init_schema(db_path: &Path) -> Result<()> {
    let conn = open(db_path)?;
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS projects (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            name            TEXT NOT NULL,
            game_path       TEXT NOT NULL,
            work_path       TEXT NOT NULL,
            engine          TEXT NOT NULL,
            source_lang     TEXT NOT NULL DEFAULT 'ja',
            target_lang     TEXT NOT NULL DEFAULT 'fr',
            project_context TEXT NOT NULL DEFAULT '',
            status          TEXT NOT NULL DEFAULT 'created',
            created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS files (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id      INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            relative_path   TEXT NOT NULL,
            status          TEXT NOT NULL DEFAULT 'pending',
            strings_total   INTEGER DEFAULT 0,
            strings_done    INTEGER DEFAULT 0,
            UNIQUE(project_id, relative_path)
        );
        CREATE TABLE IF NOT EXISTS strings (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id         INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
            source_hash     TEXT NOT NULL,
            source_text     TEXT NOT NULL,
            context_path    TEXT NOT NULL DEFAULT '',
            event_code      INTEGER,
            row_index       INTEGER DEFAULT 0,
            translation     TEXT,
            status          TEXT NOT NULL DEFAULT 'pending'
        );
        CREATE INDEX IF NOT EXISTS idx_strings_hash ON strings(source_hash);
        CREATE INDEX IF NOT EXISTS idx_strings_status ON strings(status);
        CREATE TABLE IF NOT EXISTS translation_cache (
            source_hash     TEXT NOT NULL,
            model           TEXT NOT NULL,
            source_lang     TEXT NOT NULL DEFAULT 'ja',
            target_lang     TEXT NOT NULL DEFAULT 'fr',
            source_text     TEXT NOT NULL,
            translation     TEXT NOT NULL,
            created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (source_hash, model, source_lang, target_lang)
        );
        CREATE TABLE IF NOT EXISTS glossary (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id      INTEGER REFERENCES projects(id) ON DELETE CASCADE,
            term            TEXT NOT NULL,
            translation     TEXT NOT NULL,
            note            TEXT DEFAULT '',
            match_mode      TEXT NOT NULL DEFAULT 'exact'
        );
        CREATE TABLE IF NOT EXISTS translation_jobs (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id      INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            model           TEXT NOT NULL,
            status          TEXT NOT NULL DEFAULT 'queued',
            strings_total   INTEGER DEFAULT 0,
            strings_done    INTEGER DEFAULT 0,
            error_message   TEXT,
            created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
        );
    ")?;
    Ok(())
}
```

### 3. Créer src-tauri/src/lib.rs

```rust
mod db;
mod commands;

use std::sync::Mutex;

/// État global partagé entre toutes les commandes Tauri
pub struct AppState {
    pub db_path: std::path::PathBuf,
    /// Flag d'annulation de traduction en cours
    pub translation_running: Mutex<bool>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = db::get_db_path();
    db::init_schema(&db_path).expect("Erreur initialisation DB");

    tauri::Builder::default()
        .manage(AppState {
            db_path: db_path.clone(),
            translation_running: Mutex::new(false),
        })
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
        .invoke_handler(tauri::generate_handler![
            // Détection
            commands::detect::detect_engine,
            // DB projets
            commands::db_commands::list_projects,
            commands::db_commands::create_project,
            commands::db_commands::delete_project,
            commands::db_commands::store_strings,
            commands::db_commands::get_project_strings,
            commands::db_commands::get_project_progress,
            // Parsing RPG Maker
            commands::parse::extract_rpgmv,
            commands::parse::extract_speakers,
            commands::parse::extract_rpgm_classic,
            // Réinjection
            commands::write::write_rpgmv,
            // Déchiffrement
            commands::decrypt::decrypt_rgss,
            // Traduction (Tauri Channel)
            commands::translate::start_translation,
            commands::translate::cancel_translation,
            commands::translate::check_ollama,
            commands::translate::list_ollama_models,
            // Wolf RPG
            commands::wolf::extract_wolf,
            commands::wolf::inject_wolf,
            // Glossaire
            commands::glossary::list_glossary,
            commands::glossary::add_glossary_term,
            commands::glossary::update_glossary_term,
            commands::glossary::delete_glossary_term,
            commands::glossary::import_speakers,
        ])
        .run(tauri::generate_context!())
        .expect("Erreur démarrage NeoGlot");
}
```

### 4. Créer src-tauri/src/commands/db_commands.rs

```rust
// Commandes CRUD exposées au frontend via invoke()
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;
use crate::db::open;

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub game_path: String,
    pub work_path: String,
    pub engine: String,
    pub source_lang: String,
    pub target_lang: String,
    pub project_context: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct NewProject {
    pub name: String,
    pub game_path: String,
    pub work_path: String,
    pub engine: String,
    pub source_lang: String,
    pub target_lang: String,
    pub project_context: String,
}

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, name, game_path, work_path, engine, source_lang, target_lang,
                project_context, status, created_at
         FROM projects ORDER BY updated_at DESC"
    ).map_err(|e| e.to_string())?;

    let projects = stmt.query_map([], |row| Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        game_path: row.get(2)?,
        work_path: row.get(3)?,
        engine: row.get(4)?,
        source_lang: row.get(5)?,
        target_lang: row.get(6)?,
        project_context: row.get(7)?,
        status: row.get(8)?,
        created_at: row.get(9)?,
    })).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(projects)
}

#[tauri::command]
pub fn create_project(p: NewProject, state: State<'_, AppState>) -> Result<i64, String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO projects (name, game_path, work_path, engine, source_lang, target_lang, project_context)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![p.name, p.game_path, p.work_path, p.engine, p.source_lang, p.target_lang, p.project_context],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn delete_project(project_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM projects WHERE id = ?1", params![project_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Deserialize)]
pub struct ExtractedStringInput {
    pub source_hash: String,
    pub source_text: String,
    pub context_path: String,
    pub event_code: Option<i32>,
    pub row_index: i32,
    pub file_path: String,
}

#[tauri::command]
pub fn store_strings(
    project_id: i64,
    strings: Vec<ExtractedStringInput>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;

    // Grouper par fichier
    let mut files: std::collections::HashMap<String, Vec<&ExtractedStringInput>> = std::collections::HashMap::new();
    for s in &strings {
        files.entry(s.file_path.clone()).or_default().push(s);
    }

    let mut total = 0;
    for (file_path, file_strings) in &files {
        conn.execute(
            "INSERT OR IGNORE INTO files (project_id, relative_path, strings_total) VALUES (?1, ?2, ?3)",
            params![project_id, file_path, file_strings.len()],
        ).map_err(|e| e.to_string())?;

        let file_id: i64 = conn.query_row(
            "SELECT id FROM files WHERE project_id = ?1 AND relative_path = ?2",
            params![project_id, file_path],
            |r| r.get(0),
        ).map_err(|e| e.to_string())?;

        for s in file_strings {
            conn.execute(
                "INSERT OR IGNORE INTO strings (file_id, source_hash, source_text, context_path, event_code, row_index)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![file_id, s.source_hash, s.source_text, s.context_path, s.event_code, s.row_index],
            ).map_err(|e| e.to_string())?;
            total += 1;
        }
    }
    Ok(total)
}

#[derive(Serialize)]
pub struct StringEntry {
    pub id: i64,
    pub source_hash: String,
    pub source_text: String,
    pub context_path: String,
    pub event_code: Option<i32>,
    pub row_index: i32,
    pub translation: Option<String>,
    pub status: String,
    pub file_path: String,
}

#[tauri::command]
pub fn get_project_strings(
    project_id: i64,
    status_filter: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<StringEntry>, String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;
    let filter = status_filter.unwrap_or_else(|| "%".into());
    let mut stmt = conn.prepare(
        "SELECT s.id, s.source_hash, s.source_text, s.context_path, s.event_code,
                s.row_index, s.translation, s.status, f.relative_path
         FROM strings s JOIN files f ON s.file_id = f.id
         WHERE f.project_id = ?1 AND s.status LIKE ?2
         ORDER BY f.relative_path, s.row_index"
    ).map_err(|e| e.to_string())?;

    let entries = stmt.query_map(params![project_id, filter], |row| Ok(StringEntry {
        id: row.get(0)?,
        source_hash: row.get(1)?,
        source_text: row.get(2)?,
        context_path: row.get(3)?,
        event_code: row.get(4)?,
        row_index: row.get(5)?,
        translation: row.get(6)?,
        status: row.get(7)?,
        file_path: row.get(8)?,
    })).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(entries)
}
```

/// Progression d'un projet (utilisé par la liste projets dans l'UI)
#[derive(Serialize)]
pub struct ProjectProgress {
    pub done: i64,
    pub total: i64,
}

#[tauri::command]
pub fn get_project_progress(
    project_id: i64,
    state: State<'_, AppState>,
) -> Result<ProjectProgress, String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;
    let (done, total) = conn.query_row(
        "SELECT COALESCE(SUM(strings_done),0), COALESCE(SUM(strings_total),0)
         FROM files WHERE project_id = ?1",
        params![project_id],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
    ).map_err(|e| e.to_string())?;
    Ok(ProjectProgress { done, total })
}
```

Ajouter dans `lib.rs` → `generate_handler![]` :
```rust
commands::db_commands::get_project_progress,
```

---

## Validation

- `pnpm tauri dev` compile sans erreur avec les nouvelles crates
- `invoke('list_projects')` retourne `[]` au premier lancement
- `invoke('create_project', {...})` crée un projet en DB
- `invoke('list_projects')` retourne le projet créé
- `invoke('get_project_progress', { projectId: 1 })` retourne `{ done: 0, total: 0 }`
- Schéma SQLite vérifié : `sqlite3 ~/.local/share/neoglot/neoglot.db .schema`
