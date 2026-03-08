# T06 — Moteur de Traduction (reqwest + Tauri Channel progression)

**Statut** : TODO
**Dépendances** : T05 (strings extraits en SQLite), T03 (AppState + DB prêt)

> Pas de Python, pas de SSE HTTP, pas de traduction temps réel. Tout passe par `invoke()`. Le **Tauri Channel** sert uniquement à reporter la progression (1 event par texte traduit), pas à streamer des tokens.

---

## Objectif

Implémenter le pipeline de traduction batch en Rust : déduplication, protection placeholders, appels Ollama via `reqwest` (POST /api/generate, réponse complète), progression via Tauri Channel, stockage SQLite.

---

## Étapes

### 1. Créer src-tauri/src/commands/translate.rs

```rust
use reqwest::blocking::Client;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use tauri::{ipc::Channel, State};
use crate::{AppState, db::open};

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
pub struct TranslationProgress {
    pub done: usize,
    pub total: usize,
    pub last_translation: String,
    pub status: String,  // "running" | "done" | "cancelled" | "error"
}

#[derive(Serialize, Deserialize)]
struct GlossaryTerm {
    term: String,
    translation: String,
    note: String,
}

// ─── Commandes publiques ───────────────────────────────────────────────────────

/// Vérifier si Ollama est disponible (GET /api/tags)
#[tauri::command]
pub fn check_ollama() -> bool {
    Client::new()
        .get("http://localhost:11434/api/tags")
        .send()
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Lister les modèles Ollama installés
#[tauri::command]
pub fn list_ollama_models() -> Result<Vec<String>, String> {
    #[derive(Deserialize)]
    struct TagsResponse { models: Vec<OllamaModel> }
    #[derive(Deserialize)]
    struct OllamaModel { name: String }

    let resp: TagsResponse = Client::new()
        .get("http://localhost:11434/api/tags")
        .send().map_err(|e| e.to_string())?
        .json().map_err(|e| e.to_string())?;
    Ok(resp.models.into_iter().map(|m| m.name).collect())
}

/// Démarrer la traduction d'un projet (avec streaming via Channel)
#[tauri::command]
pub async fn start_translation(
    project_id: i64,
    model: String,
    on_progress: Channel<TranslationProgress>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Marquer comme en cours
    *state.translation_running.lock().unwrap() = true;

    let result = run_translation(project_id, &model, on_progress, &state).await;

    *state.translation_running.lock().unwrap() = false;
    result
}

/// Annuler la traduction en cours
#[tauri::command]
pub fn cancel_translation(state: State<'_, AppState>) {
    *state.translation_running.lock().unwrap() = false;
}

// ─── Pipeline principal ────────────────────────────────────────────────────────

async fn run_translation(
    project_id: i64,
    model: &str,
    on_progress: Channel<TranslationProgress>,
    state: &AppState,
) -> Result<(), String> {
    let ollama = Ollama::default();

    // 1. Récupérer les strings pending
    let pending = get_pending_strings(project_id, state)?;
    let total_all = pending.len();

    // 2. Déduplication par source_hash
    let mut unique: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for (hash, text) in &pending {
        unique.entry(hash.clone()).or_insert_with(|| text.clone());
    }

    // 3. Vérifier le cache de traduction
    let (cached, to_translate) = check_cache(model, &unique, state)?;

    // Appliquer le cache immédiatement
    let mut done = 0;
    for (hash, translation) in &cached {
        apply_translation_by_hash(project_id, hash, translation, state)?;
        done += 1;
        on_progress.send(TranslationProgress {
            done,
            total: total_all,
            last_translation: translation.clone(),
            status: "running".into(),
        }).ok();
    }

    // 4. Récupérer le glossaire
    let glossary = get_glossary(project_id, state)?;

    // 5. Grouper en batches par tokens
    let texts: Vec<String> = to_translate.values().cloned().collect();
    let batches = batch_by_tokens(&texts, 2048);

    // 6. Traduire chaque batch
    for batch in &batches {
        if !*state.translation_running.lock().unwrap() {
            // Annulé par l'utilisateur
            on_progress.send(TranslationProgress {
                done, total: total_all,
                last_translation: String::new(),
                status: "cancelled".into(),
            }).ok();
            return Ok(());
        }

        let translations = translate_batch(batch, model, &glossary, &ollama).await?;

        for (text, translation) in batch.iter().zip(translations.iter()) {
            let hash = sha256_hex(text);
            apply_translation_by_hash(project_id, &hash, translation, state)?;
            save_to_cache(&hash, text, translation, model, state)?;
            done += 1;
            on_progress.send(TranslationProgress {
                done,
                total: total_all,
                last_translation: translation.clone(),
                status: "running".into(),
            }).ok();
        }
    }

    on_progress.send(TranslationProgress {
        done,
        total: total_all,
        last_translation: String::new(),
        status: "done".into(),
    }).ok();

    Ok(())
}

// ─── Traduction d'un batch ─────────────────────────────────────────────────────

async fn translate_batch(
    texts: &[String],
    model: &str,
    glossary: &[GlossaryTerm],
    ollama: &Ollama,
) -> Result<Vec<String>, String> {
    // Protéger les placeholders
    let (protected, mappings): (Vec<_>, Vec<_>) = texts.iter()
        .map(|t| protect_placeholders(t))
        .unzip();

    // Construire le prompt
    let prompt = build_prompt(&protected, model, glossary);

    // Appel Ollama
    let options = ModelOptions::default().temperature(0.3);
    let response = ollama
        .generate(GenerationRequest::new(model.to_string(), prompt).options(options))
        .await
        .map_err(|e| format!("Erreur Ollama: {e}"))?;

    // Parser la réponse JSON
    let raw_translations = parse_llm_response(&response.response, texts.len())?;

    // Restaurer les placeholders
    Ok(raw_translations.into_iter().zip(mappings.iter())
        .map(|(t, m)| restore_placeholders(&t, m))
        .collect())
}

// ─── Prompt LLM ───────────────────────────────────────────────────────────────

fn build_prompt(texts: &[String], _model: &str, glossary: &[GlossaryTerm]) -> String {
    let glossary_str = if glossary.is_empty() {
        "None.".to_string()
    } else {
        glossary.iter().map(|g| {
            if g.note.is_empty() {
                format!("- \"{}\" → \"{}\"", g.term, g.translation)
            } else {
                format!("- \"{}\" → \"{}\" ({})", g.term, g.translation, g.note)
            }
        }).collect::<Vec<_>>().join("\n")
    };

    let texts_json = serde_json::to_string_pretty(
        &texts.iter().enumerate()
            .map(|(i, t)| (i.to_string(), t))
            .collect::<serde_json::Map<_, _>>()
    ).unwrap_or_default();

    format!(r#"You are a professional translator for Japanese RPG games.
Source language: Japanese
Target language: French

Glossary (use these translations exactly):
{glossary_str}

Translate the following strings. Return a JSON object with the same numeric keys.
Preserve all formatting codes like <NAME_1>, <SE_5>, <COLOR_3> exactly as-is.
Keep translations natural and suitable for an RPG game.

Input:
{texts_json}

Output (JSON only, no markdown, no explanation):"#)
}

// ─── Protection des placeholders ──────────────────────────────────────────────

type PlaceholderMap = Vec<(String, String)>;

fn protect_placeholders(text: &str) -> (String, PlaceholderMap) {
    use regex::Regex;
    // Patterns RPG Maker : \SE[x], \BGM[x], \n[x], \V[x], \C[x], \I[x], \{, \}
    let patterns = [
        (r"\\SE\[(\d+)\]", "SE"),
        (r"\\BGM\[(\d+)\]", "BGM"),
        (r"\\n\[(\d+)\]", "NAME"),
        (r"\\V\[(\d+)\]", "VAR"),
        (r"\\C\[(\d+)\]", "COLOR"),
        (r"\\I\[(\d+)\]", "ICON"),
        (r"\\\{", "SIZU"),
        (r"\\\}", "SIZD"),
    ];
    let mut result = text.to_string();
    let mut mapping: PlaceholderMap = Vec::new();

    for (pattern, prefix) in &patterns {
        let re = Regex::new(pattern).unwrap();
        for cap in re.captures_iter(text) {
            let original = cap[0].to_string();
            let id = cap.get(1).map(|m| m.as_str()).unwrap_or("0");
            let token = format!("<{}_{}>", prefix, id);
            if !mapping.iter().any(|(t, _)| t == &token) {
                mapping.push((token.clone(), original.clone()));
            }
            result = result.replace(&original, &token);
        }
    }
    (result, mapping)
}

fn restore_placeholders(text: &str, mapping: &PlaceholderMap) -> String {
    let mut result = text.to_string();
    for (token, original) in mapping {
        result = result.replace(token.as_str(), original.as_str());
    }
    result
}

// ─── Parsing réponse LLM ──────────────────────────────────────────────────────

fn parse_llm_response(response: &str, expected: usize) -> Result<Vec<String>, String> {
    // Extraire le JSON de la réponse (peut contenir du texte autour)
    let json_start = response.find('{').unwrap_or(0);
    let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
    let json_str = &response[json_start..json_end];

    let map: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(json_str).map_err(|e| format!("Parse erreur: {e}"))?;

    let mut result = Vec::with_capacity(expected);
    for i in 0..expected {
        let translation = map.get(&i.to_string())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        result.push(translation);
    }
    Ok(result)
}

// ─── Helpers DB ───────────────────────────────────────────────────────────────

fn get_pending_strings(project_id: i64, state: &AppState) -> Result<Vec<(String, String)>, String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT s.source_hash, s.source_text FROM strings s
         JOIN files f ON s.file_id = f.id
         WHERE f.project_id = ?1 AND s.status = 'pending'"
    ).map_err(|e| e.to_string())?;

    stmt.query_map(params![project_id], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn check_cache(
    model: &str,
    unique: &std::collections::HashMap<String, String>,
    state: &AppState,
) -> Result<(Vec<(String, String)>, std::collections::HashMap<String, String>), String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;
    let mut cached = Vec::new();
    let mut to_translate = unique.clone();

    for (hash, _) in unique {
        let result: rusqlite::Result<String> = conn.query_row(
            "SELECT translation FROM translation_cache WHERE source_hash = ?1 AND model = ?2",
            params![hash, model],
            |r| r.get(0),
        );
        if let Ok(translation) = result {
            cached.push((hash.clone(), translation));
            to_translate.remove(hash);
        }
    }
    Ok((cached, to_translate))
}

fn apply_translation_by_hash(
    project_id: i64,
    hash: &str,
    translation: &str,
    state: &AppState,
) -> Result<(), String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE strings SET translation = ?1, status = 'translated'
         WHERE source_hash = ?2
         AND file_id IN (SELECT id FROM files WHERE project_id = ?3)",
        params![translation, hash, project_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

fn save_to_cache(hash: &str, text: &str, translation: &str, model: &str, state: &AppState) -> Result<(), String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO translation_cache (source_hash, model, source_text, translation)
         VALUES (?1, ?2, ?3, ?4)",
        params![hash, model, text, translation],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

fn get_glossary(project_id: i64, state: &AppState) -> Result<Vec<GlossaryTerm>, String> {
    let conn = open(&state.db_path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT term, translation, note FROM glossary
         WHERE project_id = ?1 OR project_id IS NULL ORDER BY term"
    ).map_err(|e| e.to_string())?;

    stmt.query_map(params![project_id], |r| Ok(GlossaryTerm {
        term: r.get(0)?,
        translation: r.get(1)?,
        note: r.get(2)?,
    })).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())
}

// ─── Utilitaires ──────────────────────────────────────────────────────────────

fn sha256_hex(text: &str) -> String {
    let mut h = Sha256::new();
    h.update(text.as_bytes());
    format!("{:x}", h.finalize())
}

fn batch_by_tokens(texts: &[String], max_tokens: usize) -> Vec<Vec<String>> {
    // Approximation : 4 caractères ≈ 1 token
    let mut batches: Vec<Vec<String>> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    let mut tokens = 0;

    for text in texts {
        let t = (text.len() / 4).max(1);
        if tokens + t > max_tokens && !current.is_empty() {
            batches.push(std::mem::take(&mut current));
            tokens = 0;
        }
        current.push(text.clone());
        tokens += t;
    }
    if !current.is_empty() { batches.push(current); }
    batches
}
```

