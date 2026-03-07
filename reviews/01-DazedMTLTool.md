# Analyse : DazedMTLTool-main

## Présentation générale

DazedMTLTool est un outil de traduction de jeux basé sur Python, avec une interface graphique PyQt5 et un backend de traduction via API OpenAI/Gemini. C'est l'un des projets de référence les plus complets dans ce domaine open source.

---

## Stack technique

| Couche | Technologie |
|--------|-------------|
| UI | PyQt5 (bureau natif) |
| Backend | Python pur (pas de serveur séparé) |
| IA | OpenAI API + Google Gemini (cloud, clé API requise) |
| Config | `.env` + `QSettings` |
| Distribution | Script `START.bat` + venv automatique |

**Différence majeure avec NeoGlot** : pas d'IA locale (pas d'Ollama), cloud uniquement, et pas de base de données (pas de SQLite).

---

## Moteurs supportés

| Moteur | Fichiers | Module |
|--------|----------|--------|
| RPG Maker MV/MZ | `.json` | `modules/rpgmakermvmz.py` |
| RPG Maker ACE | `.yaml` | `modules/rpgmakerace.py` |
| RPG Maker Plugins | `.js`, `.rb` | `modules/rpgmakerplugin.py` |
| Wolf RPG | `.json`, `.txt` | `modules/wolf.py`, `modules/wolf2.py` |
| Ren'Py | `.rpy` | `modules/renpy.py` |
| TyranoBuilder | `.ks` | `modules/tyrano.py` |
| Kirikiri | `.ks`, `.tjs`, `.ssd`, `.asd` | `modules/kirikiri.py` |
| NScripter | `.txt` | `modules/nscript.py` |
| Unity | `.txt` | `modules/unity.py` |
| SRPG Studio | `.json` | `modules/srpg.py` |
| CSV | `.csv` | `modules/csv.py` |
| Images | `.png` | `modules/images.py` |
| JSON générique | `.json` | `modules/json.py` |
| Regex (custom) | `.txt`, `.json`, etc. | `modules/regex.py` |
| Aquedi4 | `.json` | `modules/aquedi4.py` |

**15+ moteurs supportés** — c'est la couverture la plus large observée.

---

## Architecture du code

### Structure des modules

```
modules/
├── main.py           # Dispatch : sélection du moteur + ThreadPoolExecutor
├── rpgmakermvmz.py   # Parsing JSON RPG Maker MV/MZ + codes événements
├── rpgmakerace.py    # RPG Maker XP/VX/VXAce (YAML via ruamel.yaml)
├── wolf.py / wolf2.py
└── ...

util/
├── translation.py    # Moteur de traduction centralisé (OpenAI/Gemini)
├── dazedwrap.py      # Word wrap adapté aux polices japonaises
└── subprocess_runner.py

gui/
├── main.py           # Fenêtre principale PyQt5, sidebar VSCode-style
├── config_tab.py     # Onglet configuration .env
├── translation_tab.py
└── ...
```

### Pattern de traduction (util/translation.py)

Points notables :
- **Cache de traduction** : évite de re-traduire des textes identiques dans le même run
- **Système de placeholders** : protège les codes de script (`\SE[...]`, `\BGM[...]`, etc.) de la traduction — les remplace par tokens avant l'appel API, puis les réinjecte
- **Support OpenAI + Gemini** : abstraction commune avec switching via `API_PROVIDER`
- **Calcul de coût estimatif** : mode `estimate` qui comptabilise les tokens sans appeler l'API
- **Historique de conversation** : `MAXHISTORY=10` messages gardés pour contexte cohérent
- **Retry automatique** : décorateur `@retry` sur les appels API
- **Tiktoken** : comptage de tokens précis par modèle

### Concurrence

- `ThreadPoolExecutor` avec `fileThreads` workers configurable
- Un thread par fichier (recommandé = 1 pour GPT-4 à cause des rate limits)
- Verrous (`threading.Lock`) sur vocab, speakers, compteurs

### Gestion des speakers (RPG Maker)

- Mode "Parse Speakers" : extrait les noms de personnages sans traduire
- `FACENAME101` : mapping face_name → speaker via CODE 101
- `FIRSTLINESPEAKERS` : détection du speaker depuis la première ligne de dialogue
- Export vers `vocab.txt` pour cohérence de traduction

---

## Fonctionnalités remarquables

### Vocab + Prompt système
- `vocab.txt` : dictionnaire de termes/noms de personnages injectés dans le prompt
- `prompt.txt` : prompt système entièrement personnalisable
- Format vocab : `水無月 士乃 (Minazuki Shino) - Female`

