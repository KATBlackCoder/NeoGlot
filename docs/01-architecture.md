# Architecture NeoGlot

## Vue d'ensemble

```
┌─────────────────────────────────────────────────────────────┐
│                    Fenêtre Tauri (WebView)                   │
│                                                             │
│   Vue 3 + TypeScript + shadcn-vue + Tailwind                │
│                                                             │
│   Pages : Home | Projects | Translate | Glossary | Settings │
└──────────────────────────┬──────────────────────────────────┘
                           │
                     invoke() IPC
                     (invoke() IPC — traduction batch, pas temps réel)
                           │
          ┌────────────────▼────────────────────────────────┐
          │              Rust / Tauri                        │
          │                                                  │
          │  commands/                                       │
          │    db_commands.rs   ← CRUD projets/strings       │
          │    detect.rs        ← détection moteur jeu       │
          │    parse.rs         ← extraction RPG Maker MV/MZ │
          │    write.rs         ← réinjection RPG Maker      │
          │    decrypt.rs       ← déchiffrement .rgss        │
          │    translate.rs     ← pipeline Ollama + cache    │
          │    glossary.rs      ← CRUD glossaire             │
          │    wolf.rs          ← Wolf RPG (subprocess)      │
          │                                                  │
          │  plugins:                                        │
          │    tauri-plugin-shell  (WolfTL, UberWolf)        │
          │    tauri-plugin-fs                               │
          │    tauri-plugin-dialog                           │
          │    tauri-plugin-sql    (lectures frontend)       │
          │    tauri-plugin-store  (préférences)             │
          │    tauri-plugin-log                              │
          │                                                  │
          │  crates:                                         │
          │    rusqlite          ← SQLite natif              │
          │    reqwest           ← HTTP batch Ollama :11434   │
          │    rvpacker-txt-rs-lib ← RPG Maker MV/MZ/XP/VX  │
          │    rpgmad-lib        ← déchiffrement .rgss       │
          │    regex             ← protection placeholders   │
          │    tokio             ← async runtime             │
          └────────────────┬────────────────────────────────┘
                           │
               ┌───────────▼────────┐       ┌───────────────┐
               │   SQLite DB        │       │ Ollama :11434  │
               │   neoglot.db       │       │ (local AI)     │
               │   (rusqlite)       │       └───────────────┘
               └────────────────────┘
```

## Couches

### Frontend — Vue 3 + Tauri

