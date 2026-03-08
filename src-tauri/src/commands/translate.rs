// Pipeline de traduction batch Ollama via reqwest (pas de streaming token)
// check_ollama + list_ollama_models implémentés ici
// start_translation / cancel_translation implémentés en T06
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;

// ─── Types de progression (Channel T06) ──────────────────────────────────────

#[derive(Serialize, Clone)]
pub struct TranslationProgress {
    pub done: usize,
    pub total: usize,
    pub last_translation: String,
    pub status: String, // "running" | "done" | "cancelled" | "error"
}

// ─── Santé Ollama ─────────────────────────────────────────────────────────────

/// Vérifie si Ollama est accessible sur localhost:11434
#[tauri::command]
pub fn check_ollama() -> bool {
    Client::new()
        .get("http://localhost:11434/api/tags")
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Liste les modèles disponibles dans Ollama
#[tauri::command]
pub fn list_ollama_models() -> Result<Vec<String>, String> {
    #[derive(Deserialize)]
    struct TagsResponse {
        models: Vec<OllamaModel>,
    }
    #[derive(Deserialize)]
    struct OllamaModel {
        name: String,
    }

    let resp: TagsResponse = Client::new()
        .get("http://localhost:11434/api/tags")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .map_err(|e| format!("Erreur connexion Ollama: {e}"))?
        .json()
        .map_err(|e| format!("Erreur parsing réponse Ollama: {e}"))?;

    Ok(resp.models.into_iter().map(|m| m.name).collect())
}

// ─── Traduction (T06) ─────────────────────────────────────────────────────────

/// Démarre la traduction d'un projet — implémenté en T06
#[tauri::command]
pub async fn start_translation(
    _project_id: i64,
    _model: String,
    _on_progress: tauri::ipc::Channel<TranslationProgress>,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    Err("Non implémenté — voir T06".into())
}

/// Annule la traduction en cours — implémenté en T06
#[tauri::command]
pub fn cancel_translation(state: State<'_, AppState>) {
    *state.translation_running.lock().unwrap() = false;
}
