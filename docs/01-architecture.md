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
          │  commands/                                            │
          │    db_commands.rs   ← CRUD projets/strings          │
          │    detect.rs        ← détection moteur jeu          │
          │    translate.rs     ← pipeline Ollama + cache       │
          │    glossary.rs      ← CRUD glossaire                │
          │    engines/                                          │
          │      formatter.rs   ← trait EngineFormatter + UniversalFormatter │
          │      validation.rs  ← ContentValidator (filtrage universel)      │
          │      rpgmv/         ← RPG Maker MV/MZ               │
          │        extract.rs   ← extraction (rvpacker)         │
          │        inject.rs    ← réinjection (rvpacker)        │
          │        formatter.rs ← RpgMakerFormatter (placeholders)│
          │        validation.rs← RpgMakerTextValidator          │
          │      rpgm_classic/  ← RPG Maker XP/VX/VXAce        │
          │        extract.rs   ← extraction (marshal-rs)       │
          │        decrypt.rs   ← déchiffrement .rgss*          │
          │        formatter.rs ← réutilise RpgMakerFormatter   │
          │        validation.rs← réutilise RpgMakerTextValidator│
          │      wolf/          ← Wolf RPG Editor               │
          │        extract.rs   ← extraction (WolfTL)           │
          │        inject.rs    ← réinjection (WolfTL)          │
          │        formatter.rs ← WolfRpgFormatter (placeholders)│
          │        validation.rs← WolfRpgTextValidator           │
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
- **Progression extraction** : `listen('extraction-progress', cb)` depuis `@tauri-apps/api/event` — événement émis par Rust après chaque fichier `.txt` traité (`{ current, total, file }`) ; `AlertDialog` modal bloquant avec barre de progression, `await nextTick()` pour garantir le rendu avant le lancement de l'extraction
- **Feedback utilisateur** : `Toaster` (vue-sonner) monté dans `App.vue` avec `theme="dark" rich-colors` — toast succès/erreur après extraction ; CSS de positionnement importé via `vue-sonner/style.css` dans `main.ts` (requis en v2)
- **Progression traduction** : `new Channel<TranslationProgress>()` depuis `@tauri-apps/api/core` pour suivre l'avancement de la traduction batch (texte par texte, pas streaming token)

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
| `commands/engines/formatter.rs` | Trait `EngineFormatter` + `UniversalFormatter` (placeholders communs) | `regex`, `once_cell` |
| `commands/engines/validation.rs` | `ContentValidator` (filtrage universel textes non-traduisibles) | — |
| `commands/engines/rpgmv/extract.rs` | Extraction textes RPG Maker MV/MZ + validation + formatage | `rvpacker-txt-rs-lib`, `sha2` |
| `commands/engines/rpgmv/inject.rs` | Réinjection traductions RPG Maker MV/MZ | `rvpacker-txt-rs-lib` |
| `commands/engines/rpgmv/formatter.rs` | `RpgMakerFormatter` — codes `\C`, `\N`, `\V`, `\I` → placeholders | `regex`, `once_cell` |
| `commands/engines/rpgmv/validation.rs` | `RpgMakerTextValidator` — filtrage spécifique RPG Maker | — |
| `commands/engines/rpgm_classic/extract.rs` | Extraction RPG Maker XP/VX/VXAce (T10) | `marshal-rs` |
| `commands/engines/rpgm_classic/decrypt.rs` | Déchiffrement .rgss* | `rpgmad-lib` |
| `commands/engines/rpgm_classic/formatter.rs` | Réexporte `RpgMakerFormatter` (mêmes codes moteur) | — |
| `commands/engines/rpgm_classic/validation.rs` | Réexporte `RpgMakerTextValidator` | — |
| `commands/engines/wolf/extract.rs` | Extraction Wolf RPG via WolfTL (T09) | `std::process::Command` |
| `commands/engines/wolf/inject.rs` | Réinjection Wolf RPG via WolfTL (T09) | `std::process::Command` |
| `commands/engines/wolf/formatter.rs` | `WolfRpgFormatter` — codes `\E`, `\i`, `\cself`, `@n` → placeholders | `regex`, `once_cell` |
| `commands/engines/wolf/validation.rs` | `WolfRpgTextValidator` — filtrage spécifique Wolf RPG | `regex`, `once_cell` |
| `commands/translate.rs` | Pipeline traduction Ollama batch + `PromptType` + `EngineFormatter` + Channel progression | `reqwest`, `rusqlite`, `regex` |
| `commands/glossary.rs` | CRUD glossaire + QC post-traduction | `rusqlite` |
| `db.rs` | Init SQLite, schéma, WAL mode, migrations auto (`raw_text`) | `rusqlite` |

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
  ├─ [Rust] AppState::new() → open_db() → init_schema() (WAL + FK) → run_migrations()
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
