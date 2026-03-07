# Analyse : rpgmtranslate-qt

## Présentation générale

RPGMTranslate-Qt est un **outil de traduction statique de fichiers RPG Maker**, écrit en **C++23/Qt6** (UI) + **Rust** (logique backend). C'est une réécriture complète d'un projet Python original, motivée par la performance et la puissance. Il s'agit du projet open source le plus sophistiqué techniquement dans ce domaine.

**Licence : WTFPL** (Do What The Fuck You Want To Public License) — totalement libre.

---

## Stack technique

| Couche | Technologie |
|--------|-------------|
| UI | C++23 + Qt6 (widgets natifs) |
| Backend logique | Rust (static lib compilée via cbindgen/FFI) |
| Communication C++↔Rust | cbindgen (génération header C depuis Rust) + FFI |
| Build | CMake + Cargo |
| IA locale | Ollama (natif via `llm_connector`) |
| IA cloud | OpenAI, Anthropic, Gemini, DeepL, DeepSeek, Google, Yandex |
| Spell check | Dictionnaires Hunspell + LanguageTool (optionnel) |

**Point stratégique** : le backend Rust est compilé en `staticlib` et exposé à C++ via une API FFI générée automatiquement par cbindgen. C'est exactement le pattern que NeoGlot pourrait utiliser dans son côté Tauri/Rust.

---

## Architecture

```
┌─────────────────────────────────────────┐
│  C++23 / Qt6 UI                         │
│  MainWindow, TranslationTable, Menus... │
│               │ FFI (cbindgen)           │
└───────────────▼─────────────────────────┘
┌─────────────────────────────────────────┐
│  Rust static lib                        │
│  api.rs  :  read / write / translate    │
│  ffi.rs  :  exposition C des fonctions  │
│                                         │
│  rvpacker-txt-rs-lib  → parsing RPG Maker│
│  marshal-rs           → binaire XP/VX   │
│  rpgmad-lib           → déchiffrage .rgss│
│  llm_connector        → Ollama/OpenAI   │
└─────────────────────────────────────────┘
```

### Crates Rust utilisées — CRITIQUE pour NeoGlot

| Crate | Rôle | Utilisation pour NeoGlot |
|-------|------|--------------------------|
| `rvpacker-txt-rs-lib` | Parse RPG Maker MV/MZ/XP/VX/VXAce → fichiers `.txt` | **DIRECT** — à utiliser dans le backend Rust |
| `marshal-rs` | Désérialise le format binaire Ruby Marshal (RPG Maker XP/VX/VXAce) | **DIRECT** — résout le problème du format binaire |
| `rpgmad-lib` (alias `rpgmad-lib`) | Déchiffre les archives `.rgss` chiffrées | **DIRECT** — pour les jeux avec archives packées |
| `llm_connector` | Client LLM unifié (Ollama, OpenAI, Anthropic, Gemini...) | Adaptatble — notre backend est Python mais le pattern est utile |
| `whatlang` | Détection automatique de langue | Utile pour NeoGlot |
| `tiktoken-rs` | Comptage tokens (OpenAI o200k) | Pour estimation coût/limite tokens |
| `language-tokenizer` | Tokenisation multilingue (Snowball stemming) | Pour le glossaire / matching |

---

## Fonctionnalités remarquables

### 1. Workflow lecture→édition→écriture

```
read(source_path, translation_path, ...) → extrait textes vers .txt
    ↓ édition manuelle ou batch IA
write(source_path, translation_path, output_path, ...) → réinjecte
```

Parfaitement aligné avec l'objectif de NeoGlot. Jamais de modification des sources — toujours vers `output_path`.

### 2. Modes de lecture (`ReadMode`)

- Standard : extraction complète
- `hashes: Vec<u128>` : lecture incrémentale basée sur hash — ne relit que ce qui a changé

### 3. Gestion des types de jeux

