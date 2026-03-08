# T09 — Module Wolf RPG (UberWolf + WolfTL)

**Statut** : TODO
**Dépendances** : T04 (projets), T06 (moteur de traduction)

---

## Objectif

Supporter les jeux Wolf RPG Editor via les outils UberWolf (déchiffrement) et WolfTL (extraction/réinjection). Tout en Rust via `std::process::Command` + `rusqlite` + `invoke()`.

---

## Outils requis

| Outil | Source | Licence | Binaire |
|-------|--------|---------|---------|
| UberWolf | github.com/Sinflower/UberWolf | MIT | UberWolfCli.exe (Windows) |
| WolfTL | github.com/Sinflower/WolfTL | MIT | WolfTL.exe (Windows) |

### Stratégie Linux

- Bundler les `.exe` dans les ressources Tauri (`src-tauri/resources/`)
- Exécuter via `wine <exe>` sur Linux
- Sur Windows : exécuter directement

```rust
// Détecter l'OS et adapter la commande
fn get_wolf_command(exe_name: &str, exe_path: &Path) -> std::process::Command {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new(exe_path)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = std::process::Command::new("wine");
        cmd.arg(exe_path);
        cmd
    }
}
```

---

## Étapes

### 1. Bundler les outils Wolf RPG

Les binaires sont placés dans `src-tauri/resources/` (déjà fait) :
- `src-tauri/resources/UberWolfCli.exe` ✅
- `src-tauri/resources/WolfTL.exe` ✅

Dans `tauri.conf.json` (déjà configuré) :
```json
{
  "bundle": {
    "resources": ["resources/*"]
  }
}
```

Dans Rust, obtenir le chemin des ressources :
```rust
use tauri::Manager;

fn get_wolf_exe(app: &tauri::AppHandle, name: &str) -> PathBuf {
    app.path().resource_dir()
        .unwrap()
        .join("resources")
        .join(name)
}
```

### 2. Commandes Rust — src-tauri/src/commands/engines/wolf/

