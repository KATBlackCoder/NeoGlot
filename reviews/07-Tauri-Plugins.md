# Analyse : Plugins Tauri v2 utiles pour NeoGlot

**Source** : https://tauri.app/plugin/ + Context7 MCP (`/tauri-apps/plugins-workspace`)
**Version Tauri** : v2.x (API stable)

---

## Tableau de pertinence rapide

| Plugin | Rôle | Pertinence NeoGlot | Priorité |
|--------|------|-------------------|----------|
| `tauri-plugin-shell` | Lancer des sous-processus | **Critique** | P0 |
| `tauri-plugin-fs` | Accès fichiers/dossiers | **Critique** | P0 |
| `tauri-plugin-dialog` | Sélecteurs fichiers/dossiers | **Critique** | P0 |
| `tauri-plugin-sql` | SQLite embarqué | **Critique** | P0 |
| `tauri-plugin-store` | Config persistante légère | Utile | P1 |
| `tauri-plugin-http` | Requêtes HTTP (Ollama) | Utile | P1 |
| `tauri-plugin-window-state` | Sauvegarde taille/position fenêtre | Confort | P2 |
| `tauri-plugin-notification` | Notifications système | Confort | P2 |
| `tauri-plugin-clipboard-manager` | Presse-papiers | Optionnel | P3 |
| `tauri-plugin-process` | Quitter/relancer l'app | Utile | P2 |

---

## Plugins P0 — Critiques (obligatoires au MVP)

### 1. `tauri-plugin-shell`

**Rôle** : Lancer des processus externes depuis le frontend ou Rust.

**Pourquoi critique pour NeoGlot** :
- Lancer le backend Python (`uvicorn backend.main:app`)
- Lancer `WolfTL` (extraction/réinjection Wolf RPG)
- Lancer `UberWolfCli` (déchiffrement archives Wolf RPG)
- Tuer ces processus proprement à la fermeture de l'app

**API frontend** :
```typescript
import { Command } from '@tauri-apps/plugin-shell';

// Lancer le backend Python
const python = Command.create('python', ['-m', 'uvicorn', 'backend.main:app', '--port', '8000']);
const child = await python.spawn();

// Écouter stdout en streaming (logs Ollama, progression)
child.stdout.on('data', (line) => console.log(line));
child.stderr.on('data', (line) => console.error(line));

// Tuer proprement
await child.kill();

// Appel synchrone simple (WolfTL)
const output = await Command.create('wolftl', [dataPath, dumpPath, '--create']).execute();
```

**Configuration `capabilities/default.json`** :
```json
{
  "permissions": [
    "shell:allow-execute",
    "shell:allow-spawn",
    "shell:allow-kill"
  ]
}
```

**Configuration `tauri.conf.json`** (allowlist des commandes autorisées) :
```json
{
  "plugins": {
    "shell": {
      "open": false,
      "commands": [
        { "name": "python", "cmd": "python3", "args": true },
        { "name": "wolftl", "cmd": "WolfTL", "args": true },
        { "name": "uberwolf", "cmd": "UberWolfCli", "args": true }
      ]
    }
  }
}
```

**Note** : Le sidecar pattern (bundler l'exécutable avec l'app) est aussi supporté via `sidecar: true`.

---

### 2. `tauri-plugin-fs`

**Rôle** : Accès lecture/écriture au système de fichiers avec permissions granulaires.

**Pourquoi critique pour NeoGlot** :
- Sélectionner/lire le dossier du jeu source
- Créer la copie de travail (`work_path`)
- Lire/écrire les JSON Wolf RPG (dump/patch)
- Surveiller les changements de fichiers (optionnel, pour sync)

**API frontend** :
```typescript
import { readTextFile, writeTextFile, readDir, exists, mkdir, copyFile, watch } from '@tauri-apps/plugin-fs';
import { BaseDirectory } from '@tauri-apps/plugin-fs';

// Lire un fichier JSON de traduction
const content = await readTextFile('/path/to/dump/mps/Map001.json');

// Écrire le JSON traduit
await writeTextFile('/path/to/dump/mps/Map001.json', JSON.stringify(data));

// Lister les fichiers d'un dossier de jeu
const entries = await readDir('/path/to/game/data');

// Vérifier l'existence
const hasData = await exists('/path/to/Game.exe');

// Surveiller les modifications (live reload)
const unwatch = await watch('/path/to/dump', (event) => {
  console.log('Fichier modifié:', event.paths);
});
```