```rust
fn get_game_type(game_title: &str, ...) -> GameType {
    // Détecte automatiquement : Termina, LisaRPG, ou None
    // Applique un traitement spécialisé par jeu
}
```
→ Extensible pour tout jeu avec particularités de format.

### 4. `DuplicateMode`

Contrôle comment gérer les textes dupliqués lors de l'extraction — utile pour ne pas re-traduire les doublons.

### 5. Requêtes LLM avec token-limiting intelligent

```rust
// Regroupe les fichiers en batches qui respectent la limite de tokens
let mut limited_files: Vec<HashMap<&str, Vec<String>>> = vec![HashMap::new()];
let tokenizer = o200k_base().unwrap();
for (file, strings) in files {
    for string in &strings {
        limit += tokenizer.encode_with_special_tokens(string).len();
    }
    if limit < token_limit { entry.insert(file, strings); }
    else { /* nouveau batch */ }
}
```

La structure JSON envoyée au LLM est très propre :
```rust
struct Request<'a> {
    source_language: &'a str,
    translation_language: &'a str,
    project_context: &'a str,    // contexte global du jeu
    local_context: &'a str,      // contexte du fichier courant
    glossary: &'a [GlossaryEntry],
    files: &'a HashMap<&'a str, Vec<String>>,  // filename → [textes]
}
```
La réponse attendue est un `HashMap<String, Vec<String>>` parsé depuis JSON.

### 6. Glossaire avec QC (Quality Check)

```rust
struct GlossaryEntry<'a> {
    term: &'a str,
    translation: &'a str,
    note: &'a str,
}
```

- Mode : Exact / Fuzzy / Both
- Case sensitivity configurable
- Mode "permissive" (permet formes plus capitalisées)
- QC : vérifie que chaque terme du glossaire est présent/cohérent dans tous les fichiers

### 7. Features CAT (Computer-Assisted Translation)
- Bookmarks sur lignes
- Recherche cross-fichiers avec expressions régulières
- Whitespace highlighter (voir les espaces/tabulations)
- Spell checker avec dictionnaires Hunspell (40+ langues)
- LanguageTool (WIP) pour linting grammatical avancé

### 8. Batch processing
- **Trim** : supprime espaces début/fin
- **Translate** : traduit toutes les lignes sélectionnées
- **Wrap** : word wrap paramétrable (dangereux, à utiliser avec précaution)

### 9. Purge
Supprime les traductions pour les textes qui n'existent plus dans le jeu source — garde le fichier de traduction propre.

### 10. Auto-updater
`AutoUpdater.cpp` — mise à jour automatique de l'application.

### 11. Détection automatique de langue (`whatlang`)
Détecte la langue du texte source pour adapter l'algorithme de tokenisation. Très utile pour les jeux multilingues.

---

## Structure UI (C++ Qt6)

| Composant | Rôle |
|-----------|------|
| `MainWindow` | Fenêtre principale |
| `TranslationTable` | Tableau original/traduction (composant central) |
| `TranslationTableModel` | QAbstractTableModel — données |
| `TranslationTableDelegate` | Rendu cellules |
| `LabelInput` / `TranslationInput` | Éditeurs inline dans le tableau |
| `SpellHighlighter` | Soulignement rouge fautes orthographe |
| `WhitespaceHighlighter` | Visualisation espaces/tabs |
| `GlossaryMenu` | Gestion glossaire |
| `SearchMenu` / `SearchPanelDock` | Recherche cross-fichiers |
| `BatchMenu` | Traitement en lot |
| `BookmarkMenu` | Favoris sur lignes |
| `PurgeMenu` | Suppression traductions obsolètes |
| `ReadMenu` / `WriteMenu` | Lecture/écriture fichiers |
| `FileSelectMenu` | Sélection fichiers à traiter |
| `MatchMenu` / `MatchTable` | Résultats QC glossaire |
| `TabList` / `TabPanel` | Navigation entre fichiers ouverts |
| `SettingsWindow` | Configuration (projets + global) |
| `TaskWorker` | Worker thread pour tâches asynchrones |
| `FFILogger` | Bridge logs Rust → Qt |