```rust
use std::path::Path;
use std::process::Command;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use crate::AppState;

/// Vérifie si un fichier GameDat.wolf est chiffré (magic bytes)
fn is_encrypted(game_dat_path: &Path) -> bool {
    use std::io::Read;
    if !game_dat_path.exists() { return false; }
    let mut f = match std::fs::File::open(game_dat_path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut magic = [0u8; 4];
    f.read_exact(&mut magic).unwrap_or(());
    &magic != b"WOLF"
}

/// Lancement d'un exe Wolf avec wine sur Linux, directement sur Windows
fn run_wolf_exe(app: &AppHandle, exe_name: &str, args: &[&str]) -> Result<(), String> {
    let exe_path = app.path().resource_dir()
        .map_err(|e| e.to_string())?
        .join("resources")
        .join(exe_name);

    #[cfg(target_os = "windows")]
    let status = Command::new(&exe_path)
        .args(args)
        .status()
        .map_err(|e| format!("Erreur lancement {exe_name}: {e}"))?;

    #[cfg(not(target_os = "windows"))]
    let status = Command::new("wine")
        .arg(&exe_path)
        .args(args)
        .status()
        .map_err(|e| format!("Erreur lancement wine {exe_name}: {e}"))?;

    if status.success() { Ok(()) }
    else { Err(format!("{exe_name} a échoué (code {:?})", status.code())) }
}

#[tauri::command]
pub async fn extract_wolf(
    project_id: i64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let conn = Connection::open(&state.db_path).map_err(|e| e.to_string())?;

    // Récupérer les infos du projet
    let (game_path, work_path): (String, String) = conn.query_row(
        "SELECT game_path, work_path FROM projects WHERE id = ?1",
        params![project_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|e| e.to_string())?;

    let game = Path::new(&game_path);
    let work = Path::new(&work_path);
    let decrypted = work.join("wolf_decrypted");
    let dump = work.join("wolf_dump");

    std::fs::create_dir_all(&decrypted).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dump).map_err(|e| e.to_string())?;

    // 1. Copier le jeu dans decrypted/
    copy_dir_all(game, &decrypted).map_err(|e| e.to_string())?;

    // 2. Déchiffrement si nécessaire (UberWolfCli)
    let game_dat = decrypted.join("Data").join("BasicData").join("GameDat.wolf");
    if is_encrypted(&game_dat) {
        run_wolf_exe(&app, "UberWolfCli.exe", &[decrypted.to_str().unwrap()])?;
    }

    // 3. Extraction via WolfTL --create
    run_wolf_exe(&app, "WolfTL.exe", &[
        "--create",
        decrypted.to_str().unwrap(),
        dump.to_str().unwrap(),
    ])?;

    // 4. Lire les JSON produits par WolfTL et stocker en SQLite
    let strings = read_wolf_dump(&dump)?;
    let count = store_wolf_strings(&conn, project_id, &strings, &work_path)?;

    Ok(count)
}

#[tauri::command]
pub async fn inject_wolf(
    project_id: i64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let conn = Connection::open(&state.db_path).map_err(|e| e.to_string())?;

    let (game_path, work_path): (String, String) = conn.query_row(
        "SELECT game_path, work_path FROM projects WHERE id = ?1",
        params![project_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|e| e.to_string())?;

    let work = Path::new(&work_path);
    let dump = work.join("wolf_dump");
    let decrypted = work.join("wolf_decrypted");
    let output = work.join("wolf_output");

    std::fs::create_dir_all(&output).map_err(|e| e.to_string())?;

    // 1. Récupérer les traductions depuis SQLite
    let translations = get_wolf_translations(&conn, project_id)?;

    // 2. Écrire les traductions dans les JSON WolfTL
    write_wolf_dump(&dump, &translations)?;

    // 3. Copier les fichiers déchiffrés dans output/
    copy_dir_all(&decrypted, &output).map_err(|e| e.to_string())?;

    // 4. Réinjection via WolfTL --patch
    run_wolf_exe(&app, "WolfTL.exe", &[
        "--patch",
        dump.to_str().unwrap(),
        output.to_str().unwrap(),
    ])?;

    Ok(output.to_string_lossy().into_owned())
}

// --- Helpers ---

#[derive(Serialize, Deserialize)]
struct WolfString {
    source_text: String,
    context_path: String,
    row_index: i32,
    file_path: String,
}

fn read_wolf_dump(dump: &Path) -> Result<Vec<WolfString>, String> {
    let mut strings = Vec::new();
    for entry in walkdir::WalkDir::new(dump).into_iter().flatten() {
        if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
            let rel_path = entry.path().strip_prefix(dump)
                .unwrap().to_string_lossy().into_owned();
            let content = std::fs::read_to_string(entry.path())
                .map_err(|e| e.to_string())?;
            let data: Vec<serde_json::Value> = serde_json::from_str(&content)
                .map_err(|e| e.to_string())?;
            for (i, entry) in data.iter().enumerate() {
                if let Some(original) = entry.get("Original").and_then(|v| v.as_str()) {
                    if !original.is_empty() {
                        strings.push(WolfString {
                            source_text: original.to_string(),
                            context_path: format!("{}/{}", rel_path, i),
                            row_index: i as i32,
                            file_path: rel_path.clone(),
                        });
                    }
                }
            }
        }
    }
    Ok(strings)
}

fn store_wolf_strings(
    conn: &Connection,
    project_id: i64,
    strings: &[WolfString],
    work_path: &str,
) -> Result<usize, String> {
    use sha2::{Sha256, Digest};
    use std::collections::HashMap;

    // Regrouper par fichier
    let mut files: HashMap<&str, Vec<&WolfString>> = HashMap::new();
    for s in strings {
        files.entry(&s.file_path).or_default().push(s);
    }

    let mut total = 0usize;
    for (file_path, file_strings) in &files {
        conn.execute(
            "INSERT OR REPLACE INTO files (project_id, relative_path, strings_total)
             VALUES (?1, ?2, ?3)",
            params![project_id, file_path, file_strings.len() as i64],
        ).map_err(|e| e.to_string())?;
        let file_id = conn.last_insert_rowid();

        for s in file_strings {
            let mut hasher = Sha256::new();
            hasher.update(s.source_text.as_bytes());
            let hash = format!("{:x}", hasher.finalize());

            conn.execute(
                "INSERT OR IGNORE INTO strings
                 (file_id, source_hash, source_text, context_path, row_index, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'pending')",
                params![file_id, hash, s.source_text, s.context_path, s.row_index],
            ).map_err(|e| e.to_string())?;
            total += 1;
        }
    }
    Ok(total)
}

fn get_wolf_translations(conn: &Connection, project_id: i64) -> Result<Vec<serde_json::Value>, String> {
    let mut stmt = conn.prepare(
        "SELECT s.source_text, s.translation, s.context_path, s.row_index, f.relative_path
         FROM strings s JOIN files f ON s.file_id = f.id
         WHERE f.project_id = ?1 AND s.status IN ('translated', 'reviewed')
         ORDER BY s.row_index"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![project_id], |row| {
        Ok(serde_json::json!({
            "source_text": row.get::<_, String>(0)?,
            "translation": row.get::<_, String>(1)?,
            "context_path": row.get::<_, String>(2)?,
            "row_index": row.get::<_, i32>(3)?,
            "file_path": row.get::<_, String>(4)?,
        }))
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(rows)
}

fn write_wolf_dump(dump: &Path, translations: &[serde_json::Value]) -> Result<(), String> {
    // Regrouper par file_path
    let mut files: std::collections::HashMap<String, Vec<&serde_json::Value>> = std::collections::HashMap::new();
    for t in translations {
        let fp = t["file_path"].as_str().unwrap_or("").to_string();
        files.entry(fp).or_default().push(t);
    }

    for (file_path, file_translations) in &files {
        let json_path = dump.join(file_path);
        if !json_path.exists() { continue; }

        let content = std::fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
        let mut data: Vec<serde_json::Value> = serde_json::from_str(&content)
            .map_err(|e| e.to_string())?;

        for t in file_translations {
            let idx = t["row_index"].as_i64().unwrap_or(0) as usize;
            if idx < data.len() {
                data[idx]["Translation"] = t["translation"].clone();
            }
        }

        let updated = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
        std::fs::write(&json_path, updated).map_err(|e| e.to_string())?;
    }
    Ok(())
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

Enregistrer dans `lib.rs` :
```rust
mod commands { pub mod wolf; /* ... */ }
.invoke_handler(tauri::generate_handler![
    // ... commandes existantes ...
    commands::wolf::extract_wolf,
    commands::wolf::inject_wolf,
])
```

### 3. Intégration dans l'UI

Dans `TranslateView.vue`, les boutons "Extraire" et "Exporter" appellent `invoke()` directement :

```typescript
// src/views/TranslateView.vue
import { invoke } from '@tauri-apps/api/core';
import { useQueryClient } from '@tanstack/vue-query';

const qc = useQueryClient();

// Extraction Wolf RPG
async function handleExtractWolf() {
  const count = await invoke<number>('extract_wolf', { projectId: project.id });
  qc.invalidateQueries({ queryKey: ['project-files', project.id] });
  toast.success(`${count} strings extraits`);
}

// Réinjection Wolf RPG
async function handleInjectWolf() {
  const outputPath = await invoke<string>('inject_wolf', { projectId: project.id });
  toast.success(`Export terminé → ${outputPath}`);
}
```

---

## Note : Permissions shell (tauri.conf.json)

Les exécutables Wolf RPG sont lancés via `std::process::Command` en Rust — **pas** via `tauri-plugin-shell` depuis le frontend. Aucune permission shell supplémentaire nécessaire dans `capabilities/default.json`.

---

## Validation

- Projet Wolf RPG non chiffré : extraction directe sans UberWolf
- Projet Wolf RPG chiffré v3.5 : UberWolf déchiffre correctement
- WolfTL extrait les textes → JSON lisibles dans `wolf_dump/`
- Traduction via pipeline T06 → strings stockés en SQLite
- Réinjection via WolfTL --patch → jeu joue avec textes traduits
- Linux : Wine + exe bundlé fonctionne
