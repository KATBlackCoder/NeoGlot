# T08 — Module Glossaire

**Statut** : TODO
**Dépendances** : T04 (projets), T06 (traduction — le glossaire est utilisé dans les prompts)

---

## Objectif

Implémenter le glossaire : termes imposés (noms propres, termes de jeu) injectés dans les prompts LLM et vérifiés en QC post-traduction. Tout en Rust via `rusqlite` + `invoke()`.

---

## Étapes

### 1. Commandes Rust — src-tauri/src/commands/glossary.rs

```rust
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct GlossaryEntry {
    pub id: i64,
    pub project_id: Option<i64>, // NULL = glossaire global
    pub term: String,
    pub translation: String,
    pub note: String,
    pub match_mode: String, // "exact" | "contains" | "regex"
}

#[tauri::command]
pub fn list_glossary(
    project_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<GlossaryEntry>, String> {
    let conn = Connection::open(&state.db_path).map_err(|e| e.to_string())?;
    // Retourner le glossaire du projet + le glossaire global (project_id IS NULL)
    let mut stmt = conn.prepare(
        "SELECT id, project_id, term, translation, note, match_mode
         FROM glossary
         WHERE project_id = ?1 OR project_id IS NULL
         ORDER BY term"
    ).map_err(|e| e.to_string())?;
    let entries = stmt.query_map(params![project_id], |row| {
        Ok(GlossaryEntry {
            id: row.get(0)?,
            project_id: row.get(1)?,
            term: row.get(2)?,
            translation: row.get(3)?,
            note: row.get(4)?,
            match_mode: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(entries)
}

#[tauri::command]
pub fn add_glossary_term(
    project_id: Option<i64>,
    term: String,
    translation: String,
    note: String,
    match_mode: String,
    state: State<'_, AppState>,
) -> Result<GlossaryEntry, String> {
    let conn = Connection::open(&state.db_path).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO glossary (project_id, term, translation, note, match_mode)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![project_id, term, translation, note, match_mode],
    ).map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    Ok(GlossaryEntry { id, project_id, term, translation, note, match_mode })
}

#[tauri::command]
pub fn update_glossary_term(
    term_id: i64,
    term: String,
    translation: String,
    note: String,
    match_mode: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = Connection::open(&state.db_path).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE glossary SET term=?1, translation=?2, note=?3, match_mode=?4 WHERE id=?5",
        params![term, translation, note, match_mode, term_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_glossary_term(
    term_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = Connection::open(&state.db_path).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM glossary WHERE id = ?1", params![term_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Import automatique des speakers (noms de personnages) depuis les strings extraits
#[tauri::command]
pub fn import_speakers_to_glossary(
    project_id: i64,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let conn = Connection::open(&state.db_path).map_err(|e| e.to_string())?;

    // Récupérer les speakers uniques (event_code = 101 = header dialogue RPG Maker)
    let mut stmt = conn.prepare(
        "SELECT DISTINCT s.source_text FROM strings s
         JOIN files f ON s.file_id = f.id
         WHERE f.project_id = ?1 AND s.event_code = 101 AND s.source_text != ''"
    ).map_err(|e| e.to_string())?;

    let speakers: Vec<String> = stmt.query_map(params![project_id], |row| {
        row.get(0)
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    let mut added = 0usize;
    for speaker in &speakers {
        // Vérifier si le terme existe déjà
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM glossary WHERE project_id = ?1 AND term = ?2",
            params![project_id, speaker],
            |row| row.get::<_, i64>(0),
        ).map(|c| c > 0).unwrap_or(false);

        if !exists {
            conn.execute(
                "INSERT INTO glossary (project_id, term, translation, note, match_mode)
                 VALUES (?1, ?2, ?3, 'Nom de personnage — à vérifier', 'exact')",
                params![project_id, speaker, speaker],
            ).map_err(|e| e.to_string())?;
            added += 1;
        }
    }
    Ok(added)
}
```