---

## Points forts à reprendre pour NeoGlot

| Pattern | Détail | Priorité |
|---------|--------|----------|
| **`rvpacker-txt-rs-lib`** | Crate Rust qui gère TOUT le parsing RPG Maker | CRITIQUE — à intégrer directement |
| **`marshal-rs`** | Parsing binaire Ruby Marshal pour RPG Maker XP/VX/VXAce | CRITIQUE |
| **`rpgmad-lib`** | Déchiffrage archives `.rgss` | CRITIQUE |
| **Structure Request LLM** | `{source_lang, target_lang, project_context, local_context, glossary, files}` | DIRECT — reprendre exactement |
| **Token-limiting par batch** | Grouper les textes par limite de tokens | IMPORTANT |
| **`GlossaryEntry` (term, translation, note)** | Structure minimale mais complète | À reprendre en SQLite |
| **QC glossaire (fuzzy+exact)** | Vérification cohérence après traduction | Phase avancée NeoGlot |
| **`ReadMode` incrémental par hash** | Ne relit que ce qui a changé | IMPORTANT pour performance |
| **`DuplicateMode`** | Gestion textes identiques | À implémenter |
| **`GameType` custom** | Traitement spécialisé par jeu | À étendre pour nos 4 moteurs |
| **Purge** | Nettoyer les traductions obsolètes | Utile pour re-runs |
| **`FFILogger`** | Bridge logs backend → frontend | Pattern à reproduire Rust→React |
| **Détection de langue auto** | `whatlang` | Optionnel mais bon UX |

---

## Points faibles / différences avec NeoGlot

| Aspect | rpgmtranslate-qt | NeoGlot |
|--------|-----------------|---------|
| UI | C++/Qt6 (puissant mais lourd) | React + Tauri (moderne, web) |
| Backend logique | Rust directement | Python + FastAPI (plus facile pour IA) |
| Moteurs supportés | RPG Maker uniquement | RPG Maker + Wolf + Bakin |
| IA locale | Ollama (oui) | Ollama (obligatoire) |
| Base de données | Non (fichiers .txt) | SQLite (projets, glossaire, cache) |
| Streaming traduction | Non | Oui (SSE via FastAPI) |
| Multi-plateforme | Oui | Oui |

---

## Crates Rust à inclure dans NeoGlot

Ces crates sont utilisables directement dans le backend Rust de Tauri v2 :

```toml
# Cargo.toml de NeoGlot
[dependencies]
rvpacker-txt-rs-lib = "11.1.2"   # parsing RPG Maker MV/MZ + XP/VX/VXAce
marshal-rs = "*"                  # parsing binaire Ruby Marshal
rpgmad-lib = "4.0.0"             # déchiffrage archives .rgss
whatlang = "0.18.0"              # détection langue automatique
walkdir = "2.5.0"                # parcours récursif de répertoires
```

Cela signifie que **la couche parsing RPG Maker peut être en Rust**, appelée depuis les commandes Tauri — sans Python pour cette partie.

---

## Résumé pour NeoGlot

RPGMTranslate-Qt est de loin **la référence technique la plus précieuse**. Ses crates Rust (`rvpacker-txt-rs-lib`, `marshal-rs`, `rpgmad-lib`) résolvent directement les problèmes les plus difficiles de NeoGlot : le parsing des formats RPG Maker, y compris les formats binaires XP/VX/VXAce et les archives chiffrées.

Sa structure de requête LLM (`project_context` + `local_context` + `glossary` + `files`) est le meilleur pattern vu dans tous les projets analysés et doit être repris tel quel.

**Score de pertinence : 10/10** — bibliothèques réutilisables directement, patterns architecturaux excellents.
