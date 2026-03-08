# T10 — Module RPG Maker XP/VX/VXAce (marshal-rs + rpgmad-lib)

**Statut** : TODO
**Dépendances** : T05 (RPG Maker MV/MZ extraction), T07 (réinjection)

---

## Objectif

Étendre le support RPG Maker aux versions classiques (XP, VX, VXAce) qui utilisent le format binaire Ruby Marshal et les archives `.rgss*`.

---

## Différences par rapport à MV/MZ

| Aspect | MV/MZ | XP/VX/VXAce |
|--------|-------|-------------|
| Format données | JSON | Ruby Marshal binaire (.rxdata/.rvdata/.rvdata2) |
| Archive | Aucune | .rgss / .rgss2 / .rgss3 |
| Scripts | JavaScript | Ruby (non traduit) |
| Crates Rust | rvpacker-txt-rs-lib | + marshal-rs + rpgmad-lib |

---

## Étapes

### 1. Ajouter les crates dans Cargo.toml

```toml
[dependencies]
# Existant depuis T05 :
rvpacker-txt-rs-lib = "11.1.2"
walkdir = "2"
sha2 = "0.10"

# Nouveau pour XP/VX/VXAce :
marshal-rs = "*"              # parsing Ruby Marshal binaire
rpgmad-lib = "4.0.0"         # déchiffrement .rgss/.rgss2/.rgss3
```

### 2. Commande Rust — decrypt_rgss

Implémenter dans `src-tauri/src/commands/engines/rpgm_classic/decrypt.rs` :

```rust
use rpgmad_lib::Decrypter;
use std::path::Path;

#[tauri::command]
pub fn decrypt_rgss(
    archive_path: String,
    output_path: String,
) -> Result<String, String> {
    let archive = Path::new(&archive_path);
    let output = Path::new(&output_path);

    // Vérifier que l'archive existe
    if !archive.exists() {
        return Err(format!("Archive introuvable : {archive_path}"));
    }

    // Créer le dossier de sortie
    std::fs::create_dir_all(output).map_err(|e| e.to_string())?;

    // Déchiffrer l'archive .rgss / .rgss2 / .rgss3
    let mut decrypter = Decrypter::new(archive).map_err(|e| e.to_string())?;
    decrypter.extract(output).map_err(|e| format!("Erreur déchiffrement: {e}"))?;

    Ok(output_path)
}
```

Les chemins dans `lib.rs` (déjà configurés) :
```rust
commands::engines::rpgm_classic::decrypt::decrypt_rgss,
commands::engines::rpgm_classic::extract::extract_rpgm_classic,
```

### 3. Commande Rust — extract_rpgm_classic

Dans `src-tauri/src/commands/engines/rpgm_classic/extract.rs` :

```rust
#[tauri::command]
pub fn extract_rpgm_classic(
    game_path: String,
    work_path: String,
) -> Result<Vec<ExtractedString>, String> {
    let game = Path::new(&game_path);
    let work = Path::new(&work_path);
    std::fs::create_dir_all(work).map_err(|e| e.to_string())?;

    // Détecter l'extension archive
    let (archive, engine_type) = if game.join("Game.rgss3").exists() {
        (game.join("Game.rgss3"), "rpgmvxa")
    } else if game.join("Game.rgss2").exists() {
        (game.join("Game.rgss2"), "rpgmvx")
    } else if game.join("Game.rgss").exists() {
        (game.join("Game.rgss"), "rpgmxp")
    } else {
        // Pas d'archive, données déjà accessibles dans Data/
        return extract_classic_from_data(&game.join("Data"), work);
    };

    // Déchiffrer l'archive dans un dossier temporaire
    let temp_data = work.join("_rgss_extracted");
    std::fs::create_dir_all(&temp_data).map_err(|e| e.to_string())?;
    let mut decrypter = rpgmad_lib::Decrypter::new(&archive).map_err(|e| e.to_string())?;
    decrypter.extract(&temp_data).map_err(|e| e.to_string())?;

    extract_classic_from_data(&temp_data, work)
}

fn extract_classic_from_data(
    data_path: &Path,
    work: &Path,
) -> Result<Vec<ExtractedString>, String> {
    // rvpacker-txt-rs-lib supporte les .rvdata/.rxdata via marshal-rs
    // (la crate intègre marshal-rs en interne)
    let game_type = rvpacker_txt_rs_lib::GameType::default();
    let strings = rvpacker_txt_rs_lib::read_dir(data_path, &game_type)
        .map_err(|e| format!("Erreur parsing Marshal: {e}"))?;

    Ok(strings.into_iter().enumerate().map(|(i, s)| {
        let mut hasher = sha2::Sha256::new();
        sha2::Digest::update(&mut hasher, s.original.as_bytes());
        let hash = format!("{:x}", sha2::Digest::finalize(hasher));

        ExtractedString {
            source_hash: hash,
            source_text: s.original,
            context_path: s.path.unwrap_or_default(),
            event_code: s.event_code.map(|c| c as i32),
            row_index: i as i32,
            file_path: s.file.unwrap_or_default(),
        }
    }).collect())
}
```

