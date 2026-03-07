# Synthèse — Analyse comparative des projets de référence

## Projets analysés

| # | Projet | Fichier |
|---|--------|---------|
| 1 | DazedMTLTool | `01-DazedMTLTool.md` |
| 2 | MTool | `02-MTool.md` |
| 3 | rpgmtranslate-qt | `03-rpgmtranslate-qt.md` |
| 4 | SLR Translator | `04-SLR-Translator.md` |
| 5 | WolfTL | `05-WolfTL.md` |
| 6 | UberWolf | `06-UberWolf.md` |
| 7 | Tauri v2 Plugins | `07-Tauri-Plugins.md` |
| 8 | Bibliothèques additionnelles (SQLModel, sse-starlette, tauri-plugin-log) | `08-Additional-Libraries.md` |

---

## Tableau comparatif rapide

| Critère | DazedMTL | MTool | rpgmtranslate-qt | SLR Translator |
|---------|----------|-------|-----------------|----------------|
| Stack | Python + PyQt5 | NW.js (bytecode) | C++23/Qt6 + Rust | NW.js + JS |
| Open source | Oui | Non | Oui (WTFPL) | Oui (GPL) |
| IA locale | Non | Oui (Sugoi/Llama3) | Oui (Ollama) | Non |
| RPG Maker MV/MZ | Oui | Oui (hook) | Oui | Oui |
| RPG Maker XP/VX | Oui (YAML) | Oui (hook) | Oui (marshal-rs) | Oui |
| Wolf RPG | Oui | Oui (hook) | Non | Oui |
| RPG Bakin | Non | Oui (hook) | Non | Non |
| Base de données | Non | Non | Non | localStorage |
| Gestion projets | Non | Non | Non | Oui (.trans) |
| Traduction par contexte | Non | N/A | Non | **Oui** |
| Glossaire avec QC | Non | Non | **Oui** | Non |
| Batch processing | Oui | Non | Oui | Oui |
| Placeholder protection | **Oui** | N/A | Non | Non |
| Token limiting LLM | Non | N/A | **Oui** | Non |
| Linux compatible | Oui | Non | Oui | Non |
| Architecture extensible | Non | N/A | Non | **Oui (addons)** |
| Pertinence NeoGlot | 8/10 | 3/10 | **10/10** | 9/10 |

---

## Décisions d'architecture pour NeoGlot

### Ce qu'on intègre directement

#### Crates Rust (depuis rpgmtranslate-qt)
```toml
rvpacker-txt-rs-lib = "11.1.2"   # Parsing RPG Maker MV/MZ/XP/VX/VXAce
marshal-rs = "*"                  # Format binaire Ruby Marshal
rpgmad-lib = "4.0.0"             # Déchiffrage archives .rgss
whatlang = "0.18.0"              # Détection langue automatique
walkdir = "2.5.0"                # Parcours répertoires
```

Ces crates remplacent des mois de développement. Le parsing RPG Maker vit dans le backend Rust de Tauri.

#### Outils Wolf RPG (depuis SLR Translator)
- `wolfDec` + `wolftrans` — à étudier pour le module Wolf RPG
- Ou portager en Python/Rust en s'en inspirant

#### Patterns de code

| Pattern | Source | Application NeoGlot |
|---------|--------|---------------------|
| Placeholder protection (`\SE[...]`, `\BGM[...]`) | DazedMTL | Module Python : protéger avant envoi Ollama |
| Structure requête LLM | rpgmtranslate-qt | `{source_lang, target_lang, project_context, local_context, glossary, files}` |
| Token limiting par batch | rpgmtranslate-qt | Grouper les textes selon limite tokens Ollama |
| Parse Speakers avant traduction | DazedMTL | Étape 1 du workflow RPG Maker |
| Cache par hash | DazedMTL + MTool | SQLite : `hash(source) → translation` |
| Traduction par contexte | SLR | SQLite : `(source, context_path) → translation` |
| Glossaire (term, translation, note) + QC | rpgmtranslate-qt | Table `glossary` SQLite |
| Architecture addon par moteur | SLR | Un module Python par moteur, interface commune |
| DuplicateMode | rpgmtranslate-qt | Ne pas re-traduire les doublons |
| Purge des traductions obsolètes | rpgmtranslate-qt | Nettoyage lors de mise à jour du jeu source |

