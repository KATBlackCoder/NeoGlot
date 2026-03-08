# PROGRESS — NeoGlot

Suivi de l'avancement par tâche. Mis à jour à chaque complétion de tâche.

**Dernière mise à jour** : 2026-03-08 (refactoring qualité code — DRY, design UI, accessibilité, bugs)

---

## Vue d'ensemble

```
T01 ██████████ DONE
T02 ██████████ DONE
T03 ██████████ DONE
T04 ██████████ DONE
T05 ██████████ DONE
T06 ░░░░░░░░░░ TODO  (dépend T05, T03)
T07 ░░░░░░░░░░ TODO  (dépend T06)
T08 ░░░░░░░░░░ TODO  (dépend T04, T06)
T09 ░░░░░░░░░░ TODO  (dépend T04, T06)
T10 ░░░░░░░░░░ TODO  (dépend T05, T07)
```

**Progression globale : 5 / 10 tâches** (50%)

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
- Tous les composants shadcn-vue nécessaires ajoutés : `alert-dialog`, `badge`, `button`, `card`, `dialog`, `input`, `progress`, `resizable`, `scroll-area`, `select`, `separator`, `sheet`, `sidebar`, `skeleton`, `sonner`, `table`, `textarea`, `toggle`, `toggle-group`, `tooltip`

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
- `src-tauri/src/commands/mod.rs` — déclaration des modules (refactoré avec `engines/`)
- `src-tauri/src/commands/db_commands.rs` — CRUD complet : `list_projects`, `create_project`, `delete_project`, `store_strings`, `get_project_strings`, `get_project_progress`
- `src-tauri/src/commands/translate.rs` — `check_ollama()` et `list_ollama_models()` fonctionnels via `reqwest` blocking ; `start_translation` / `cancel_translation` stubs
- `src-tauri/src/commands/detect.rs` — `detect_engine()` implémenté (fichiers marqueurs, 6 moteurs)
- Stubs compilants : `engines/rpgmv/inject.rs`, `engines/rpgm_classic/extract.rs`, `engines/rpgm_classic/decrypt.rs`, `engines/wolf/extract.rs`, `engines/wolf/inject.rs`, `glossary.rs`
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
- `src/composables/useProjects.ts` — `useProjects()` (TanStack Query), `useDeleteProject()` (mutation + invalidation), `useOpenProject()` (Pinia + router), `useProjectProgress(id)` (polling 5s)
- `src/types/project.ts` — constantes de domaine centralisées : `ENGINE_LABELS`, `PROJECT_STATUS_LABELS`, `PROJECT_STATUS_VARIANTS`, `STRING_STATUS_LABELS`, `STRING_STATUS_VARIANTS`
- `src/lib/utils.ts` — `calcPercent(done, total)` utilitaire partagé
- `src/components/projects/ProjectCard.vue` — carte projet : nom, badge moteur, badge statut, barre de progression, boutons Ouvrir/Supprimer avec `AlertDialog` de confirmation
- `src/components/projects/ProjectList.vue` — skeletons de chargement, état vide, liste de `ProjectCard`
- `src/components/projects/NewProjectDialog.vue` — formulaire complet : sélection dossier via `tauri-plugin-dialog`, détection moteur automatique via `detect_engine`, select langues, contexte optionnel, création via `create_project` + navigation immédiate
- `src/views/ProjectsView.vue` — view fine : orchestre composable + composants, ouvre le dialog

**Fichiers créés :** `src/composables/useProjects.ts`, `src/components/projects/ProjectCard.vue`, `src/components/projects/ProjectList.vue`, `src/components/projects/NewProjectDialog.vue`

**Fichiers modifiés :** `src/views/ProjectsView.vue`

---

### ✅ T05 — Module RPG Maker MV/MZ : Extraction

**Statut** : DONE

**Ce qui a été fait :**
- `Cargo.toml` — ajout `rvpacker-txt-rs-lib = "11.1.2"` + `RUSTFLAGS="-C target-feature=+aes,+sse2"` requis (gxhash)
- `src-tauri/src/commands/engines/rpgmv/extract.rs` — `extract_rpgmv()` : détection MV/MZ, `Reader::read()` → fichiers `.txt`, parsing `<#>` + `\#`, stockage SQLite + mise à jour statut projet ; `extract_speakers()` : extraction noms via commentaires `<!-- NAME -->`
- `src-tauri/src/commands/db_commands.rs` — `create_project` retourne `Project` complet (fix T04) ; ajout `list_project_files` + struct `FileEntry`
- `src/composables/useTranslate.ts` — `useProjectFiles()`, `useProjectStrings()` (réactif via `MaybeRef`), `useExtractProject()`
- `src/components/translate/FileList.vue` — liste des fichiers avec progression %, tri maps en premier
- `src/components/translate/StringList.vue` — liste des chaînes avec filtre statut + recherche texte
- `src/views/TranslateView.vue` — rewrite complet : en-tête (bouton Extraire, progression), panneau FileList | StringList

**Fixes post-implémentation :**
- `create_project` : paramètres aplatis (suppression struct `NewProject` wrappé → erreur Tauri "missing key p")
- `useProjectStrings` : queryKey réactif via `computed()` + `MaybeRef` (fix clic fichier sans effet)
- `extract_rpgmv` : ajout `AppHandle` + émission événement `extraction-progress` par fichier (barre de chargement visible)