### 2. Frontend — src/composables/useTranslation.ts

Le composable délègue l'état (`isRunning`, `progress`) au store Pinia `useTranslationStore`, accessible globalement sans prop-drilling.

```typescript
import { invoke, Channel } from '@tauri-apps/api/core';
import { sendNotification } from '@tauri-apps/plugin-notification';
import { useTranslationStore, type TranslationProgress } from '@/stores';
import { useQueryClient } from '@tanstack/vue-query';

export function useTranslation(projectId: number) {
  const translationStore = useTranslationStore();
  const qc = useQueryClient();

  const start = async (model: string) => {
    translationStore.start();

    // Tauri Channel — progression Rust → Vue (1 event par texte traduit)
    const channel = new Channel<TranslationProgress>();
    channel.onmessage = (event) => {
      translationStore.updateProgress(event);

      if (event.status === 'done') {
        sendNotification({
          title: 'NeoGlot — Traduction terminée',
          body: `${event.total} textes traduits`,
        });
        // Invalider le cache Vue Query pour rafraîchir la liste et la progression
        qc.invalidateQueries({ queryKey: ['project-progress', projectId] });
        qc.invalidateQueries({ queryKey: ['project-strings', projectId] });
      }
    };

    try {
      await invoke('start_translation', { projectId, model, onProgress: channel });
    } catch {
      translationStore.stop();
    }
  };

  const cancel = async () => {
    await invoke('cancel_translation');
    translationStore.stop();
  };

  return { start, cancel };
}
```

### 3. Frontend — src/composables/useOllama.ts

(Défini dans T02 — rappel)

```typescript
import { invoke } from '@tauri-apps/api/core';
import { useQuery } from '@tanstack/vue-query';

export function useOllamaStatus() {
  return useQuery({
    queryKey: ['ollama-status'],
    queryFn: () => invoke<boolean>('check_ollama'),
    refetchInterval: 30_000,
    retry: false,
  });
}

export function useOllamaModels() {
  return useQuery({
    queryKey: ['ollama-models'],
    queryFn: () => invoke<string[]>('list_ollama_models'),
    retry: false,
  });
}
```

---

## Validation

- `invoke('check_ollama')` → `true` si Ollama tourne
- `invoke('list_ollama_models')` → liste `['llama3:8b', ...]`
- Démarrer la traduction sur un projet extrait → Channel émet les events en temps réel
- Les strings se mettent à jour en SQLite (`status = 'translated'`)
- Le cache fonctionne : re-traduire → 0 appels Ollama supplémentaires
- Annuler mid-traduction → s'arrête proprement au prochain batch
- Les placeholders `\n[1]`, `\SE[5]` sont préservés dans les traductions