Enregistrer dans `lib.rs` :
```rust
mod commands { pub mod glossary; /* ... */ }
.invoke_handler(tauri::generate_handler![
    // ... commandes existantes ...
    commands::glossary::list_glossary,
    commands::glossary::add_glossary_term,
    commands::glossary::update_glossary_term,
    commands::glossary::delete_glossary_term,
    commands::glossary::import_speakers_to_glossary,
])
```

### 2. Injection glossaire dans les prompts (T06 — translate.rs)

Dans `build_prompt()`, charger et injecter le glossaire :

```rust
// src-tauri/src/commands/translate.rs
fn build_glossary_section(glossary: &[GlossaryEntry]) -> String {
    if glossary.is_empty() {
        return "No glossary.".into();
    }
    glossary.iter().map(|g| {
        let mut line = format!("- \"{}\" → \"{}\"", g.term, g.translation);
        if !g.note.is_empty() {
            line.push_str(&format!(" ({})", g.note));
        }
        line
    }).collect::<Vec<_>>().join("\n")
}
```

### 3. QC Glossaire post-traduction (translate.rs)

Après chaque string traduit, vérifier la conformité au glossaire :

```rust
fn check_glossary_compliance(
    translation: &str,
    source_text: &str,
    glossary: &[GlossaryEntry],
) -> Vec<String> {
    let mut issues = Vec::new();
    for g in glossary {
        let term_present = match g.match_mode.as_str() {
            "exact" => source_text.contains(&g.term),
            "contains" => source_text.to_lowercase().contains(&g.term.to_lowercase()),
            _ => false,
        };
        if term_present && !translation.contains(&g.translation) {
            issues.push(format!("'{}' devrait être '{}'", g.term, g.translation));
        }
    }
    issues
}
```

Si des issues sont détectées → marquer le string `status = 'review'` en SQLite.

### 4. Composable — src/composables/useGlossary.ts

```typescript
import { invoke } from '@tauri-apps/api/core';
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query';
import type { GlossaryEntry } from '@/types/glossary';

export function useGlossary(projectId: number) {
  return useQuery({
    queryKey: ['glossary', projectId],
    queryFn: () => invoke<GlossaryEntry[]>('list_glossary', { projectId }),
  });
}

export function useAddGlossaryTerm(projectId: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: NewTerm) => invoke('add_glossary_term', { projectId, ...data }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['glossary', projectId] }),
  });
}

export function useDeleteGlossaryTerm(projectId: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (termId: number) => invoke('delete_glossary_term', { termId }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['glossary', projectId] }),
  });
}

export function useImportSpeakers(projectId: number) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => invoke<number>('import_speakers_to_glossary', { projectId }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['glossary', projectId] }),
  });
}
```

### 5. Vue — src/views/GlossaryView.vue

```vue
<script setup lang="ts">
import { useRoute } from 'vue-router';
import { useGlossary, useDeleteGlossaryTerm, useImportSpeakers } from '@/composables/useGlossary';

const route = useRoute();
const projectId = Number(route.params.id);

const { data: terms } = useGlossary(projectId);
const { mutate: deleteTerm } = useDeleteGlossaryTerm(projectId);
const { mutate: importSpeakers } = useImportSpeakers(projectId);
</script>

<template>
  <!-- Table shadcn-vue : Terme | Traduction | Note | Mode | Actions -->
  <!-- Dialog : formulaire ajout/édition terme -->
  <!-- Bouton "Importer speakers" → importSpeakers() -->
  <!-- Filtre par mode (exact / contains / regex) -->
  <!-- Séparation glossaire projet vs global (project_id IS NULL) -->
</template>
```

---

## Validation

- Ajouter un terme → apparaît dans la liste
- Le terme est injecté dans le prompt lors de la prochaine traduction
- QC : un string traduit avec le mauvais terme est marqué `status = 'review'`
- Import speakers : les noms de personnages (event_code 101) apparaissent dans le glossaire