**Intégration formatter/validation (post-T05) :**
- `extract_rpgmv` applique désormais `RpgMakerTextValidator::validate_text()` (filtrage textes non-traduisibles) + `RpgMakerFormatter::prepare_for_translation()` (codes moteur → placeholders AI) avant insertion en DB
- `source_text` en DB contient le texte formaté (avec placeholders `[COLOR_1]`, `[NAME_2]`, etc.)
- `raw_text` en DB contient le texte original (avec codes moteur `\C[1]`, `\N[2]`, etc.)
- Après traduction AI, `restore_after_translation()` restaure les codes originaux
- Ré-extraction nettoie automatiquement les anciennes données du projet avant réinsertion

**UX extraction (post-T05) :**
- `AlertDialog` modal bloquant pendant l'extraction (barre de progression, nom du fichier en cours, empêche la navigation)
- `await nextTick()` avant l'extraction pour garantir le rendu du dialog (Vue scheduling fix)
- `Toaster` (vue-sonner) monté dans `App.vue` avec `theme="dark" rich-colors position="bottom-right"` — toast succès/erreur après extraction
- `vue-sonner/style.css` importé dans `main.ts` (vue-sonner v2 n'injecte plus les styles automatiquement)
- Migration DB automatique : ajout colonne `raw_text` sur bases existantes via `ALTER TABLE` au démarrage

**Filtrage validation renforcé (post-T05) :**
- `ContentValidator` : filtrage code JS (`();`, `);`, `this.`, `self.`, appels de méthodes `mot.mot(`)
- `RpgMakerTextValidator` : filtrage commandes de script ASCII (type `PSS start`, `MOT argument`)

**Fichiers créés :** `src/composables/useTranslate.ts`, `src/components/translate/FileList.vue`, `src/components/translate/StringList.vue`, `src-tauri/src/commands/engines/rpgmv/extract.rs`, `engines/formatter.rs`, `engines/validation.rs`, `rpgmv/formatter.rs`, `rpgmv/validation.rs`, `wolf/formatter.rs`, `wolf/validation.rs`, `rpgm_classic/formatter.rs`, `rpgm_classic/validation.rs`

**Fichiers modifiés :** `src-tauri/src/commands/db_commands.rs`, `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src/views/TranslateView.vue`, `src/composables/useTranslate.ts`, `src-tauri/src/db.rs`, `src/App.vue`

---

### ⏳ T06 — Moteur de Traduction (reqwest + Tauri Channel)

**Statut** : TODO — _dépend T05, T03_

**À faire :**
- Rust : `commands/translate.rs` — pipeline complet (déduplication, batches, Ollama, cache)
- Rust : `check_ollama()`, `list_ollama_models()`, `start_translation()`, `cancel_translation()`
- Vue : `useTranslation.ts` — Channel → `useTranslationStore.updateProgress()`
- Attention : utiliser `reqwest` (blocking), pas `ollama-rs` (incompatible CPU)
- Après réponse Ollama : appliquer `restore_after_translation()` pour restaurer les codes moteur avant stockage

**Prérequis déjà implémentés :**
- ✅ `EngineFormatter` trait + `RpgMakerFormatter` / `WolfRpgFormatter` — placeholders intégrés dans l'extraction
- ✅ `ContentValidator` + validators engine-specific — filtrage des textes non-traduisibles
- ✅ `source_text` en DB contient déjà le texte formaté (prêt pour Ollama)
- ✅ `raw_text` en DB contient le texte original (pour restauration post-traduction)
- Intégrer `PromptType` (Character, Dialogue, Item, Skill, System…) pour adapter les prompts Ollama par type de texte

---

### ⏳ T07 — Module RPG Maker MV/MZ : Réinjection

**Statut** : TODO — _dépend T06_

**À faire :**
- Rust : `commands/engines/rpgmv/inject.rs` → `write_rpgmv()` — copie sources + `rvpacker_txt_rs_lib::write_dir()`
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
- Rust : `commands/engines/wolf/extract.rs` → `extract_wolf()` + `commands/engines/wolf/inject.rs` → `inject_wolf()` via `std::process::Command`
- Linux : `wine <exe>` avec `#[cfg(target_os)]`

---

### ⏳ T10 — Module RPG Maker XP/VX/VXAce

**Statut** : TODO — _dépend T05, T07_

**À faire :**
- Ajouter `marshal-rs`, `rpgmad-lib` dans `Cargo.toml`
- Rust : `commands/engines/rpgm_classic/decrypt.rs` → `decrypt_rgss()`
- Rust : `commands/engines/rpgm_classic/extract.rs` → `extract_rpgm_classic()` (même API que MV/MZ via `rvpacker-txt-rs-lib`)

---

## Dépendances entre tâches

```
T01 ──┬──▶ T02 ──┐
      └──▶ T03 ──┴──▶ T04 ──┬──▶ T05 ──▶ T06 ──┬──▶ T07 ──▶ T10
                              │                   ├──▶ T08
                              └───────────────────┘
                                                  └──▶ T09
```
