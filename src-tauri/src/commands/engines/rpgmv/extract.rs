// Extraction de textes depuis les fichiers de jeu RPG Maker MV/MZ
// Utilise rvpacker-txt-rs-lib : Reader produit des .txt, on les parse et stocke en SQLite
// Applique RpgMakerTextValidator (filtrage) + RpgMakerFormatter (placeholders) avant insertion
use rusqlite::params;
use rvpacker_txt_rs_lib::{EngineType, FileFlags, Reader, NEW_LINE, SEPARATOR};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::Path;
use tauri::{AppHandle, Emitter, State};
use crate::AppState;
use crate::db::open;
use crate::commands::engines::formatter::EngineFormatter;
use super::formatter::RpgMakerFormatter;
use super::validation::RpgMakerTextValidator;

#[derive(Clone, Serialize)]
struct ExtractionProgress {
    current: usize,
    total: usize,
    file: String,
}

const NAME_COMMENT: &str = "<!-- NAME -->";

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Détermine le répertoire data selon le moteur (MV : www/data, MZ : data)
fn data_dir(game_path: &Path) -> std::path::PathBuf {
    let mv = game_path.join("www").join("data");
    if mv.exists() { mv } else { game_path.join("data") }
}

/// Calcule un hash SHA-256 pour une chaîne source
fn sha256(text: &str) -> String {
    let mut h = Sha256::new();
    h.update(text.as_bytes());
    format!("{:x}", h.finalize())
}

// ─── Commandes ────────────────────────────────────────────────────────────────

/// Extrait les textes d'un jeu RPG Maker MV/MZ, stocke en SQLite, retourne le compte
/// Émet des événements Tauri "extraction-progress" à chaque fichier traité
#[tauri::command]
pub fn extract_rpgmv(
    app: AppHandle,
    project_id: i64,
    game_path: String,
    work_path: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let game = Path::new(&game_path);
    let work = Path::new(&work_path);
    let data_src = data_dir(game);
    let translation_dir = work.join("translation");

    std::fs::create_dir_all(&translation_dir).map_err(|e| e.to_string())?;

    // Extraction via rvpacker-txt-rs-lib → produit des .txt dans translation_dir
    let mut reader = Reader::new();
    reader.set_files(FileFlags::Map | FileFlags::other() | FileFlags::System);
    reader
        .read(&data_src, &translation_dir, EngineType::New)
        .map_err(|e| e.to_string())?;

    // Compter les .txt pour la progression
    let txt_files: Vec<_> = std::fs::read_dir(&translation_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("txt"))
        .collect();
    let total_files = txt_files.len();

    let conn = open(&state.db_path).map_err(|e| e.to_string())?;

    // Nettoyer les strings et fichiers existants du projet avant ré-extraction
    conn.execute(
        "DELETE FROM strings WHERE file_id IN (SELECT id FROM files WHERE project_id = ?1)",
        params![project_id],
    ).map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM files WHERE project_id = ?1",
        params![project_id],
    ).map_err(|e| e.to_string())?;

    let mut total = 0usize;

    for (file_idx, entry) in txt_files.into_iter().enumerate() {
        let path = entry.path();

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Émettre la progression avant de traiter le fichier
        let _ = app.emit("extraction-progress", ExtractionProgress {
            current: file_idx + 1,
            total: total_files,
            file: file_name.clone(),
        });

        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

        // Extraction : raw_text → validation → formatage → (raw_text, formatted_text)
        let sources: Vec<(String, String)> = content
            .lines()
            .filter_map(|line| {
                let sep_pos = line.find(SEPARATOR)?;
                let left = &line[..sep_pos];
                if left.starts_with("<!-- ") {
                    return None;
                }
                let raw = left.replace(NEW_LINE, "\n");
                if raw.trim().is_empty() {
                    return None;
                }
                // Filtrage : skip textes non-traduisibles (techniques, ponctuation, etc.)
                if !RpgMakerTextValidator::validate_text(&raw) {
                    return None;
                }
                // Formatage : codes moteur → placeholders AI-friendly
                let formatted = RpgMakerFormatter::prepare_for_translation(&raw);
                Some((raw, formatted))
            })
            .collect();

        if sources.is_empty() {
            continue;
        }

        conn.execute(
            "INSERT OR IGNORE INTO files (project_id, relative_path, strings_total)
             VALUES (?1, ?2, ?3)",
            params![project_id, &file_name, sources.len() as i64],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE files SET strings_total = ?1
             WHERE project_id = ?2 AND relative_path = ?3",
            params![sources.len() as i64, project_id, &file_name],
        )
        .map_err(|e| e.to_string())?;

        let file_id: i64 = conn
            .query_row(
                "SELECT id FROM files WHERE project_id = ?1 AND relative_path = ?2",
                params![project_id, &file_name],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;

        for (row_idx, (raw_text, formatted_text)) in sources.iter().enumerate() {
            let hash = sha256(formatted_text);
            conn.execute(
                "INSERT OR IGNORE INTO strings
                 (file_id, source_hash, source_text, raw_text, context_path, row_index)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![file_id, hash, formatted_text, raw_text, &file_name, row_idx as i64],
            )
            .map_err(|e| e.to_string())?;
            total += 1;
        }
    }

    conn.execute(
        "UPDATE projects SET status = 'extracted' WHERE id = ?1",
        params![project_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(total)
}

/// Extrait les noms de personnages uniques depuis les fichiers .txt rvpacker (event 101 — NAME)
#[tauri::command]
pub fn extract_speakers(
    project_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;

    let work_path: String = conn
        .query_row(
            "SELECT work_path FROM projects WHERE id = ?1",
            params![project_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    let translation_dir = Path::new(&work_path).join("translation");
    if !translation_dir.exists() {
        return Err("Extraction non effectuée — lancez d'abord extract_rpgmv".into());
    }

    let mut speakers: HashSet<String> = HashSet::new();

    for entry in std::fs::read_dir(&translation_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("txt") {
            continue;
        }

        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let mut next_is_name = false;

        for line in content.lines() {
            if line.is_empty() {
                continue;
            }

            if let Some(sep_pos) = line.find(SEPARATOR) {
                let left = &line[..sep_pos];
                if left.starts_with("<!-- ") {
                    continue;
                }
                let name = left.replace(NEW_LINE, "\n");
                let name = name.trim();
                if next_is_name && !name.is_empty() {
                    speakers.insert(name.to_string());
                }
                next_is_name = false;
            } else if line == NAME_COMMENT {
                next_is_name = true;
            } else if line.starts_with("<!-- ") {
                // autre commentaire — ne pas modifier next_is_name
            } else {
                next_is_name = false;
            }
        }
    }

    let mut result: Vec<String> = speakers.into_iter().collect();
    result.sort();
    Ok(result)
}