### Système de logs
- `log/translations.txt` : log courant
- `log/history/translationHistory_YYYYMMDD_HHMMSS.txt` : historique par run (10 derniers conservés)
- Lien symbolique legacy vers le dernier run

### Word wrap intelligent
- `dazedwrap.py` : wrapping adapté aux caractères japonais/CJK
- Largeur configurable par type de texte : `width`, `listWidth`, `noteWidth`

### GUI VSCode-style
- Sidebar icônes à gauche (60px de large)
- `QStackedWidget` pour les pages
- Dark theme CSS complet inline
- `QSettings` pour persistance fenêtre/état
- Thème : fond `#2b2b2b`, accent `#007acc`

---

## Points forts à reprendre pour NeoGlot

| Pattern | Détail | Applicabilité |
|---------|--------|---------------|
| **Placeholder protection** | Protège `\SE[...]`, `\BGM[...]` avant traduction | CRITIQUE — à reproduire exactement |
| **Mode estimation** | Compte les tokens/coût sans appeler l'API | Adapter pour Ollama (estimation de durée) |
| **Parse Speakers** | Extraction des noms avant traduction | À intégrer dans le module RPG Maker MV/MZ |
| **vocab.txt injection** | Glossaire injecté dans le prompt système | Notre "Module Glossaire" |
| **Cache de traduction** | Hash du texte source → traduction mise en cache | À stocker en SQLite dans NeoGlot |
| **Concurrence par fichier** | ThreadPoolExecutor + un worker par fichier | FastAPI + asyncio/BackgroundTasks |
| **Gestion des codes EVENT** | RPG Maker CODE 101/122/356 etc. | Reprendre la liste complète |
| **ruamel.yaml pour ACE** | Préserve les commentaires YAML | À utiliser pour RPG Maker ACE |

---

## Points faibles / problèmes identifiés

| Problème | Impact | Solution pour NeoGlot |
|----------|--------|----------------------|
| Pas d'IA locale | Cloud uniquement, coût réel, vie privée | Ollama obligatoire |
| Pas de base de données | Pas de persistance de projet, pas d'historique | SQLite avec projets |
| Config via `.env` | Peu robuste, pas de validation formelle | SQLite + validation API |
| Pas de preview inline | Pas de visualisation avant/après | Éditeur side-by-side dans l'UI |
| PyQt5 (desktop vieux) | UI datée, pas réactive | React + Tauri (moderne) |
| Pas de gestion de copies | Modifie directement les fichiers? | NeoGlot travaille sur copies |
| Globals Python massifs | `MODEL`, `LANGUAGE`, etc. au niveau module | FastAPI dependency injection |
| Pas de SQLite | Pas de suivi progression | NeoGlot : suivi par fichier/texte |
| `show_font_tip` MessageBox au démarrage | UX mauvaise | À ne pas reproduire |
| Windows-centric (`START.bat`) | Linux peu considéré | NeoGlot : Linux + Windows natif |

---

## Code RPG Maker MV/MZ — codes EVENT importants

Liste des codes gérés (à reprendre dans NeoGlot) :

| Code | Signification |
|------|---------------|
| 101 | Dialogue + nom du speaker |
| 102 | Choix (menu de sélection) |
| 122 | Variables de jeu (parfois du texte) |
| 356 | Plugin commands |
| 401 | Suite de dialogue (continuation CODE 101) |
| 405 | Suite de scrolling text |

---

## Dépendances Python clés

```
openai>=2.8.0          # Client API OpenAI/compatible
python-dotenv>=1.0.0   # Chargement .env
tiktoken>=0.8.0        # Comptage tokens OpenAI
tqdm>=4.65.0           # Barres de progression
colorama>=0.4.6        # Couleurs terminal
ruamel.yaml>=0.17.32   # YAML avec préservation commentaires (pour RPG Maker ACE)
pillow>=12.1.0         # Traitement images
PyQt5>=5.15.0          # Interface graphique
retry>=0.9.2           # Retry décorateur
```

---

## Résumé pour NeoGlot

DazedMTLTool est le projet de référence le plus mature. Sa couverture de moteurs est excellente et son système de placeholder protection est une **fonctionnalité critique** à reproduire. Ses principales limites sont :

1. **Cloud uniquement** → NeoGlot résout ça avec Ollama
2. **Pas de base de données** → NeoGlot résout ça avec SQLite + projets
3. **UI PyQt5 datée** → NeoGlot résout ça avec React + Tauri
4. **Gestion de fichiers manuelle** → NeoGlot résout ça avec UI dédiée

**Score de maturité** : 8/10 — excellente référence technique, stack à moderniser.
