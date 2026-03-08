# CHANGELOG — NeoGlot

Toutes les modifications notables du projet sont documentées ici.
Format basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/).

---

## [Unreleased]

### Refactored — Qualité code + design UI (2026-03-08)

**DRY — Centralisation des constantes de domaine dans `types/project.ts`**

- `ENGINE_LABELS` déplacé de `useProjects.ts` (composable) vers `types/project.ts` (domaine) ; ré-exporté depuis `useProjects` pour rétrocompatibilité
- `PROJECT_STATUS_LABELS` + `PROJECT_STATUS_VARIANTS` — nouvelles constantes centralisées (étaient dupliquées dans `ProjectCard.vue`)
- `STRING_STATUS_LABELS` + `STRING_STATUS_VARIANTS` — nouvelles constantes centralisées (étaient locales dans `StringList.vue`)
- `ProjectProgress` interface nettoyée : suppression des champs `project_id` et `percentage` qui ne correspondaient pas au retour backend

**DRY — Utilitaire `calcPercent()` dans `src/lib/utils.ts`**

- `calcPercent(done, total): number` — remplace 5 implémentations identiques de `Math.round((done/total)*100)` présentes dans `TranslateView.vue`, `ProjectCard.vue`, `FileList.vue`, `translationStore.ts`

**Bugs critiques corrigés**

- `HomeView.vue` — suppression du bouton de debug `Give me a toast` (anglais, en production)
- `HomeView.vue` — carte "Nouveau projet" ouvre maintenant directement `NewProjectDialog` au lieu de rediriger vers `/projects` (même destination que "Mes projets")
- `GlossaryView.vue` — typage `entries: never[]` → `entries: GlossaryEntry[]` + suppression de 5 casts `as any` dans le template
- `NewProjectDialog.vue` — deux `catch (err) as string` remplacés par `String(err)` (évite `[object Object]` si Tauri retourne un objet)

**Accessibilité**

- `NewProjectDialog.vue` — tous les `<label>` ont désormais un attribut `for` associé à leur `<input>` via `id` (champs : nom, dossier, langue source, langue cible, contexte)

**Design UI**

