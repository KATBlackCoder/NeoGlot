// Gestion du chemin et du schéma SQLite (rusqlite, mode WAL)
use std::path::{Path, PathBuf};
use rusqlite::{Connection, Result};

/// Retourne le chemin du fichier neoglot.db selon l'OS
/// Linux  : ~/.local/share/neoglot/neoglot.db
/// Windows: %APPDATA%\neoglot\neoglot.db
pub fn get_db_path() -> PathBuf {
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()));

    let dir = base.join("neoglot");
    std::fs::create_dir_all(&dir).ok();
    dir.join("neoglot.db")
}

/// Ouvre une connexion rusqlite avec WAL et clés étrangères activées
pub fn open(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    Ok(conn)
}

/// Crée les 6 tables du schéma NeoGlot si elles n'existent pas
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

        CREATE INDEX IF NOT EXISTS idx_strings_hash   ON strings(source_hash);
        CREATE INDEX IF NOT EXISTS idx_strings_status ON strings(status);
        CREATE INDEX IF NOT EXISTS idx_strings_file   ON strings(file_id);

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
            note            TEXT NOT NULL DEFAULT '',
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