**BaseDirectory** — chemins relatifs sécurisés :
```typescript
// Stocker config dans AppData
await writeTextFile('config.json', data, { baseDir: BaseDirectory.AppData });
// → Linux: ~/.local/share/neoglot/config.json
// → Windows: %APPDATA%\neoglot\config.json
```

**Permissions recommandées** :
```json
{
  "permissions": [
    "fs:allow-read-recursive",
    "fs:allow-write-recursive",
    "fs:allow-exists",
    "fs:allow-mkdir",
    "fs:allow-copy-file",
    "fs:scope-app-recursive"
  ]
}
```

---

### 3. `tauri-plugin-dialog`

**Rôle** : Dialogues natifs OS (sélection fichiers/dossiers, confirmations).

**Pourquoi critique pour NeoGlot** :
- Sélectionner le dossier du jeu source
- Choisir le dossier de travail de destination
- Confirmer la suppression d'un projet

**API frontend** :
```typescript
import { open, save, message, ask, confirm } from '@tauri-apps/plugin-dialog';

// Sélectionner le dossier du jeu (étape 1 création projet)
const gameDir = await open({
  directory: true,
  multiple: false,
  title: 'Sélectionner le dossier du jeu'
});

// Sélectionner un fichier .exe (pour Wolf RPG, extraction clé)
const gameExe = await open({
  filters: [{ name: 'Exécutable', extensions: ['exe'] }],
  title: 'Sélectionner Game.exe'
});

// Choisir où exporter la traduction
const outputPath = await save({
  title: 'Dossier de sortie',
  defaultPath: '/home/user/games/translated'
});

// Confirmation avant opération destructive
const confirmed = await confirm(
  'Cette opération va écraser les fichiers traduits existants. Continuer ?',
  { title: 'NeoGlot', kind: 'warning' }
);

// Message d'erreur natif
await message('Ollama n\'est pas démarré sur localhost:11434', { kind: 'error' });
```

---

### 4. `tauri-plugin-sql`

**Rôle** : Accès SQLite (+ MySQL/PostgreSQL) depuis le frontend via IPC.

**Pourquoi critique pour NeoGlot** :
- Toute la persistence : projets, fichiers, strings, cache de traduction, glossaire
- Requêtes directes depuis le frontend React sans passer par le backend Python pour les lectures UI

**Installation** :
```toml
# Cargo.toml
tauri-plugin-sql = { version = "2", features = ["sqlite"] }
```

```typescript
// package.json
"@tauri-apps/plugin-sql": "^2.0.0"
```

**API frontend** :
```typescript
import Database from '@tauri-apps/plugin-sql';

// Connexion (crée la DB si elle n'existe pas)
const db = await Database.load('sqlite:neoglot.db');

// Créer les tables
await db.execute(`
  CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    game_path TEXT NOT NULL,
    work_path TEXT NOT NULL,
    engine TEXT NOT NULL,
    source_lang TEXT DEFAULT 'ja',
    target_lang TEXT DEFAULT 'fr',
    project_context TEXT DEFAULT '',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
  )
`);

// Insérer un projet
const result = await db.execute(
  'INSERT INTO projects (name, game_path, work_path, engine) VALUES ($1, $2, $3, $4)',
  [name, gamePath, workPath, engine]
);
const newId = result.lastInsertId;

// Lire tous les projets
const projects = await db.select<Project[]>('SELECT * FROM projects ORDER BY created_at DESC');

// Requête avec params
const strings = await db.select<StringEntry[]>(
  'SELECT * FROM strings WHERE file_id = $1 AND status = $2',
  [fileId, 'pending']
);
```

**Note importante** : La DB SQLite est créée dans `BaseDirectory.AppData` automatiquement. Le chemin `sqlite:neoglot.db` résout vers `~/.local/share/neoglot/neoglot.db` sur Linux.

**Alternative** : Le backend Python (SQLAlchemy) peut gérer la DB pour les opérations complexes (batch inserts, transactions longues). Les deux peuvent coexister sur le même fichier SQLite — le frontend lit en direct pour l'UI, Python écrit pendant les traitements.

---

## Plugins P1 — Utiles (fortement recommandés)

### 5. `tauri-plugin-store`

**Rôle** : Persistance clé-valeur légère (JSON), idéal pour les préférences utilisateur.

