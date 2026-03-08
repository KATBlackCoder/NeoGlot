# PROGRESS — NeoGlot

Suivi de l'avancement par tâche. Mis à jour à chaque complétion de tâche.

**Dernière mise à jour** : 2026-03-08 (T04)

---

## Vue d'ensemble

```
T01 ██████████ DONE
T02 ██████████ DONE
T03 ██████████ DONE
T04 ██████████ DONE
T05 ░░░░░░░░░░ TODO  (dépend T04)
T06 ░░░░░░░░░░ TODO  (dépend T05, T03)
T07 ░░░░░░░░░░ TODO  (dépend T06)
T08 ░░░░░░░░░░ TODO  (dépend T04, T06)
T09 ░░░░░░░░░░ TODO  (dépend T04, T06)
T10 ░░░░░░░░░░ TODO  (dépend T05, T07)
```

**Progression globale : 4 / 10 tâches** (40%)

---

## Détail par tâche

### ✅ T01 — Installer plugins Tauri + shadcn-vue + Tailwind

**Statut** : DONE

**Ce qui a été fait :**
- Plugins Tauri installés côté Rust : `tauri-plugin-shell`, `fs`, `dialog`, `store`, `log`, `window-state`, `notification`, `process`, `clipboard-manager`
- Crates Rust ajoutées dans `Cargo.toml` : `reqwest` (0.13, blocking+json), `rusqlite` (0.38, bundled), `tokio` (1, full), `regex`, `once_cell`, `serde/serde_json`, `sha2`, `walkdir`
- shadcn-vue initialisé (style New York, base color Zinc, CSS variables)
- Tailwind CSS v4 + `@tailwindcss/vite` configuré
- Vue Router 4 + TanStack Vue Query installés
- Tous les composants shadcn-vue nécessaires ajoutés : `alert-dialog`, `badge`, `button`, `card`, `dialog`, `input`, `progress`, `resizable`, `scroll-area`, `select`, `separator`, `sheet`, `sidebar`, `skeleton`, `sonner`, `table`, `textarea`, `tooltip`

**Fichiers modifiés :** `package.json`, `pnpm-lock.yaml`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `index.html`, `src/style.css`, `src/lib/utils.ts`, `src/components/ui/**`

---

### ✅ T02 — App Shell, Routing et Layout

**Statut** : DONE

**Ce qui a été fait :**
- `src/router/index.ts` — `createWebHashHistory` + 5 routes (/, /projects, /projects/:id/translate, /projects/:id/glossary, /settings)
- `src/types/project.ts` — interfaces `Project`, `NewProject`, `ProjectProgress`
- `src/types/glossary.ts` — interfaces `GlossaryEntry`, `NewGlossaryEntry`
- `src/composables/useOllama.ts` — `useOllamaStatus()` (poll 30s) + `useOllamaModels()`
- `src/composables/useStore.ts` — `useStore()` avec `LazyStore` (settings persistés)
- `src/stores/projectStore.ts` — `useProjectStore` Pinia (projet ouvert)
- `src/stores/translationStore.ts` — `useTranslationStore` Pinia (job de traduction)
- `src/stores/index.ts` — barrel export
- `src/components/OllamaStatus.vue` — badge vert/rouge/gris
- `src/components/AppSidebar.vue` — navigation avec icônes Lucide
- `src/components/AppShell.vue` — `SidebarProvider` + `SidebarTrigger` + `RouterView`
- `src/views/HomeView.vue` — `AlertDialog` bloquant si Ollama absent + cards navigation
- `src/views/ProjectsView.vue` — scaffold liste projets (prête pour T04)
- `src/views/TranslateView.vue` — scaffold `ResizablePanelGroup` (prête pour T05/T06)
- `src/views/GlossaryView.vue` — scaffold Table (prête pour T08)
- `src/views/SettingsView.vue` — config Ollama URL + modèle + tokens/batch
- `src/main.ts` — Pinia + VueQueryPlugin + router branchés
- `src/App.vue` — racine minimaliste `<RouterView />`
- **Pinia** installé et configuré (3.0.4)

**Décisions prises :**
- Pinia pour l'état client partagé (`projectStore`, `translationStore`)
- TanStack Vue Query pour le state serveur (données `invoke()`)
- `useStore.ts` + `LazyStore` pour les settings persistés

---

### ✅ T03 — Scaffold Rust Commands (DB + State)

**Statut** : DONE

**Ce qui a été fait :**
- `src-tauri/src/db.rs` — `get_db_path()` (Linux + Windows), `open()` (WAL + FK), `init_schema()` (6 tables + 3 index)
- `src-tauri/src/lib.rs` — `AppState { db_path, translation_running: Mutex<bool> }` + tous les plugins branchés + `generate_handler![]` complet
- `src-tauri/src/commands/mod.rs` — déclaration des 8 modules
- `src-tauri/src/commands/db_commands.rs` — CRUD complet : `list_projects`, `create_project`, `delete_project`, `store_strings`, `get_project_strings`, `get_project_progress`
- `src-tauri/src/commands/translate.rs` — `check_ollama()` et `list_ollama_models()` fonctionnels via `reqwest` blocking ; `start_translation` / `cancel_translation` stubs
- `src-tauri/src/commands/detect.rs` — `detect_engine()` implémenté (fichiers marqueurs, 6 moteurs)
- Stubs compilants : `parse.rs`, `write.rs`, `decrypt.rs`, `glossary.rs`, `wolf.rs`
- `dirs = "5"` ajouté dans `Cargo.toml`
- **`cargo check` : 0 erreur, 1 warning inoffensif** (champs stub non utilisés)

