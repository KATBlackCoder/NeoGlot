# Schéma Base de Données — NeoGlot

**Moteur** : SQLite via `rusqlite` (Rust — accès exclusif). Pas de `tauri-plugin-sql`.
**Fichier** : `~/.local/share/neoglot/neoglot.db` (Linux) / `%APPDATA%\neoglot\neoglot.db` (Windows)

---

## Schéma SQL

```sql
-- ═══════════════════════════════════════════════════════
-- TABLE projects : un projet = un jeu à traduire
-- ═══════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS projects (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    game_path       TEXT NOT NULL,          -- dossier jeu original (lecture seule)
    work_path       TEXT NOT NULL,          -- copie de travail NeoGlot
    engine          TEXT NOT NULL,          -- 'rpgmv' | 'rpgmz' | 'rpgmxp' | 'rpgmvx' | 'rpgmvxa' | 'wolf' | 'bakin'
    source_lang     TEXT NOT NULL DEFAULT 'ja',
    target_lang     TEXT NOT NULL DEFAULT 'fr',
    project_context TEXT NOT NULL DEFAULT '',  -- contexte fourni à l'IA (genre, ambiance, noms propres)
    status          TEXT NOT NULL DEFAULT 'created',  -- 'created' | 'extracted' | 'translating' | 'done'
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ═══════════════════════════════════════════════════════
-- TABLE files : fichiers d'un projet
-- ═══════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS files (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id      INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    relative_path   TEXT NOT NULL,          -- ex: 'data/Map001.json'
    status          TEXT NOT NULL DEFAULT 'pending',  -- 'pending' | 'extracted' | 'translated' | 'injected'
    strings_total   INTEGER DEFAULT 0,
    strings_done    INTEGER DEFAULT 0,
    UNIQUE(project_id, relative_path)
);

-- ═══════════════════════════════════════════════════════
-- TABLE strings : textes extraits
-- ═══════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS strings (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id         INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    source_hash     TEXT NOT NULL,          -- SHA256(source_text) pour déduplication
    source_text     TEXT NOT NULL,
    context_path    TEXT NOT NULL DEFAULT '',  -- ex: 'Map001/event_12/page_0/cmd_5' (SLR pattern)
    event_code      INTEGER,               -- code EVENT RPG Maker (101, 401, etc.) ou Wolf (101, 102)
    row_index       INTEGER,               -- position dans le fichier pour réinjection
    translation     TEXT,
    status          TEXT NOT NULL DEFAULT 'pending',  -- 'pending' | 'translated' | 'reviewed' | 'skipped'
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_strings_file_id ON strings(file_id);
CREATE INDEX IF NOT EXISTS idx_strings_source_hash ON strings(source_hash);
CREATE INDEX IF NOT EXISTS idx_strings_status ON strings(status);

-- ═══════════════════════════════════════════════════════
-- TABLE translation_cache : cache global inter-projets
-- ═══════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS translation_cache (
    source_hash     TEXT NOT NULL,
    source_text     TEXT NOT NULL,
    translation     TEXT NOT NULL,
    model           TEXT NOT NULL,          -- ex: 'llama3:8b'
    source_lang     TEXT NOT NULL DEFAULT 'ja',
    target_lang     TEXT NOT NULL DEFAULT 'fr',
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (source_hash, model, source_lang, target_lang)
);

-- ═══════════════════════════════════════════════════════
-- TABLE glossary : termes imposés par l'utilisateur
-- ═══════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS glossary (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id      INTEGER REFERENCES projects(id) ON DELETE CASCADE,
                    -- NULL = glossaire global (tous projets)
    term            TEXT NOT NULL,          -- terme original (japonais ou autre)
    translation     TEXT NOT NULL,          -- traduction imposée
    note            TEXT DEFAULT '',        -- contexte d'usage (ex: "nom propre, héros principal")
    match_mode      TEXT NOT NULL DEFAULT 'exact',  -- 'exact' | 'contains' | 'regex'
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_glossary_project ON glossary(project_id);

-- ═══════════════════════════════════════════════════════
-- TABLE translation_jobs : suivi des jobs en cours/terminés
-- ═══════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS translation_jobs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id      INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    model           TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'queued',  -- 'queued' | 'running' | 'done' | 'failed' | 'cancelled'
    strings_total   INTEGER DEFAULT 0,
    strings_done    INTEGER DEFAULT 0,
    error_message   TEXT,
    started_at      DATETIME,
    finished_at     DATETIME,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

## Notes d'utilisation

### Accès DB (Rust — rusqlite uniquement)

Toutes les lectures et écritures passent par des commandes Rust via `invoke()`.
Le frontend n'a **aucun accès direct** à la DB.

```typescript
// Charger les projets
const projects = await invoke<Project[]>('list_projects');

// Progression d'un projet
const progress = await invoke<{ done: number, total: number }>(
  'get_project_progress', { projectId }
);

// Strings d'un projet (filtré par status)
const strings = await invoke<StringEntry[]>(
  'get_project_strings', { projectId, statusFilter: 'pending' }
);
```

### Accès Rust (écriture — rusqlite)

```rust
// Batch insert strings après extraction (db_commands.rs)
for s in &strings {
    conn.execute(
        "INSERT OR IGNORE INTO strings (file_id, source_hash, source_text, context_path, event_code, row_index)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![file_id, s.source_hash, s.source_text, s.context_path, s.event_code, s.row_index],
    )?;
}
```

### Déduplication

Avant traduction, on vérifie le cache :

```sql
SELECT t.translation
FROM strings s
JOIN translation_cache t ON s.source_hash = t.source_hash
WHERE t.model = ? AND t.source_lang = ? AND t.target_lang = ?
  AND s.status = 'pending'
```

Les strings avec cache existant sont marquées `translated` sans appel Ollama.

### Propagation des traductions

Quand un string est traduit, toutes les occurrences du même `source_hash` dans le même projet reçoivent la même traduction :

```sql
UPDATE strings
SET translation = ?, status = 'translated', updated_at = CURRENT_TIMESTAMP
WHERE source_hash = ? AND file_id IN (
  SELECT id FROM files WHERE project_id = ?
)
```