---

## Schema SQLite NeoGlot — basé sur l'analyse

```sql
-- Projets
CREATE TABLE projects (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    game_path TEXT NOT NULL,       -- dossier original du jeu (lecture seule)
    work_path TEXT NOT NULL,        -- copie de travail
    engine TEXT NOT NULL,           -- 'rpgmv', 'rpgmz', 'rpgmace', 'wolf', 'bakin'
    source_lang TEXT DEFAULT 'ja',
    target_lang TEXT DEFAULT 'fr',
    project_context TEXT DEFAULT '', -- contexte global envoyé au LLM
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Fichiers du projet
CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    project_id INTEGER REFERENCES projects(id),
    relative_path TEXT NOT NULL,   -- chemin relatif depuis work_path
    status TEXT DEFAULT 'pending'  -- 'pending', 'in_progress', 'done'
);

-- Textes extraits
CREATE TABLE strings (
    id INTEGER PRIMARY KEY,
    file_id INTEGER REFERENCES files(id),
    source_hash TEXT NOT NULL,     -- hash du texte source
    source_text TEXT NOT NULL,
    context_path TEXT DEFAULT '',  -- ex: "Map001/events/5/pages/0/list/3"
    event_code INTEGER,            -- 101, 102, 122, 356, etc.
    translation TEXT DEFAULT '',
    status TEXT DEFAULT 'pending', -- 'pending', 'translated', 'reviewed'
    row_index INTEGER
);

-- Cache de traduction (réutilisation cross-projets)
CREATE TABLE translation_cache (
    source_hash TEXT PRIMARY KEY,
    source_text TEXT NOT NULL,
    translation TEXT NOT NULL,
    model TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Glossaire par projet
CREATE TABLE glossary (
    id INTEGER PRIMARY KEY,
    project_id INTEGER REFERENCES projects(id),
    term TEXT NOT NULL,
    translation TEXT NOT NULL,
    note TEXT DEFAULT '',
    match_mode TEXT DEFAULT 'exact' -- 'exact', 'fuzzy', 'both'
);
```

---

## Architecture finale NeoGlot — mise à jour post-analyse

```
NeoGlot/
├── src/                    # React + Tauri frontend
│   ├── components/ui/      # shadcn/ui
│   ├── components/features/
│   │   ├── ProjectManager/ # création/ouverture projets
│   │   ├── FileExplorer/   # liste des fichiers du projet
│   │   ├── TranslationTable/ # éditeur source/traduction (inspiré rpgmtranslate-qt)
│   │   ├── GlossaryPanel/  # glossaire + QC
│   │   └── OllamaStatus/   # vérification Ollama au démarrage
│   └── stores/             # Zustand
│
├── src-tauri/              # Rust/Tauri
│   └── src/
│       ├── commands/
│       │   ├── parse.rs    # appelle rvpacker-txt-rs-lib + marshal-rs
│       │   ├── write.rs    # réinjection traduction
│       │   └── decrypt.rs  # rpgmad-lib pour .rgss
│       └── Cargo.toml      # rvpacker-txt-rs-lib, marshal-rs, rpgmad-lib
│
└── backend/                # Python + FastAPI
    ├── routers/
    │   ├── projects.py     # CRUD projets (SQLite)
    │   ├── translation.py  # queue Ollama + streaming SSE
    │   ├── engines/
    │   │   ├── base.py     # interface commune (pattern SLR addons)
    │   │   ├── rpgmv.py    # placeholder protection + event codes
    │   │   ├── rpgmz.py
    │   │   ├── wolf.py     # UberWolfCli (déchiffrement) + WolfTL (parsing)
    │   │   └── bakin.py
    │   └── ollama.py       # client Ollama + vérification démarrage
    ├── models/
    │   ├── project.py
    │   ├── string_entry.py
    │   └── glossary.py
    └── database.py         # SQLAlchemy + SQLite
```