- `OllamaStatus.vue` — `bg-green-600 text-white hover:bg-green-700` (couleur hardcodée) remplacé par `variant="outline"` avec tokens CSS variables (`text-emerald-600 dark:text-emerald-400 border-emerald-500/50`) — compatible dark mode
- `StringList.vue` — 4 `<button>` natifs avec classes Tailwind manuelles remplacés par `<ToggleGroup>` shadcn-vue (accessible, keyboard-navigable) ; composant `toggle-group` + `toggle` ajoutés
- `FileList.vue` — `text-green-500` hardcodé → `text-emerald-500 dark:text-emerald-400`
- `ProjectCard.vue` — `<FolderOpenIcon class="size-4 mr-1">` remplacé par `gap-1.5` sur le bouton parent (pattern cohérent avec le reste de l'UI)
- `TranslateView.vue` — boutons "Traduire" et "Exporter" (`disabled`) wrappés dans `<Tooltip>` avec messages explicatifs ("Disponible après l'extraction des textes", "Disponible une fois la traduction terminée")
- `HomeView.vue` — grille de 4 cartes d'actions rapides (Mes projets, Nouveau projet, Glossaire, Paramètres) avec hover animé sur l'icône via group Tailwind ; architecture déclarative avec `QUICK_ACTIONS[]`
- `ProjectsView.vue` + `GlossaryView.vue` — `data-icon="inline-start"` remplacé par `class="size-4"` cohérent

**Composants shadcn-vue ajoutés**

- `toggle` — composant base (dépendance de toggle-group)
- `toggle-group` + `toggle-group-item` — groupes de boutons à sélection unique/multiple

---

### Added — Système de formatage/validation texte (EngineFormatter + ContentValidator)

**Architecture modulaire par moteur** — porté depuis l'analyse de `parsers/`

- `engines/formatter.rs` — trait `EngineFormatter` (`prepare_for_translation` / `restore_after_translation`) + `UniversalFormatter` (patterns communs : `%n` → `[ARG_n]`, guillemets japonais, whitespace, codes contrôle)
- `engines/validation.rs` — `ContentValidator` (filtrage universel : vide, placeholders seuls, identifiants techniques EV/MAP, code JS, extensions de fichiers, pipes)
- `rpgmv/formatter.rs` — `RpgMakerFormatter` : codes `\C[n]` → `[COLOR_n]`, `\N[n]` → `[NAME_n]`, `\V[n]`, `\I[n]`, `\W[n]`, `\A[n]`, `\P[n]`, `\G`, `\$`, `\F`, `\AA` + délégation `UniversalFormatter`
- `rpgmv/validation.rs` — `RpgMakerTextValidator` : ponctuation seule, chemins de fichiers (sauf codes `\n[`, `\C[`, `\N[`)
- `wolf/formatter.rs` — `WolfRpgFormatter` : codes `\E`, `\i[n]`, `\f[n]`, `@n`, `\cself[n]`, `\c[n]`/`\C[n]` (distinction case), `\sys[n]`, `\font[n]`, `\ax`, `\ay`, `\v[n]`, `\cdb`, `\-[n]`, `\space[n]`, `<C>`, `<R>`, `\>`, `<<`, `>>`
- `wolf/validation.rs` — `WolfRpgTextValidator` : suppression itérative placeholders (simples + imbriqués), filtrage chiffres seuls, `X[` (debug), extensions, chemins `Data\`/`Data/`
- `rpgm_classic/formatter.rs` — réexporte `RpgMakerFormatter` (mêmes codes moteur)
- `rpgm_classic/validation.rs` — réexporte `RpgMakerTextValidator`

**Intégration dans l'extraction**

- `extract_rpgmv` applique désormais `RpgMakerTextValidator::validate_text()` puis `RpgMakerFormatter::prepare_for_translation()` avant insertion SQLite
- Colonne `raw_text` ajoutée à la table `strings` (texte original avec codes moteur intacts)
- `source_text` contient le texte formaté (placeholders AI-friendly) — prêt pour envoi direct à Ollama
- `StringEntry` côté Rust et TypeScript mis à jour avec `raw_text`
- Requête `get_project_strings` étendue pour inclure `raw_text`

**Impact** : moins de strings inutiles envoyés à Ollama, codes moteur protégés par placeholders, restauration fiable après traduction

---

### Fixed — Bugs post-T05

**Bug : clic sur fichier dans FileList sans effet**
- `useProjectStrings` recevait `.value` évalué une seule fois à l'initialisation (valeur statique) → la query ne se ré-exécutait jamais quand `selectedPath` changeait
- Fix : signature modifiée en `MaybeRef<string | undefined>` + `queryKey` via `computed()` réactif
- `src/composables/useTranslate.ts` — `useProjectStrings` accepte maintenant `MaybeRef<string | undefined>`
- `src/views/TranslateView.vue` — passage d'un `computed()` à `useProjectStrings` au lieu de `.value`

**Bug : invoke `create_project` → "missing required key p"**
- La commande Rust utilisait `p: NewProject` (struct wrappé) → Tauri attendait `{ p: { ... } }` côté frontend
- Fix : paramètres aplatis dans la signature Rust (`name`, `game_path`, `work_path`, etc.)
- Tauri v2 convertit automatiquement camelCase → snake_case
- `src-tauri/src/commands/db_commands.rs` — `create_project` refactoré + struct `NewProject` supprimé

**Bug : textes non affichés après extraction (table strings sans raw_text)**
- Les bases SQLite existantes n'avaient pas la colonne `raw_text` → `INSERT` échouait silencieusement
- Fix : migration automatique dans `db.rs` (`ALTER TABLE strings ADD COLUMN raw_text`) + cleanup avant ré-extraction (`DELETE FROM strings/files WHERE project_id`)

**Bug : toasts vue-sonner invisibles**
- vue-sonner v2 n'injecte plus ses styles automatiquement → `[data-sonner-toaster]` sans `position: fixed` → invisible sous le layout
- Fix : import explicite de `vue-sonner/style.css` dans `src/main.ts`
- Fix : props `theme="dark"`, `rich-colors`, `position="bottom-right"` ajoutés sur `<Toaster />` dans `App.vue`

**Bug : AlertDialog d'extraction invisible**
- `isExtracting = true` et le lancement de l'extraction se faisaient dans le même flush cycle Vue → le dialog n'avait pas le temps de se rendre avant que l'extraction commence
- Fix : `await nextTick()` forcé après `isExtracting.value = true` pour garantir le rendu du dialog avant de procéder
- Ref manuelle `isExtracting` (au lieu de `isPending` de TanStack Query) pour contrôle explicite du cycle de vie du dialog

**Bug : code JavaScript visible dans les textes extraits**
- `ContentValidator` : filtrage étendu aux patterns `();`, `);`, `function `, `var `, `this.`, `self.`, et appels de méthodes ASCII `mot.mot(`
- `RpgMakerTextValidator` : filtrage des commandes de script RPG Maker (ASCII pur, type `PSS start`, `COMMANDE argument`)

### Added — UX extraction améliorée

- `src/views/TranslateView.vue` — `AlertDialog` modal bloquant pendant l'extraction : barre de progression, fichier en cours, empêche navigation (`@pointer-down-outside.prevent`, `@escape-key-down.prevent`)
- `src/App.vue` — `Toaster` (vue-sonner) intégré à la racine pour toast succès/erreur post-extraction
- `engines/rpgmv/extract.rs` — `extract_rpgmv` reçoit `AppHandle` et émet l'événement `extraction-progress` (`{ current, total, file }`) après chaque fichier `.txt` traité

### Analyzed — parsers/ (décision d'intégration partielle pour T06)

- Répertoire `parsers/` analysé : parser maison avec `GameEngineHandler` trait, `EngineFactory`, `TextUnit` + `PromptType` (Character, Dialogue, Item, Skill, System…), `EngineFormatter` trait (`prepare_for_translation` / `restore_after_translation`)
- **Décision** : ne pas remplacer `rvpacker-txt-rs-lib` (extraction MV/MZ déjà opérationnelle)
- **À intégrer en T06** : `PromptType` (catégorisation texte pour les prompts Ollama) + `EngineFormatter` (protection des placeholders `\C[1]`, `\n`, etc.)

---

### Refactored — Rust : reorganisation commands/ par moteur de jeu

- `src-tauri/src/commands/engines/` créé : chaque moteur dispose de ses propres modules `extract.rs` + `inject.rs` (ou `decrypt.rs`)
- `engines/rpgmv/` — `extract.rs` (contenu de l'ancien `parse.rs`) + `inject.rs` (ancien `write.rs`)
- `engines/rpgm_classic/` — `extract.rs` + `decrypt.rs` (anciens stubs de `parse.rs` + `decrypt.rs`)
- `engines/wolf/` — `extract.rs` + `inject.rs` (ancien `wolf.rs` splitté)
- Anciens fichiers supprimés : `parse.rs`, `write.rs`, `decrypt.rs`, `wolf.rs`
- `commands/mod.rs` et `lib.rs` mis à jour pour les nouveaux chemins de modules

---

### Added — T05 : Module RPG Maker MV/MZ Extraction

**Backend Rust**
- `src-tauri/Cargo.toml` — `rvpacker-txt-rs-lib = "11.1.2"` (requiert `RUSTFLAGS="-C target-feature=+aes,+sse2"`)
- `src-tauri/src/commands/engines/rpgmv/extract.rs` — `extract_rpgmv()` complet (Reader → .txt → SQLite) + `extract_speakers()` (noms via `<!-- NAME -->`)
- `src-tauri/src/commands/db_commands.rs` — `list_project_files` + struct `FileEntry` ; fix `create_project` → retourne `Project`

**Frontend**
- `src/composables/useTranslate.ts` — `useProjectFiles`, `useProjectStrings`, `useExtractProject`
- `src/components/translate/FileList.vue` — liste fichiers avec progression
- `src/components/translate/StringList.vue` — liste strings avec filtre statut + recherche
- `src/views/TranslateView.vue` — rewrite complet avec extraction, panneau fichiers/strings

---

### Added — T04 : Module Projets (CRUD + détection moteur)

**Frontend**
- `src/composables/useProjects.ts` — `useProjects()`, `useDeleteProject()`, `useOpenProject()`, `useProjectProgress()`, `ENGINE_LABELS`
- `src/components/projects/ProjectCard.vue` — carte projet avec barre de progression et `AlertDialog` suppression
- `src/components/projects/ProjectList.vue` — skeletons + état vide + liste
- `src/components/projects/NewProjectDialog.vue` — formulaire création : sélection dossier, détection moteur automatique, select langues, création + navigation

**Modified**
- `src/views/ProjectsView.vue` — rewrote complet, view mince orchestre composable + composants

---

### Added — Préparation T09 : Binaires Wolf RPG

- `src-tauri/resources/WolfTL.exe` — outil d'extraction/réinjection Wolf RPG (placé manuellement)
- `src-tauri/resources/UberWolfCli.exe` — outil de déchiffrement archives Wolf RPG (placé manuellement)
- `src-tauri/tauri.conf.json` — `"resources": ["resources/*"]` pour bundler les exécutables Wolf RPG

### Changed
- `tasks/T09-wolf-rpg.md` — chemin final : `src-tauri/resources/` (flat, sans sous-dossier)
- `docs/04-game-engines.md` — solution Linux documentée (`resources/` + Wine)
- `docs/PROGRESS.md` — statut binaires Wolf RPG mis à jour (✅ déjà en place)

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
