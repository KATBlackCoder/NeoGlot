# T07 — Module RPG Maker MV/MZ : Réinjection

**Statut** : TODO
**Dépendances** : T06 (traduction complète en SQLite)

---

## Objectif

Réinjecter les traductions dans les fichiers RPG Maker MV/MZ via `rvpacker-txt-rs-lib`, sans modifier les fichiers originaux (travail sur `work_path`).

---

## Étapes

### 1. Commande Rust — write_rpgmv

Créer `src-tauri/src/commands/write.rs` :

```rust
use rvpacker_txt_rs_lib::{GameType, write_dir};
use std::path::Path;

#[derive(serde::Deserialize)]
pub struct TranslatedString {
    pub source_hash: String,
    pub source_text: String,
    pub translation: String,
    pub context_path: String,
    pub row_index: i32,
    pub file_path: String,
}

#[tauri::command]
pub fn write_rpgmv(
    game_path: String,
    work_path: String,
    translations: Vec<TranslatedString>,
) -> Result<String, String> {
    let work = Path::new(&work_path);

    // Créer le dossier de sortie
    std::fs::create_dir_all(work).map_err(|e| e.to_string())?;

    // Copier les fichiers originaux vers work_path si pas déjà fait
    let data_src = {
        let p = Path::new(&game_path);
        if p.join("www").join("data").exists() { p.join("www").join("data") }
        else { p.join("data") }
    };
    let data_dst = work.join("data");
    if !data_dst.exists() {
        copy_dir_all(&data_src, &data_dst).map_err(|e| e.to_string())?;
    }

    // Convertir les traductions au format rvpacker-txt-rs-lib
    // (adapter selon l'API exacte de la crate)
    let game_type = GameType::default();
    let translated_entries: Vec<_> = translations.iter().map(|t| {
        // Adapter au type attendu par write_dir
        todo!("adapter selon API rvpacker-txt-rs-lib")
    }).collect();

    write_dir(&data_dst, &translated_entries, &game_type)
        .map_err(|e| format!("Erreur réinjection: {e}"))?;

    Ok(work_path)
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
```

### 2. Commande Rust — get_project_strings (T03 — db_commands.rs)

La récupération des strings traduits est gérée par `get_project_strings` (définie dans T03) :

```rust
// src-tauri/src/commands/db_commands.rs
#[tauri::command]
pub fn get_project_strings(
    project_id: i64,
    status_filter: Option<String>, // "translated" | "reviewed" | None = tous
    state: State<'_, AppState>,
) -> Result<Vec<StringRow>, String>
```

Pas de route Python — tout passe par `invoke('get_project_strings', ...)`.

### 3. Flux d'export dans l'UI

```typescript
// src/views/TranslateView.vue — bouton "Exporter"
import { invoke } from '@tauri-apps/api/core';
import { message } from '@tauri-apps/plugin-dialog';

async function handleExport() {
  // 1. Récupérer les strings traduits depuis SQLite via Rust
  const strings = await invoke<TranslatedString[]>('get_project_strings', {
    projectId: project.id,
    statusFilter: 'translated',
  });

  if (project.engine === 'rpgmv' || project.engine === 'rpgmz') {
    // 2. Réinjecter via Rust (rvpacker-txt-rs-lib)
    const outputPath = await invoke<string>('write_rpgmv', {
      gamePath: project.game_path,
      workPath: project.work_path,
      translations: strings,
    });

    // 3. Notifier l'utilisateur
    await message(
      `Export terminé ! Les fichiers traduits sont dans :\n${outputPath}`,
      { title: 'Export réussi' }
    );
  }
}
```

### 4. Validation de l'export

Avant réinjection, vérifier :
- Nombre de strings traduits vs total (avertir si < 100%)
- Présence de placeholders non restaurés (erreurs)
- Afficher un résumé dans un Dialog : "1234/1500 strings traduits. Continuer ?"

---

## Note importante

Vérifier l'API exacte de `rvpacker-txt-rs-lib` v11.1.2 pour le mode écriture :
- `write_dir()` ou équivalent
- Format des entrées : est-ce que ça prend le `context_path` ou le `row_index` pour positionner ?
- La crate est issue de rpgmtranslate-qt — voir `src/lib.rs` de la crate pour les types

---

## Validation

- Exporter un projet RPG Maker MZ traduit → fichiers JSON dans `work_path/data/`
- Ouvrir le jeu depuis `work_path` → les textes traduits apparaissent en jeu
- Fichiers originaux dans `game_path` inchangés
