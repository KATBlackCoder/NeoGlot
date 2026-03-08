# Analyse : Bibliothèques Rust additionnelles

**Source** : Context7 MCP + crates.io
**Contexte** : Architecture Tauri-only (pas de Python). Toute la logique tourne en Rust.

---

## 1. reqwest — Client HTTP Rust pour Ollama

**Source** : https://github.com/seanmonstar/reqwest | **Licence** : MIT/Apache-2.0
**Version** : 0.11 | **Cargo** : `reqwest = { version = "0.13", features = ["json", "blocking"] }`

### Pourquoi pas ollama-rs

`ollama-rs` dépend de `gxhash` qui nécessite les instructions CPU **AES+SSE2** et refuse de compiler sans `RUSTFLAGS="-C target-cpu=native"`. Incompatible avec une build reproductible.

De plus, NeoGlot ne fait **pas de traduction temps réel** (pas de streaming token par token). L'appel Ollama est simple : `POST /api/generate` avec `"stream": false`, on attend la réponse complète. `reqwest` suffit amplement.

### API clé (appels Ollama)

```rust
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

// Appel batch — attend la réponse complète
let client = Client::new();
let resp: OllamaResponse = client
    .post("http://localhost:11434/api/generate")
    .json(&OllamaRequest { model: "llama3:8b", prompt: &prompt, stream: false })
    .send()?
    .json()?;
let translation = resp.response;

// Vérifier disponibilité Ollama
let ok = client.get("http://localhost:11434/api/tags").send()
    .map(|r| r.status().is_success())
    .unwrap_or(false);
```

**Décision** : Adopter. Remplace `ollama-rs` — plus simple, aucun problème de compilation CPU.

---

## 2. rusqlite — SQLite natif pour Rust

**Source** : https://github.com/rusqlite/rusqlite | **Licence** : MIT
**Version** : 0.32 | **Cargo** : `rusqlite = { version = "0.38", features = ["bundled"] }`

### Pourquoi `features = ["bundled"]`

Embarque SQLite directement dans le binaire — pas de dépendance système sur la machine cible. Simplifie la distribution Windows.

### Patterns utilisés dans NeoGlot

```rust
use rusqlite::{Connection, params};

// Ouverture + configuration WAL
let conn = Connection::open(&db_path)?;
conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

// INSERT
conn.execute(
    "INSERT INTO projects (name, game_path, work_path, engine) VALUES (?1, ?2, ?3, ?4)",
    params![name, game_path, work_path, engine],
)?;
let id = conn.last_insert_rowid();

// SELECT
let mut stmt = conn.prepare("SELECT id, name, engine FROM projects ORDER BY updated_at DESC")?;
let projects: Vec<Project> = stmt.query_map([], |row| {
    Ok(Project { id: row.get(0)?, name: row.get(1)?, engine: row.get(2)? })
})?.collect::<Result<_, _>>()?;

// UPDATE
conn.execute(
    "UPDATE strings SET translation = ?1, status = 'translated' WHERE id = ?2",
    params![translation, string_id],
)?;
```

### AppState pattern

```rust
pub struct AppState {
    pub db_path: PathBuf,
    pub translation_running: Mutex<bool>,
}

// Dans chaque commande :
#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let conn = Connection::open(&state.db_path).map_err(|e| e.to_string())?;
    // ...
}
```

**Décision** : Adopter. Remplace SQLAlchemy (Python) + tauri-plugin-sql (pour l'écriture).

---

## 3. tokio — Runtime async Rust

**Source** : https://tokio.rs | **Licence** : MIT
**Version** : 1 | **Cargo** : `tokio = { version = "1", features = ["full"] }`

### Pourquoi

Tauri utilise tokio en interne. Les commandes `async fn` (appels Ollama) nécessitent un runtime async. `features = ["full"]` active tout (threads, I/O, timers).

```rust
// Commande async Tauri — tokio est déjà le runtime
#[tauri::command]
pub async fn start_translation(
    project_id: i64,
    model: String,
    on_progress: Channel<TranslationProgress>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Les appels reqwest blocking tournent dans un thread dédié (spawn_blocking)
    let client = reqwest::blocking::Client::new();
    // ...
}
```

**Décision** : Adopter. Requis pour les commandes async Tauri (reqwest blocking s'exécute via `spawn_blocking`).

---

## 4. regex — Expressions régulières Rust

**Source** : https://github.com/rust-lang/regex | **Licence** : MIT/Apache-2.0
**Version** : 1 | **Cargo** : `regex = "1"`

### Usage dans NeoGlot (protection placeholders)

Les textes RPG Maker contiennent des codes de contrôle (`\SE[3]`, `\BGM[1]`, `\n[2]`, etc.) qui ne doivent pas être traduits. La crate `regex` permet de les détecter et remplacer avant envoi à Ollama.

```rust
use regex::Regex;
use once_cell::sync::Lazy;

// Patterns compilés une seule fois (Lazy pour éviter re-compilation)
static PLACEHOLDER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\\(SE|BGM|ME|SE|V|N|C|I|[A-Z]+)\[(\d+)\]").unwrap()
});

pub fn protect_placeholders(text: &str) -> (String, Vec<(String, String)>) {
    let mut mapping = Vec::new();
    let mut counter = 0;
    let result = PLACEHOLDER_RE.replace_all(text, |caps: &regex::Captures| {
        let token = format!("<PH_{}>", counter);
        mapping.push((token.clone(), caps[0].to_string()));
        counter += 1;
        token
    });
    (result.into_owned(), mapping)
}

pub fn restore_placeholders(text: &str, mapping: &[(String, String)]) -> String {
    let mut result = text.to_string();
    for (token, original) in mapping {
        result = result.replace(token.as_str(), original.as_str());
    }
    result
}
```

**Décision** : Adopter. Requis pour protéger les codes RPG Maker avant traduction.

---

## Récapitulatif Cargo.toml

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
tauri-plugin-sql = { version = "2", features = ["sqlite"] }
tauri-plugin-store = "2"
tauri-plugin-log = "2"
tauri-plugin-window-state = "2"
tauri-plugin-notification = "2"
tauri-plugin-process = "2"
tauri-plugin-clipboard-manager = "2"

# Logique métier
reqwest = { version = "0.13", features = ["json", "blocking"] }
rusqlite = { version = "0.38", features = ["bundled"] }
tokio = { version = "1", features = ["full"] }
regex = "1"
once_cell = "1"

# Parsing RPG Maker
rvpacker-txt-rs-lib = "11.1.2"
rpgmad-lib = "4.0.0"
marshal-rs = "*"

# Utilitaires
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
walkdir = "2"
log = "^0.4"
```
