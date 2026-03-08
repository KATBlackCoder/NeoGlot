# T05 — Module RPG Maker MV/MZ : Extraction

**Statut** : DONE
**Dépendances** : T04 (projets créés), T01 (Cargo.toml prêt)

---

## Objectif

Implémenter l'extraction des textes RPG Maker MV/MZ via la crate Rust `rvpacker-txt-rs-lib`, stocker en SQLite, et afficher les fichiers dans l'UI.

---

## Étapes

### 1. Ajouter les crates dans Cargo.toml

```toml
[dependencies]
rvpacker-txt-rs-lib = "11.1.2"
walkdir = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"      # pour hashes SHA256 côté Rust
```

### 2. Créer src-tauri/src/commands/engines/rpgmv/extract.rs

```rust
use rvpacker_txt_rs_lib::{GameType, read_dir};
use std::path::Path;
use sha2::{Sha256, Digest};

#[derive(serde::Serialize)]
pub struct ExtractedString {
    pub source_hash: String,
    pub source_text: String,
    pub context_path: String,
    pub event_code: Option<i32>,
    pub row_index: i32,
    pub file_path: String,
}

#[tauri::command]
pub fn extract_rpgmv(
    game_path: String,
    work_path: String,
) -> Result<Vec<ExtractedString>, String> {
    let game = Path::new(&game_path);
    let work = Path::new(&work_path);

    // Créer le dossier de travail
    std::fs::create_dir_all(work).map_err(|e| e.to_string())?;

    // Copier les fichiers data dans work_path
    let data_src = if game.join("www").join("data").exists() {
        game.join("www").join("data")    // MV
    } else {
        game.join("data")               // MZ
    };

    // Utiliser rvpacker-txt-rs-lib pour extraire les textes
    let game_type = GameType::default(); // détecte MV vs MZ automatiquement
    let strings = read_dir(&data_src, &game_type)
        .map_err(|e| format!("Erreur extraction: {e}"))?;

    // Convertir en ExtractedString avec hash
    let result = strings.into_iter().enumerate().map(|(i, s)| {
        let mut hasher = Sha256::new();
        hasher.update(s.original.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        ExtractedString {
            source_hash: hash,
            source_text: s.original,
            context_path: s.path.unwrap_or_default(),
            event_code: s.event_code.map(|c| c as i32),
            row_index: i as i32,
            file_path: s.file.unwrap_or_default(),
        }
    }).collect();

    Ok(result)
}

// Extraction des speakers (noms de personnages) pour pré-remplir le glossaire
#[tauri::command]
pub fn extract_speakers(game_path: String) -> Result<Vec<String>, String> {
    let data_path = if Path::new(&game_path).join("www").join("data").exists() {
        format!("{}/www/data", game_path)
    } else {
        format!("{}/data", game_path)
    };

    // rvpacker-txt-rs-lib a un mode speakers
    // Retourne les noms uniques trouvés dans les headers de dialogues (event 101)
    todo!("Implémenter avec rvpacker speakers mode")
}
```

Enregistrer dans `lib.rs` :
```rust
// commands/mod.rs : pub mod engines;
// commands/engines/mod.rs : pub mod rpgmv;
// commands/engines/rpgmv/mod.rs : pub mod extract;
.invoke_handler(tauri::generate_handler![
    commands::detect::detect_engine,
    commands::engines::rpgmv::extract::extract_rpgmv,
    commands::engines::rpgmv::extract::extract_speakers,
])
```

### 3. Stockage SQLite via Rust (T03 — db_commands.rs)

Le stockage est géré par la commande Rust `store_strings` (définie dans T03) :

```rust
// src-tauri/src/commands/db_commands.rs
#[tauri::command]
pub fn store_strings(
    project_id: i64,
    strings: Vec<ExtractedString>,
    state: State<'_, AppState>,
) -> Result<usize, String>
```

Pas de route Python — tout passe par `invoke('store_strings', ...)` directement.

### 4. Flux d'extraction dans l'UI

```typescript
// src/views/TranslateView.vue — bouton "Extraire les textes"
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useQueryClient } from '@tanstack/vue-query';

const qc = useQueryClient();
const extracting = ref(false);

async function handleExtract() {
  extracting.value = true;

  // 1. Extraire les strings via Rust (rvpacker-txt-rs-lib)
  const strings = await invoke<ExtractedString[]>('extract_rpgmv', {
    gamePath: project.game_path,
    workPath: project.work_path,
  });

  // 2. Stocker directement en SQLite via Rust (rusqlite)
  await invoke('store_strings', { projectId: project.id, strings });

  // 3. Rafraîchir la liste de fichiers
  qc.invalidateQueries({ queryKey: ['project-files', project.id] });
  extracting.value = false;
}
```

### 5. Affichage des fichiers extraits

Dans `src/views/TranslateView.vue` :
- Panel gauche : liste des fichiers avec progression par fichier
- Panel droit : liste des strings du fichier sélectionné
- Chaque string : texte original + traduction (si disponible) + badge status
- Filtre : par status (pending / translated / reviewed)

---

## Vérifier l'API rvpacker-txt-rs-lib

Consulter la documentation/source pour les types exacts :

```bash
# Voir les types disponibles
cd /home/blackat/project/NeoGlot
cargo doc --open -p rvpacker-txt-rs-lib 2>/dev/null || true
```

Adapter les appels selon l'API réelle de la crate (version 11.1.2).

---

## Validation

- Extraction d'un jeu RPG Maker MZ test → strings correctement extraits
- Stockage SQLite → compter les rows avec `SELECT COUNT(*) FROM strings`
- Affichage dans l'UI : liste fichiers + strings dans le panel droit
- Pas de doublons (source_hash unique par projet)