**Commandes disponibles dès T03 :**
- `check_ollama` → `bool` — utilisé par `useOllamaStatus()` (HomeView)
- `list_ollama_models` → `Vec<String>` — utilisé par SettingsView
- `list_projects` / `create_project` / `delete_project` / `get_project_progress` / `store_strings` / `get_project_strings`
- `detect_engine` — prête pour T04

---

### ✅ T04 — Module Projets (CRUD + détection moteur)

**Statut** : DONE

**Ce qui a été fait :**
- `src/composables/useProjects.ts` — `useProjects()` (TanStack Query), `useDeleteProject()` (mutation + invalidation), `useOpenProject()` (Pinia + router), `useProjectProgress(id)` (polling 5s) + `ENGINE_LABELS`
- `src/components/projects/ProjectCard.vue` — carte projet : nom, badge moteur, badge statut, barre de progression, boutons Ouvrir/Supprimer avec `AlertDialog` de confirmation
- `src/components/projects/ProjectList.vue` — skeletons de chargement, état vide, liste de `ProjectCard`
- `src/components/projects/NewProjectDialog.vue` — formulaire complet : sélection dossier via `tauri-plugin-dialog`, détection moteur automatique via `detect_engine`, select langues, contexte optionnel, création via `create_project` + navigation immédiate
- `src/views/ProjectsView.vue` — view fine : orchestre composable + composants, ouvre le dialog

**Fichiers créés :** `src/composables/useProjects.ts`, `src/components/projects/ProjectCard.vue`, `src/components/projects/ProjectList.vue`, `src/components/projects/NewProjectDialog.vue`

**Fichiers modifiés :** `src/views/ProjectsView.vue`

---

### ⏳ T05 — Module RPG Maker MV/MZ : Extraction

**Statut** : TODO — _dépend T04_

**À faire :**
- Rust : `commands/parse.rs` → `extract_rpgmv()` via `rvpacker-txt-rs-lib` + hash SHA256
- Rust : `extract_speakers()` — extraction noms personnages (event 101)
- Vérifier l'API exacte `rvpacker-txt-rs-lib` v11.1.2 via `cargo doc` avant implémentation
- UI : bouton "Extraire" dans `TranslateView.vue`

---

### ⏳ T06 — Moteur de Traduction (reqwest + Tauri Channel)

**Statut** : TODO — _dépend T05, T03_

**À faire :**
- Rust : `commands/translate.rs` — pipeline complet (déduplication, placeholders, batches, Ollama, cache)
- Rust : `check_ollama()`, `list_ollama_models()`, `start_translation()`, `cancel_translation()`
- Vue : `useTranslation.ts` — Channel → `useTranslationStore.updateProgress()`
- Attention : utiliser `reqwest` (blocking), pas `ollama-rs` (incompatible CPU)

---

### ⏳ T07 — Module RPG Maker MV/MZ : Réinjection

**Statut** : TODO — _dépend T06_

**À faire :**
- Rust : `commands/write.rs` → `write_rpgmv()` — copie sources + `rvpacker_txt_rs_lib::write_dir()`
- UI : bouton "Exporter" dans `TranslateView.vue`

---

### ⏳ T08 — Module Glossaire

**Statut** : TODO — _dépend T04, T06_

**À faire :**
- Rust : `commands/glossary.rs` — CRUD complet + `import_speakers_to_glossary()` + `check_glossary_compliance()`
- Vue : `GlossaryView.vue` complète + `useGlossary.ts`
- Injection du glossaire dans le prompt Ollama (via `build_prompt()` de T06)

---

### ⏳ T09 — Module Wolf RPG (UberWolf + WolfTL)

**Statut** : TODO — _dépend T04, T06_

**À faire :**
- Binaires `UberWolfCli.exe` + `WolfTL.exe` déjà placés dans `src-tauri/resources/` ✅
- Rust : `commands/wolf.rs` → `extract_wolf()` + `inject_wolf()` via `std::process::Command`
- Linux : `wine <exe>` avec `#[cfg(target_os)]`

---

### ⏳ T10 — Module RPG Maker XP/VX/VXAce

**Statut** : TODO — _dépend T05, T07_

**À faire :**
- Ajouter `marshal-rs`, `rpgmad-lib` dans `Cargo.toml`
- Rust : `commands/decrypt.rs` → `decrypt_rgss()`
- Rust : `commands/parse.rs` → `extract_rpgm_classic()` (même API que MV/MZ via `rvpacker-txt-rs-lib`)

---

## Dépendances entre tâches

```
T01 ──┬──▶ T02 ──┐
      └──▶ T03 ──┴──▶ T04 ──┬──▶ T05 ──▶ T06 ──┬──▶ T07 ──▶ T10
                              │                   ├──▶ T08
                              └───────────────────┘
                                                  └──▶ T09
```