---

## Workflow de traduction NeoGlot — optimisé

Basé sur les meilleures pratiques de tous les projets :

```
1. Ouvrir projet → détecter moteur (rvpacker-txt-rs-lib)
2a. [RPG Maker] Décrypter archives .rgss si nécessaire (rpgmad-lib / Rust)
2b. [Wolf RPG] Décrypter archives .wolf si nécessaire (UberWolfCli)
3. [RPG Maker] Parse Speakers → glossaire automatique (DazedMTL pattern)
4a. [RPG Maker] Extraire textes via rvpacker-txt-rs-lib (Rust → JSON)
4b. [Wolf RPG] Extraire textes via WolfTL --create (C++ CLI → JSON)
5. Stocker textes → SQLite avec context_path + event_code (SLR pattern)
6. Dédupliquer par hash → ne traduire qu'une fois par texte unique
7. Construire requête LLM :
   {project_context, local_context, glossary, files: {filename: [texts]}}
8. Placeholder protection : \SE[...], \BGM[...] → tokens (DazedMTL pattern)
9. Grouper par token limit → batches (rpgmtranslate-qt pattern)
10. Envoyer à Ollama → stream résultats → stocker SQLite
11. Restaurer placeholders dans les résultats
12. QC glossaire optionnel (rpgmtranslate-qt pattern)
13a. [RPG Maker] Réinjecter via rvpacker-txt-rs-lib --write → output_path
13b. [Wolf RPG] Réinjecter via WolfTL --patch → patched/data/
```

---

## Plugins Tauri v2 retenus

Voir `07-Tauri-Plugins.md` pour l'analyse complète.

| Plugin | Usage NeoGlot | Priorité |
|--------|--------------|----------|
| `tauri-plugin-shell` | Lancer Python backend + WolfTL + UberWolfCli | **P0** |
| `tauri-plugin-fs` | Lecture/écriture fichiers jeu + JSON Wolf RPG | **P0** |
| `tauri-plugin-dialog` | Sélection dossier jeu / fichier Game.exe | **P0** |
| `tauri-plugin-sql` | SQLite — projets, strings, glossaire, cache | **P0** |
| `tauri-plugin-store` | Préférences utilisateur (modèle Ollama, langues) | P1 |
| `tauri-plugin-http` | Health check Ollama + liste modèles | P1 |
| `tauri-plugin-window-state` | Sauvegarder taille/position fenêtre | P2 |
| `tauri-plugin-notification` | Notifier fin de batch de traduction | P2 |
| `tauri-plugin-process` | Quitter proprement (tuer Python backend) | P2 |
| `tauri-plugin-clipboard-manager` | Copier traduction / overlay translator | P3 |

---

## Priorités de développement confirmées

1. **Scaffold Tauri v2 + React + shadcn** — base UI + plugins P0
2. **Vérification Ollama** au démarrage (`tauri-plugin-http` health check)
3. **Module Projets** + SQLite (`tauri-plugin-sql` + schéma ci-dessus)
4. **Module RPG Maker MV/MZ** — intégrer `rvpacker-txt-rs-lib` dans Rust, Python pour Ollama
5. **Module traduction** — queue, batches tokens, placeholder protection, streaming SSE
6. **Module Glossaire** — term/translation/note + QC
7. **Module RPG Maker XP/VX/VXAce** — `marshal-rs` + `rpgmad-lib`
8. **Module Wolf RPG** — **UberWolfCli** (déchiffrement) + **WolfTL** (parsing/réinjection), MIT
9. **Module RPG Bakin** — from scratch (seul MTool le supporte, non réutilisable)
10. **Traduction par contexte** — phase avancée (SLR pattern)
