# Workflow de Traduction — NeoGlot

## Diagramme complet

```
Utilisateur ouvre un projet
          │
          ▼
┌─────────────────────────────────────────────────────────────┐
│  ÉTAPE 1 — Détection du moteur                              │
│  [Rust] detect_engine(game_path)                            │
│  → cherche : Game.exe + www/data/*.json → RPG Maker MV/MZ   │
│  → cherche : Game.exe + Data/Scripts.rxdata → RPG Maker XP  │
│  → cherche : Game.exe + Data/BasicData/*.wolf → Wolf RPG    │
│  → cherche : *.bakin → RPG Bakin                            │
└──────────────────────┬──────────────────────────────────────┘
                       │
          ┌────────────┴────────────┐
          │ RPG Maker XP/VX/VXAce  │ Wolf RPG
          ▼                        ▼
┌──────────────────┐    ┌────────────────────────────────────┐
│ ÉTAPE 2a         │    │ ÉTAPE 2b                           │
│ Déchiffrement    │    │ Déchiffrement archive Wolf         │
│ .rgss / .rgss2   │    │ [Python] UberWolfCli.exe           │
│ [Rust] rpgmad-lib│    │   → auto-détecte clé depuis        │
│                  │    │     Game.exe                       │
│ → extrait Data/  │    │   → déchiffre *.wolf → dossiers    │
└────────┬─────────┘    └──────────────┬─────────────────────┘
         │                             │
         ▼                             ▼
┌──────────────────────────────────────────────────────────────┐
│  ÉTAPE 3 — Parse Speakers (RPG Maker uniquement)             │
│  [Rust] rvpacker-txt-rs-lib (mode speakers)                  │
│  → extrait noms de personnages → pré-remplit glossaire       │
│  → évite traductions incohérentes des noms propres           │
└──────────────────────────────────┬───────────────────────────┘
                                   │
          ┌────────────────────────┴──────────────────────┐
          │ RPG Maker MV/MZ/XP/VX/VXAce                  │ Wolf RPG
          ▼                                               ▼
┌──────────────────────┐                    ┌──────────────────────┐
│ ÉTAPE 4a             │                    │ ÉTAPE 4b             │
│ Extraction RPG Maker │                    │ Extraction Wolf RPG  │
│ [Rust] rvpacker-     │                    │ [Python subprocess]  │
│ txt-rs-lib           │                    │ WolfTL --create      │
│                      │                    │   game_data/  →      │
│ → JSON par fichier   │                    │   dump/              │
│ → event codes:       │                    │ Codes extraits :     │
│   101,401 (messages) │                    │   101 (Message)      │
│   102,402 (choices)  │                    │   102 (Choices)      │
│   122 (set string)   │                    │   122 (SetString)    │
│   356 (plugin cmd)   │                    │   250 (Database)     │
└──────────┬───────────┘                    └──────────┬───────────┘
           │                                           │
           └──────────────────┬────────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  ÉTAPE 5 — Stockage SQLite                                      │
│  [Python] INSERT INTO strings (source_hash, source_text,        │
│             context_path, event_code, row_index)                │
│  context_path = "Map001/event_12/page_0/cmd_5"                  │
│  → permet traduction différente selon contexte (SLR pattern)   │
└─────────────────────────────────┬───────────────────────────────┘
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│  ÉTAPE 6 — Déduplication                                        │
│  [Python] GROUP BY source_hash                                  │
│  → un seul appel Ollama par texte unique                        │
│  → vérifier translation_cache → skip si déjà traduit            │
│  → résultat : liste de strings uniques à traduire               │
└─────────────────────────────────┬───────────────────────────────┘
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│  ÉTAPE 7 — Construction de la requête LLM                       │
│  [Python] pour chaque batch :                                   │
│  {                                                              │
│    "source_lang": "Japanese",                                   │
│    "target_lang": "French",                                     │
│    "project_context": "RPG médiéval-fantastique, héros: Yuki",  │
│    "local_context": "Combat boss final, tension dramatique",    │
│    "glossary": [{"term": "勇者", "translation": "Héros"}],       │
│    "files": {                                                   │
│      "Map001.json": ["text1", "text2", ...]                     │
│    }                                                            │
│  }                                                              │
└─────────────────────────────────┬───────────────────────────────┘
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│  ÉTAPE 8 — Protection des placeholders                          │
│  [Python] utils/placeholders.py                                 │
│  → remplacer AVANT envoi à l'IA :                               │
│    \SE[x] → <SE_x>   (effets sonores)                           │
│    \BGM[x] → <BGM_x> (musique)                                  │
│    \n[x] → <NAME_x>  (noms de personnages)                      │
│    \V[x] → <VAR_x>   (variables)                                │
│    \C[x] → <COLOR_x> (couleurs)                                 │
│    \I[x] → <ICON_x>  (icônes)                                   │
│    \{, \} → <SIZE_U>, <SIZE_D>                                   │
│  → restaurer APRÈS réception de la traduction                   │
└─────────────────────────────────┬───────────────────────────────┘
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│  ÉTAPE 9 — Groupage en batches par tokens                       │
│  [Python] utils/tokenizer.py                                    │
│  → approximation : 4 chars ≈ 1 token                            │
│  → limite configurable (défaut : 2048 tokens par batch)         │
│  → respecter les limites du contexte du modèle Ollama           │
└─────────────────────────────────┬───────────────────────────────┘
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│  ÉTAPE 10 — Appel Ollama + streaming                            │
│  [Python] POST localhost:11434/api/generate (stream: true)      │
│  → pour chaque token reçu :                                     │
│    → stocker partiel en mémoire                                 │
│    → émettre via SSE → frontend affiche en temps réel           │
│  → à la fin du batch :                                          │
│    → parser le JSON de réponse                                  │
│    → UPDATE strings SET translation=?, status='translated'      │
│    → INSERT INTO translation_cache                              │
└─────────────────────────────────┬───────────────────────────────┘
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│  ÉTAPE 11 — Restauration des placeholders                       │
│  [Python] utils/placeholders.py restore()                       │
│  → <SE_x> → \SE[x], <NAME_x> → \n[x], etc.                     │
│  → vérifier qu'aucun placeholder n'est perdu                    │
└─────────────────────────────────┬───────────────────────────────┘
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│  ÉTAPE 12 — QC Glossaire (optionnel)                            │
│  [Python] pour chaque terme du glossaire :                      │
│  → vérifier que la traduction imposée est bien présente         │
│  → si mismatch : marquer string pour révision humaine           │
└─────────────────────────────────┬───────────────────────────────┘
                                  ▼
                ┌─────────────────┴─────────────────┐
                │ RPG Maker                         │ Wolf RPG
                ▼                                   ▼
┌───────────────────────────┐       ┌───────────────────────────────┐
│ ÉTAPE 13a                 │       │ ÉTAPE 13b                     │
│ Réinjection RPG Maker     │       │ Réinjection Wolf RPG          │
│ [Rust] rvpacker-txt-rs-   │       │ [Python subprocess]           │
│ lib write mode            │       │ WolfTL --patch dump/ game/    │
│                           │       │   (--inplace pour modifier     │
│ → écrire JSON traduits    │       │    les fichiers en place)     │
│   dans work_path          │       │                               │
└───────────────────────────┘       └───────────────────────────────┘
```

---

## Gestion des erreurs

| Erreur | Comportement |
|--------|-------------|
| Ollama non disponible | Bloquer étape 10, afficher modal avec lien install |
| Placeholder perdu après traduction | Marquer string `status='error'`, logger |
| Timeout Ollama (>60s) | Retry 2x puis marquer batch `failed`, continuer les autres |
| Fichier JSON malformé (Wolf) | Skip le fichier, logger, continuer |
| UberWolf échec déchiffrement | Afficher erreur : version WolfPro non supportée ou clé manquante |

## Modes de traduction

| Mode | Description |
|------|-------------|
| **Tout traduire** | Traduit tous les strings `pending` |
| **Reprendre** | Saute les strings déjà `translated`, continue depuis le dernier point |
| **Fichier unique** | Traduit seulement le fichier sélectionné |
| **Re-traduire** | Force la retraduction même si `translated` (ignore le cache) |
