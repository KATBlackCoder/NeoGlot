# CHANGELOG — NeoGlot

Toutes les modifications notables du projet sont documentées ici.
Format basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/).

---

## [Unreleased]

_Aucun changement en attente de release._

---

## [0.3.0] — 2026-03-08

### Added — T03 : Scaffold Rust Commands (DB + State)

**Backend Rust**
- `src-tauri/src/db.rs` — `get_db_path()`, `open()` (WAL + foreign keys), `init_schema()` : 6 tables (`projects`, `files`, `strings`, `translation_cache`, `glossary`, `translation_jobs`) + 3 index
- `src-tauri/src/lib.rs` — `AppState { db_path: PathBuf, translation_running: Mutex<bool> }` + initialisation DB au démarrage + tous les plugins branchés + `generate_handler![]` complet
- `src-tauri/src/commands/mod.rs` — déclaration des 8 modules commandes
- `src-tauri/src/commands/db_commands.rs` — CRUD complet : `list_projects`, `create_project`, `delete_project`, `store_strings`, `get_project_strings`, `get_project_progress`
- `src-tauri/src/commands/translate.rs` — `check_ollama()` et `list_ollama_models()` fonctionnels (reqwest blocking) ; stubs `start_translation` / `cancel_translation`
- `src-tauri/src/commands/detect.rs` — `detect_engine()` complet (6 moteurs détectés par fichiers marqueurs)
- `src-tauri/src/commands/parse.rs` — stub (T05)
- `src-tauri/src/commands/write.rs` — stub (T07)
- `src-tauri/src/commands/decrypt.rs` — stub (T10)
- `src-tauri/src/commands/glossary.rs` — stub avec types (T08)
- `src-tauri/src/commands/wolf.rs` — stub (T09)

### Changed

- `src-tauri/Cargo.toml` — ajout `dirs = "5"` pour résolution du chemin DB cross-platform
- `tasks/T03-rust-commands.md` — statut DONE

---

## [0.2.0] — 2026-03-08

### Added — T02 : App Shell, Routing et Layout

**Frontend Vue 3**
- `src/router/index.ts` — Vue Router 4 avec `createWebHashHistory` (obligatoire Tauri), 5 routes
- `src/types/project.ts` — interfaces TypeScript `Project`, `NewProject`, `ProjectProgress`
- `src/types/glossary.ts` — interfaces TypeScript `GlossaryEntry`, `NewGlossaryEntry`
- `src/composables/useOllama.ts` — `useOllamaStatus()` (polling 30s) + `useOllamaModels()`
- `src/composables/useStore.ts` — `useStore()` avec `LazyStore` (tauri-plugin-store)
- `src/stores/projectStore.ts` — store Pinia `useProjectStore` (projet actuellement ouvert)
- `src/stores/translationStore.ts` — store Pinia `useTranslationStore` (job de traduction en cours)
- `src/stores/index.ts` — barrel export des stores
- `src/components/OllamaStatus.vue` — badge de statut Ollama (vert/rouge/gris)
- `src/components/AppSidebar.vue` — sidebar de navigation avec icônes Lucide
- `src/components/AppShell.vue` — layout principal (`SidebarProvider` + `RouterView`)
- `src/views/HomeView.vue` — accueil + `AlertDialog` bloquant si Ollama absent
- `src/views/ProjectsView.vue` — scaffold liste de projets (skeleton + état vide)
- `src/views/TranslateView.vue` — scaffold `ResizablePanelGroup` fichiers | strings
- `src/views/GlossaryView.vue` — scaffold Table shadcn-vue
- `src/views/SettingsView.vue` — configuration Ollama URL + modèle + tokens/batch avec persistance

**Dependencies**
- `pinia` 3.0.4 ajouté (état client partagé)

### Changed

- `src/main.ts` — ajout `createPinia()` + `VueQueryPlugin` + `router`
- `src/App.vue` — remplacé le template de démo Tauri par `<RouterView />` minimal
- `CLAUDE.md` — tech stack mise à jour avec Pinia + structure `src/stores/`
- `docs/01-architecture.md` — section Frontend complétée avec le tableau des 3 couches d'état
- `docs/05-ui-structure.md` — structure de fichiers mise à jour avec `stores/` + tableau responsabilités
- `tasks/T02-app-shell-routing.md` — statut DONE, structure dossiers mise à jour
- `tasks/T04-projects-module.md` — `useOpenProject()` intègre `useProjectStore`
- `tasks/T06-translation-engine.md` — `useTranslation.ts` délègue à `useTranslationStore`

---

## [0.1.0] — 2026-03-08

### Added — T01 : Installation des dépendances

**Plugins Tauri v2 (Rust)**
- `tauri-plugin-shell` — exécution de sous-processus (WolfTL, UberWolf)
- `tauri-plugin-fs` — accès fichiers jeu
- `tauri-plugin-dialog` — sélecteur de dossier natif
- `tauri-plugin-store` — préférences utilisateur persistées
- `tauri-plugin-log` — logs applicatifs
- `tauri-plugin-window-state` — sauvegarde taille/position fenêtre
- `tauri-plugin-notification` — notification de fin de traduction
- `tauri-plugin-process` — arrêt propre de l'application
- `tauri-plugin-clipboard-manager` — copie de traduction

**Crates Rust**
- `reqwest` (0.13, blocking+json) — appels HTTP Ollama (`POST /api/generate`)
- `rusqlite` (0.38, bundled) — SQLite natif, remplace `tauri-plugin-sql`
- `tokio` (1, full) — runtime async pour les commandes Tauri
- `regex` (1) + `once_cell` — protection des placeholders RPG Maker
- `serde` + `serde_json` — sérialisation IPC
- `sha2` (0.10) — hash SHA256 des strings (déduplication + cache)
- `walkdir` (2) — parcours de dossiers (détection moteur jeu)

**Frontend**
- shadcn-vue initialisé — style New York, base color Zinc, CSS variables, Tailwind CSS v4
- Composants shadcn-vue ajoutés : `alert-dialog`, `badge`, `button`, `card`, `dialog`, `input`, `progress`, `resizable`, `scroll-area`, `select`, `separator`, `sheet`, `sidebar`, `skeleton`, `sonner`, `table`, `textarea`, `tooltip`
- `vue-router` 4 installé
- `@tanstack/vue-query` installé
- `lucide-vue-next` installé

### Changed

- `index.html` — `class="dark"` sur `<html>` (thème sombre par défaut)
- `src/style.css` — variables CSS shadcn-vue (Zinc) + `@custom-variant dark`
- `src-tauri/Cargo.toml` — toutes les dépendances Rust configurées
- `src-tauri/tauri.conf.json` — configuration Tauri mise à jour
- `src-tauri/capabilities/default.json` — permissions des plugins

### Décisions d'architecture

- **Pas de `tauri-plugin-sql`** — SQLite géré exclusivement via `rusqlite` côté Rust
- **Pas de `tauri-plugin-http`** — appels Ollama via `reqwest` blocking côté Rust
- **Pas de `ollama-rs`** — incompatible CPU AES+SSE2, remplacé par `reqwest`
- **Pas de Python** — toute la logique métier est en Rust

---

## [0.0.1] — 2026-03-07

### Added — Initialisation du projet

- Projet Tauri v2 + Vue 3 + TypeScript + Vite scaffoldé
- Documentation d'architecture initiale (`docs/`)
- Backlog de tâches (`tasks/T01` → `T10`)
- Synthèse des projets de référence (`reviews/`)
