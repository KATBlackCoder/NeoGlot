// Commandes CRUD exposées au frontend via invoke()
// Projets, fichiers, strings, progression
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;
use crate::db::open;

// ─── Types sérialisables ──────────────────────────────────────────────────────

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

#[derive(Deserialize)]
pub struct ExtractedStringInput {
    pub source_hash: String,
    pub source_text: String,
    pub context_path: String,
    pub event_code: Option<i32>,
    pub row_index: i32,
    pub file_path: String,
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

#[derive(Serialize)]
pub struct ProjectProgress {
    pub done: i64,
    pub total: i64,
}

// ─── Commandes projets ────────────────────────────────────────────────────────

/// Liste tous les projets triés par date de mise à jour
#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, game_path, work_path, engine, source_lang, target_lang,
                    project_context, status, created_at
             FROM projects ORDER BY updated_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
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
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(projects)
}

/// Crée un nouveau projet et retourne son id
#[tauri::command]
pub fn create_project(p: NewProject, state: State<'_, AppState>) -> Result<i64, String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO projects (name, game_path, work_path, engine, source_lang, target_lang, project_context)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            p.name, p.game_path, p.work_path, p.engine,
            p.source_lang, p.target_lang, p.project_context
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

/// Supprime un projet et toutes ses données (CASCADE)
#[tauri::command]
pub fn delete_project(project_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM projects WHERE id = ?1", params![project_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Commandes strings ────────────────────────────────────────────────────────

/// Stocke les strings extraits d'un projet en base (INSERT OR IGNORE)
#[tauri::command]
pub fn store_strings(
    project_id: i64,
    strings: Vec<ExtractedStringInput>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;

    // Regrouper les strings par fichier source
    let mut files: std::collections::HashMap<String, Vec<&ExtractedStringInput>> =
        std::collections::HashMap::new();
    for s in &strings {
        files.entry(s.file_path.clone()).or_default().push(s);
    }

    let mut total = 0;
    for (file_path, file_strings) in &files {
        // Créer le fichier si absent
        conn.execute(
            "INSERT OR IGNORE INTO files (project_id, relative_path, strings_total)
             VALUES (?1, ?2, ?3)",
            params![project_id, file_path, file_strings.len() as i64],
        )
        .map_err(|e| e.to_string())?;

        let file_id: i64 = conn
            .query_row(
                "SELECT id FROM files WHERE project_id = ?1 AND relative_path = ?2",
                params![project_id, file_path],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;

        for s in file_strings {
            conn.execute(
                "INSERT OR IGNORE INTO strings
                 (file_id, source_hash, source_text, context_path, event_code, row_index)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    file_id, s.source_hash, s.source_text,
                    s.context_path, s.event_code, s.row_index
                ],
            )
            .map_err(|e| e.to_string())?;
            total += 1;
        }
    }
    Ok(total)
}

/// Retourne les strings d'un projet avec filtre optionnel sur le statut
#[tauri::command]
pub fn get_project_strings(
    project_id: i64,
    status_filter: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<StringEntry>, String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;
    // "%" = tous les statuts (LIKE wildcard)
    let filter = status_filter.unwrap_or_else(|| "%".into());

    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.source_hash, s.source_text, s.context_path, s.event_code,
                    s.row_index, s.translation, s.status, f.relative_path
             FROM strings s
             JOIN files f ON s.file_id = f.id
             WHERE f.project_id = ?1 AND s.status LIKE ?2
             ORDER BY f.relative_path, s.row_index",
        )
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map(params![project_id, filter], |row| {
            Ok(StringEntry {
                id: row.get(0)?,
                source_hash: row.get(1)?,
                source_text: row.get(2)?,
                context_path: row.get(3)?,
                event_code: row.get(4)?,
                row_index: row.get(5)?,
                translation: row.get(6)?,
                status: row.get(7)?,
                file_path: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

/// Retourne la progression globale d'un projet (strings done / total)
#[tauri::command]
pub fn get_project_progress(
    project_id: i64,
    state: State<'_, AppState>,
) -> Result<ProjectProgress, String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;
    let (done, total) = conn
        .query_row(
            "SELECT COALESCE(SUM(strings_done), 0), COALESCE(SUM(strings_total), 0)
             FROM files WHERE project_id = ?1",
            params![project_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .map_err(|e| e.to_string())?;

    Ok(ProjectProgress { done, total })
}