### 4. Adapter le flux d'extraction dans l'UI

Dans `TranslateView.vue`, adapter `handleExtract` — tout via `invoke()`, pas d'HTTP :

```typescript
// src/views/TranslateView.vue
import { invoke } from '@tauri-apps/api/core';
import { useQueryClient } from '@tanstack/vue-query';

const qc = useQueryClient();

async function handleExtract() {
  let strings: ExtractedString[];

  if (project.engine === 'wolf') {
    // Wolf RPG : commande Rust qui lance WolfTL en subprocess (voir T09)
    await invoke('extract_wolf', { projectId: project.id });
    qc.invalidateQueries({ queryKey: ['project-files', project.id] });
    return;
  } else if (['rpgmxp', 'rpgmvx', 'rpgmvxa'].includes(project.engine)) {
    // RPG Maker classique (marshal-rs + rpgmad-lib)
    strings = await invoke<ExtractedString[]>('extract_rpgm_classic', {
      gamePath: project.game_path,
      workPath: project.work_path,
    });
  } else {
    // RPG Maker MV/MZ (rvpacker-txt-rs-lib)
    strings = await invoke<ExtractedString[]>('extract_rpgmv', {
      gamePath: project.game_path,
      workPath: project.work_path,
    });
  }

  // Stocker en SQLite via Rust (rusqlite)
  await invoke('store_strings', { projectId: project.id, strings });
  qc.invalidateQueries({ queryKey: ['project-files', project.id] });
}
```

### 5. Réinjection RPG Maker classique

La réinjection utilise aussi `rvpacker-txt-rs-lib` en mode écriture (même logique que T07).
La commande `write_rpgmv` dans `commands/write.rs` devrait fonctionner pour XP/VX/VXAce aussi,
car la crate détecte le format automatiquement via `GameType`.

### 6. Différences éventuelles XP vs VX vs VXAce

| Moteur | Archive | Dossier Data | Fichiers |
|--------|---------|-------------|---------|
| XP | Game.rgss | Data/ | *.rxdata |
| VX | Game.rgss2 | Data/ | *.rvdata |
| VXAce | Game.rgss3 | Data/ | *.rvdata2 |

Tous supportés par `rpgmad-lib` et `rvpacker-txt-rs-lib`.

---

## Vérification des crates

Avant d'implémenter, vérifier les APIs exactes :

```bash
# Dans le projet NeoGlot
cargo doc --open 2>/dev/null
# Ou consulter crates.io :
# https://crates.io/crates/rpgmad-lib
# https://crates.io/crates/marshal-rs
```

---

## Validation

- Projet RPG Maker VXAce avec archive .rgss3 → déchiffrement + extraction
- Projet RPG Maker XP sans archive → extraction directe depuis Data/
- Strings extraits identiques à ceux qu'on verrait dans le jeu
- Réinjection → jeu joue avec textes traduits