**Pourquoi utile pour NeoGlot** :
- Sauvegarder les préférences UI (thème, langue de l'interface)
- Mémoriser les derniers dossiers ouverts
- Stocker le modèle Ollama sélectionné, la langue par défaut source/cible
- Config légère sans surcharger SQLite

**API frontend** :
```typescript
import { Store } from '@tauri-apps/plugin-store';

const store = await Store.load('preferences.json', { autoSave: true });

// Sauvegarder préférences
await store.set('ollama.model', 'llama3:8b');
await store.set('default.source_lang', 'ja');
await store.set('default.target_lang', 'fr');
await store.set('ui.theme', 'dark');
await store.set('recent.projects', ['/path/to/game1', '/path/to/game2']);

// Lire
const model = await store.get<string>('ollama.model');

// Écouter les changements
const unlisten = await store.onKeyChange('ollama.model', (newValue) => {
  console.log('Modèle Ollama changé:', newValue);
});

// LazyStore — chargement différé (meilleure perf au démarrage)
import { LazyStore } from '@tauri-apps/plugin-store';
const lazyStore = new LazyStore('preferences.json');
```

---

### 6. `tauri-plugin-http`

**Rôle** : Requêtes HTTP natives via le backend Rust (contourne les restrictions CORS).

**Pourquoi utile pour NeoGlot** :
- Vérifier si Ollama est disponible sur `localhost:11434`
- Appeler l'API Ollama (`/api/generate`, `/api/tags`, `/api/chat`) directement depuis le frontend
- Alternative au backend Python pour les appels Ollama simples

**API frontend** :
```typescript
import { fetch } from '@tauri-apps/plugin-http';

// Vérification Ollama au démarrage
async function checkOllama(): Promise<boolean> {
  try {
    const response = await fetch('http://localhost:11434/api/tags', {
      method: 'GET',
      connectTimeout: 2000,
    });
    return response.ok;
  } catch {
    return false;
  }
}

// Lister les modèles disponibles
const response = await fetch('http://localhost:11434/api/tags');
const { models } = await response.json();

// Appel génération (non-streaming — pour vérification rapide)
const result = await fetch('http://localhost:11434/api/generate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    model: 'llama3:8b',
    prompt: 'Translate: こんにちは',
    stream: false
  })
});
```

**Note** : Pour le streaming SSE Ollama (traduction en temps réel), le backend Python (FastAPI + SSE) reste plus adapté. Ce plugin est idéal pour le health check et la liste des modèles.

**Permissions** :
```json
{
  "permissions": [
    "http:default",
    { "identifier": "http:allow-fetch", "allow": [{ "url": "http://localhost:11434/**" }] }
  ]
}
```

---

## Plugins P2 — Confort (recommandés post-MVP)

### 7. `tauri-plugin-window-state`

**Rôle** : Sauvegarde automatique de la taille/position de la fenêtre entre sessions.

**Intégration Rust** :
```rust
// src-tauri/src/main.rs
tauri::Builder::default()
    .plugin(tauri_plugin_window_state::Builder::default().build())
    .run(tauri::generate_context!())
```

**API frontend** :
```typescript
import { saveWindowState, restoreStateCurrent, StateFlags } from '@tauri-apps/plugin-window-state';

// Sauvegarder à la fermeture
await saveWindowState(StateFlags.ALL);

// Restaurer au démarrage
await restoreStateCurrent(StateFlags.ALL);

// Flags disponibles : SIZE, POSITION, MAXIMIZED, FULLSCREEN, ALL
```

**Pourquoi utile** : NeoGlot est une app de travail — l'utilisateur positionne sa fenêtre une fois, elle doit rester là.

---

### 8. `tauri-plugin-notification`

**Rôle** : Notifications système natives (Linux, Windows, macOS).

**Cas d'usage NeoGlot** :
- Notifier quand une traduction de batch est terminée (l'utilisateur peut avoir minimisé l'app)
- Alerte si Ollama crash pendant une traduction longue

**API frontend** :
```typescript
import { sendNotification, requestPermission, isPermissionGranted } from '@tauri-apps/plugin-notification';

// Vérifier/demander permission
let permission = await isPermissionGranted();
if (!permission) {
  const result = await requestPermission();
  permission = result === 'granted';
}

// Notifier fin de traduction
if (permission) {
  sendNotification({
    title: 'NeoGlot — Traduction terminée',
    body: `${count} textes traduits dans ${filename}`,
    icon: 'icons/icon.png'
  });
}
```

---

### 9. `tauri-plugin-process`

**Rôle** : Contrôle du cycle de vie de l'application.

**Cas d'usage NeoGlot** :
- `relaunch()` après mise à jour ou changement de config majeur
- `exit(0)` propre (s'assurer que le backend Python est bien tué avant)

**API frontend** :
```typescript
import { exit, relaunch } from '@tauri-apps/plugin-process';

// Quitter proprement (après avoir arrêté le backend Python)
await stopPythonBackend();
await exit(0);

// Relancer après mise à jour
await relaunch();
```

---

## Plugin P3 — Optionnel

### 10. `tauri-plugin-clipboard-manager`

**Rôle** : Lecture/écriture du presse-papiers.

**Cas d'usage NeoGlot** :
- Copier une traduction d'un clic
- Fonctionnalité "overlay translator" (comme SLR) — surveiller le clipboard pour traduire à la volée

**API frontend** :
```typescript
import { writeText, readText, writeHtml, clear } from '@tauri-apps/plugin-clipboard-manager';

// Copier une traduction
await writeText(translatedText);

// Lire pour l'overlay translator (optionnel)
const copiedText = await readText();
```

---

## Plugins à exclure (hors scope NeoGlot)

| Plugin | Raison d'exclusion |
|--------|-------------------|
| `tauri-plugin-updater` | Pas de distribution binaire prévue au MVP |
| `tauri-plugin-deep-link` | Pas de liens URL externes |
| `tauri-plugin-global-shortcut` | Pas de raccourcis globaux au MVP |
| `tauri-plugin-positioner` | Remplacé par window-state |
| `tauri-plugin-stronghold` | Pas de secrets sensibles à stocker |
| `tauri-plugin-biometric` | Non pertinent |
| `tauri-plugin-barcode-scanner` | Non pertinent |
| `tauri-plugin-nfc` | Non pertinent |

---

## Récapitulatif — `Cargo.toml` plugins Tauri

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
tauri-plugin-sql = { version = "2", features = ["sqlite"] }
tauri-plugin-store = "2"
tauri-plugin-http = "2"
tauri-plugin-window-state = "2"
tauri-plugin-notification = "2"
tauri-plugin-process = "2"
tauri-plugin-clipboard-manager = "2"   # optionnel
```

## Récapitulatif — `package.json` plugins Tauri

```json
{
  "dependencies": {
    "@tauri-apps/plugin-shell": "^2.0.0",
    "@tauri-apps/plugin-fs": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0",
    "@tauri-apps/plugin-sql": "^2.0.0",
    "@tauri-apps/plugin-store": "^2.0.0",
    "@tauri-apps/plugin-http": "^2.0.0",
    "@tauri-apps/plugin-window-state": "^2.0.0",
    "@tauri-apps/plugin-notification": "^2.0.0",
    "@tauri-apps/plugin-process": "^2.0.0",
    "@tauri-apps/plugin-clipboard-manager": "^2.0.0"
  }
}
```

## Récapitulatif — `src/main.rs` enregistrement plugins

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .run(tauri::generate_context!())
        .expect("Erreur démarrage NeoGlot");
}
```

---

## Architecture de communication résultante

```
Frontend React
    │
    ├─ tauri-plugin-dialog   → sélection dossier jeu
    ├─ tauri-plugin-fs       → lecture fichiers JSON Wolf RPG
    ├─ tauri-plugin-sql      → lecture projets/strings pour l'UI
    ├─ tauri-plugin-store    → préférences utilisateur
    ├─ tauri-plugin-http     → health check Ollama + liste modèles
    ├─ tauri-plugin-shell    → lancer/tuer Python backend + WolfTL + UberWolf
    │
    └─ invoke() (Rust commands)
           │
           ├─ parse_rpgm()     → rvpacker-txt-rs-lib
           ├─ decrypt_rgss()   → rpgmad-lib
           └─ write_rpgm()     → rvpacker-txt-rs-lib write mode

Python Backend (FastAPI :8000)
    │
    ├─ /projects             → CRUD SQLite (SQLAlchemy)
    ├─ /translation/start    → queue Ollama + batch processing
    ├─ /translation/stream   → SSE streaming résultats
    ├─ /engines/wolf/extract → subprocess WolfTL --create
    ├─ /engines/wolf/inject  → subprocess WolfTL --patch
    └─ /ollama/models        → proxy liste modèles
```