- **Framework** : Vue 3 + TypeScript + Vite — Composition API, `<script setup lang="ts">`, SFCs `.vue`
- **UI** : shadcn-vue (Reka UI) + Tailwind CSS v4, thème sombre
- **Navigation** : Vue Router v4 (`createWebHashHistory` — requis Tauri, pas de serveur)
- **IPC** : `invoke()` pour toutes les opérations (pas d'HTTP vers un backend séparé)
- **Progression** : `new Channel<TranslationProgress>()` depuis `@tauri-apps/api/core` pour suivre l'avancement de la traduction batch (texte par texte, pas streaming token)

#### Gestion d'état — 3 couches distinctes

| Couche | Outil | Dossier | Rôle |
|--------|-------|---------|------|
| État client partagé | **Pinia** | `src/stores/` | Projet ouvert, job de traduction en cours — accessible partout sans prop-drilling |
| État serveur | **TanStack Vue Query** | `src/composables/use*.ts` | Données de `invoke()` (projets, strings, glossaire) avec cache + invalidation automatique |
| Settings persistés | **`useStore.ts`** + `tauri-plugin-store` | `src/composables/` | Préférences utilisateur (URL Ollama, modèle, tokens/batch) |

**Stores Pinia :**
- `useProjectStore` — projet actuellement ouvert (`currentProject: Project | null`) — évite de passer l'id/objet projet entre TranslateView, GlossaryView, AppSidebar
- `useTranslationStore` — état du job en cours (`isRunning`, `progress`, `percentage`) — mis à jour par les events du Channel Tauri

**Composables Vue Query :**
- `useOllamaStatus()` / `useOllamaModels()` — dans `src/composables/useOllama.ts`
- `useProjects()` / `useDeleteProject()` / `useProjectProgress(id)` — dans `src/composables/useProjects.ts`
- `useTranslation(projectId)` — dans `src/composables/useTranslation.ts` (T06)

### Backend Rust — src-tauri/

Responsabilités : **tout** — parsing, SQLite, Ollama, Wolf RPG, glossaire, cache de traduction.

| Fichier | Rôle | Crates |
|---------|------|--------|
| `commands/db_commands.rs` | CRUD projets, fichiers, strings | `rusqlite` |
| `commands/detect.rs` | Détection moteur de jeu | `walkdir` |
| `commands/parse.rs` | Extraction textes RPG Maker MV/MZ/XP/VX | `rvpacker-txt-rs-lib` |
| `commands/write.rs` | Réinjection traductions RPG Maker | `rvpacker-txt-rs-lib` |
| `commands/decrypt.rs` | Déchiffrement .rgss (RPG Maker XP/VX) | `rpgmad-lib` |
| `commands/translate.rs` | Pipeline traduction Ollama batch + cache + Channel progression | `reqwest`, `rusqlite`, `regex` |
| `commands/glossary.rs` | CRUD glossaire + QC post-traduction | `rusqlite` |
| `commands/wolf.rs` | Extraction/réinjection Wolf RPG | `std::process::Command` |
| `db.rs` | Init SQLite, schéma, WAL mode | `rusqlite` |

### AppState (partagé entre commandes)

```rust
pub struct AppState {
    pub db_path: PathBuf,
    pub translation_running: Mutex<bool>,
}
```

Injecté via `tauri::State<'_, AppState>` dans chaque commande qui en a besoin.

### Base de données — SQLite

Fichier unique : `~/.local/share/neoglot/neoglot.db` (Linux) / `%APPDATA%\neoglot\neoglot.db` (Windows).

Géré exclusivement par `rusqlite` côté Rust — toutes les lectures/écritures passent par `invoke()`. Pas de `tauri-plugin-sql`.

Voir `docs/02-database-schema.md` pour le schéma complet.

## Démarrage de l'application

```
Tauri démarre
  │
  ├─ [Rust] AppState::new() → open_db() → init_schema() (WAL + FK)
  │
  ├─ tauri_plugin_window_state::restore()  ← restaure taille/position fenêtre
  │
  ├─ [Frontend] invoke('check_ollama') → bool
  │     └─ false → afficher modal "Ollama non détecté" avec lien install
  │     └─ true  → continuer
  │
  └─ Afficher l'écran d'accueil (liste des projets)
```

## Arrêt de l'application

```
Utilisateur ferme la fenêtre
  │
  ├─ [Rust] on_window_event → CloseRequested
  ├─ saveWindowState(StateFlags.ALL)
  └─ exit(0)
```

## Communication Frontend ↔ Rust (IPC)

```typescript
// Toutes les opérations passent par invoke()

// Projets
const projects = await invoke<Project[]>('list_projects');
const project = await invoke<Project>('create_project', { name, gamePath, workPath, engine, ... });
await invoke('delete_project', { projectId: 1 });

// Extraction RPG Maker MV/MZ
const strings = await invoke<ExtractedString[]>('extract_rpgmv', { gamePath, workPath });
await invoke('store_strings', { projectId: 1, strings });

// Détection moteur
const engine = await invoke<string>('detect_engine', { gamePath: '/path/to/game' });

// Traduction batch — Channel pour la progression (1 event par texte traduit, pas streaming token)
import { Channel } from '@tauri-apps/api/core';
const channel = new Channel<TranslationProgress>();
channel.onmessage = (event) => { setProgress(event); };  // done/total
await invoke('start_translation', { projectId: 1, model: 'llama3:8b', onProgress: channel });
await invoke('cancel_translation');

// Glossaire
const terms = await invoke<GlossaryEntry[]>('list_glossary', { projectId: 1 });
await invoke('add_glossary_term', { projectId: 1, term, translation, note, matchMode });

// Wolf RPG
await invoke('extract_wolf', { projectId: 1 });
await invoke('inject_wolf', { projectId: 1 });
```

## Flux de traduction complet

Voir `docs/03-translation-workflow.md`.
