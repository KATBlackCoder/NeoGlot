# Analyse : SLR Translator

## Présentation générale

SLR Translator est l'outil le plus **feature-rich** des quatre analysés. C'est une application **NW.js** (Node-Webkit) avec code source JavaScript lisible, construite par l'auteur "Shisaye" (v2.083, licence GPL). Elle se distingue par son **architecture extensible par addons**, ses parsers de jeux multiples, et des outils tiers embarqués pour chaque moteur.

Elle embarque **Ruby, PHP et Python** comme runtimes complets pour ses parsers backend. C'est l'outil suisse de la traduction de jeux.

---

## Stack technique

| Couche | Technologie |
|--------|-------------|
| UI | NW.js (HTML/CSS/JS + jQuery) |
| Architecture | Système d'addons JS chargeables dynamiquement |
| Parsers backend | JavaScript + Ruby (embarqué) + PHP (embarqué) + Python 3.10 (embarqué) |
| Binary parsing | `kaitai-struct` (format déclaratif YAML→parser) |
| Éditeur de code | Ace Editor (intégré) |
| Translation cache | `localStorage` + fichiers `.trans` (format maison) |
| Build | npm |

---

## Architecture — Système d'Addons

C'est la caractéristique la plus distinctive de SLR. Chaque moteur de jeu est un **addon indépendant** chargé dynamiquement :

```
www/addons/
├── rmmv/          → RPG Maker MV (JSON)
├── rmmz/          → RPG Maker MZ (JSON)
├── rmrgss/        → RPG Maker XP/VX/VXAce (RGSS binaire)
├── jsonParser/    → JSON générique
├── ESParser/      → JavaScript/ES parser
├── customParser/  → Parsers custom définis par l'utilisateur
├── SLRbatch/      → Traitement en lot
├── SLRtrans/      → Intégration moteurs de traduction
├── tpp-localizationTool/ → Format TPP (Translator++)
├── aec/           → Moteur spécifique
├── transRikaikun/ → Dictionnaire japonais intégré
└── SEP/           → Script Event Parser
```

Chaque addon hérite de `Engine` (via `Engines.js`) et implémente des handlers :
```javascript
Engine.prototype.addHandler("onLoadTrans", handler);
Engine.prototype.addHandler("injectHandler", handler);   // écriture fichiers
Engine.prototype.addHandler("exportHandler", handler);   // export
Engine.prototype.addHandler("onOpenInjectDialog", handler);
```

---

## Format de fichier `.trans`

SLR utilise son propre format de projet `.trans` (+ `.tpp` pour Translator++ compat) qui stocke par ligne :

```javascript
transData = {
    data: [],          // textes source
    context: [],       // chemins de contexte (ex: "CommonEvent.dat/0/1/Message/0")
    tags: [],          // tags utilisateur
    parameters: [],    // paramètres RPG Maker (code event, acteur, etc.)
    indexIds: {}       // mapping index → ID
}
```

**Le système de contexte** (`TranslationByContext.js`) est unique : la même phrase source peut avoir plusieurs traductions différentes selon son chemin de contexte dans le jeu. Ex : `"はい"` traduit par `"Yes"` dans un dialogue mais `"Oui"` dans un menu.

---

## Liste complète des EVENT codes RPG Maker (rmmv.js)

La liste la plus complète observée dans tous les projets analysés :

| Code | Signification |
|------|---------------|
| 0 | Empty |
| 101 | Show Text Attributes |
| 102 | Show Choices |
| 103 | Input Number |
| 104 | Select Key Item |
| 105 | Show Scrolling Text Attributes |
| 108 | Comment |
| 111 | Conditional Branch |
| 112 | Loop |
| 115 | Exit Event Processing |
| 117 | Call Common Event |
| 118 | Label |
| 119 | Jump to Label |
| 121 | Control Switches |
| 122 | Control Variables |
| 123 | Control Self Switch |
| 124 | Control Timer |
| 125 | Change Gold |
| ... | (et beaucoup d'autres) |
| 356 | Plugin Commands |
| 401 | Suite Show Text |
| 405 | Suite Scrolling Text |

---

## Outils tiers embarqués (3rdParty/)

SLR embarque une collection remarquable d'outils pour chaque moteur :

| Outil | Rôle | Moteur |
|-------|------|--------|
| `RgssDecrypter/` | Déchiffre archives `.rgss` | RPG Maker XP/VX/VXAce |
| `rmxp_translator/` | Traduit fichiers RPG Maker XP | RPG Maker XP |
| `rpgmt_cli_v4.5/` | CLI pour RPG Maker Translator | RPG Maker |
| `wolfDec/` | Déchiffre fichiers Wolf RPG | Wolf RPG |
| `wolftrans-0.2.1/` | Traduit Wolf RPG | Wolf RPG |
| `WRPGE_2.24Z_EN/` | Wolf RPG Editor | Wolf RPG |
| `DXExtract/` | Extrait archives DX | DX Archive |
| `dxadecodedec/` | Decode/encode DXA | DXA |
| `MeCab/` | Analyseur morphologique japonais | IA / pre-processing |
| `atlas.exe` | ATLAS MT (vieux moteur JP→EN) | Traduction |
| `XUnity_AutoTranslator/` | Auto-translator Unity | Unity |
| `EnigmaVBUnpacker.exe` | Dépacker Enigma VB | Jeux compilés VB |
| `TES-Patcher/` | Patcher TES | TES games |
| `GnuWin32/` | Outils Unix portés Windows | Utilitaires |
| `crc32.exe`, `md5.exe` | Hash utilitaires | Vérification |

---

## Fonctionnalités avancées

### 1. CustomParser — Parsers définis par l'utilisateur

`www/addons/customParser/` permet à l'utilisateur de définir ses propres parsers via un éditeur de modèle (`modelEditor/`). Très puissant pour les formats non supportés nativement.

### 2. Automation Editor

`www/automationEditor.html` + `www/js/automationEditor.js` — éditeur d'automatisation pour créer des workflows de traduction complexes.

### 3. Code Editor (Ace)

Ace Editor intégré pour éditer directement les scripts de jeu et les configurations.

### 4. TranslationByContext

Chaque texte peut avoir **plusieurs traductions selon son contexte** (chemin dans le fichier source). Permet des traductions précises qui tiennent compte de l'usage :
```
"CommonEvent.dat/CommonEvent/0/Command/1/Message/0" → "Bonjour !"
"Map001.json/events/5/pages/0/list/3/parameters/0" → "Salut !"
```
(même texte source, traduction différente selon le contexte)

### 5. Traducteur overlay (translator.js)

Fenêtre popup séparée (toujours au dessus) qui monitore le **presse-papiers** et traduit les textes copiés. Permet de traduire du texte hors fichiers (OCR, etc.)

### 6. Speech Synthesis

`speech.js` + `synth.js` — synthèse vocale pour lire les textes traduits.

### 7. Rikaikun intégré

`transRikaikun/` — dictionnaire japonais intégré accessible pendant l'édition (survol d'un mot → définition).

### 8. Détection d'encodage automatique

`detect-file-encoding-and-language`, `jschardet`, `iconv-lite`, `encoding-japanese` — détection et conversion automatique des encodages (Shift-JIS, CP932, UTF-8, BOM, etc.)

### 9. Kaitai Struct — Binary parser déclaratif

Utilise `kaitai-struct` pour parser les formats binaires via des fichiers YAML déclaratifs. Très élégant pour supporter de nouveaux formats sans coder un parser from scratch.

---

## Moteurs de traduction supportés

| Moteur | Intégration |
|--------|------------|
| Google Translate | `trans.google.js` (reverse engineered) |
| Baidu | `baidu-translate-api` npm |
| `light-trans` | Bibliothèque custom de l'auteur |
| ATLAS | `3rdParty/atlas.exe` (legacy, offline) |
| Sugoi/KoboldAI | Via endpoint compatible |
| Tout service HTTP | Configurable via `TranslatorEngine` |

---

## Dépendances npm notables

| Package | Rôle |
|---------|------|
| `kaitai-struct` | Parsing binaire déclaratif |
| `renpy-js` + `rpy-parser` | Parsing Ren'Py |
| `encoding-japanese` + `iconv-lite` + `jschardet` | Détection/conversion encodage |
| `7zip-min` | Extraction archives 7zip |
| `pako` | Compression zlib/deflate |
| `acorn` + `acorn-loose` | Parsing JavaScript (pour ESParser) |
| `dragselect` | Sélection drag dans les tableaux |
| `jsonata` | Requêtes JSONata sur les données |
| `ws` | WebSocket (communication inter-fenêtres) |
| `kstg` | ? (package de l'auteur) |
| `better-localstorage` | Wrapper localStorage fiable |
| `buffer-tools` | Manipulation buffers binaires |

---

## Points forts à reprendre pour NeoGlot

| Pattern | Applicabilité |
|---------|--------------|
| **Architecture addon par moteur** | NeoGlot : chaque moteur = un module Python indépendant avec la même interface |
| **TranslationByContext** | À implémenter en SQLite : `(source, context_path) → translation` |
| **Liste EVENT codes RPG Maker** | Reprendre la liste complète de `rmmv.js` |
| **3rdParty wolfDec/wolftrans** | À utiliser ou à portager pour Wolf RPG dans NeoGlot |
| **3rdParty RgssDecrypter** | Alternative à `rpgmad-lib` pour les archives RGSS |
| **Kaitai Struct** | Pattern pour parser de nouveaux formats binaires sans coder from scratch |
| **Détection d'encodage** | NeoGlot doit gérer Shift-JIS/CP932/UTF-8 automatiquement |
| **`light-trans`** | À étudier pour voir si compatible Ollama |
| **Automation workflows** | Phase avancée NeoGlot — pipeline de traduction configurable |
| **Clipboard translator overlay** | Fonctionnalité bonus utile pour les playtests |

---

## Points faibles / différences avec NeoGlot

| Aspect | SLR | NeoGlot |
|--------|-----|---------|
| UI | HTML/jQuery vieillissant | React + shadcn/ui (moderne) |
| Performance | JavaScript pur, lent sur gros fichiers | Rust pour le parsing lourd |
| Windows-only | Oui (Ruby.exe, PHP.exe, 3rdParty .exe) | Linux + Windows natif |
| IA locale Ollama | Non supporté nativement | Obligatoire |
| Base de données structurée | localStorage + fichiers .trans | SQLite avec projets |
| Code maintenabilité | jQuery + globals massivement utilisés | Architecture moderne |
| Taille | Gigantesque (runtimes Ruby/PHP/Python embarqués) | Léger (Ollama externe) |

---

## Découverte importante : wolfDec + wolftrans

```
3rdParty/wolfDec/       → déchiffrement des archives Wolf RPG (.wolf)
3rdParty/wolftrans-0.2.1/ → extraction/réinjection textes Wolf RPG
```

Ces outils résolvent la partie Wolf RPG Editor qui était encore floue dans notre architecture. À étudier et potentiellement intégrer ou portager en Python/Rust.

---

## Résumé pour NeoGlot

SLR Translator est la **référence UX la plus complète** et la **source d'inspiration d'architecture la plus riche** de tous les projets. Son système d'addons, sa gestion du contexte de traduction, et sa collection d'outils tiers sont invaluables.

Ses limites techniques (JavaScript/jQuery, Windows-only, runtimes embarqués géants) sont exactement ce que NeoGlot résout avec sa stack moderne.

**Points critiques à retenir :**
1. Architecture **addon/plugin par moteur** — interface commune, implémentation indépendante
2. **Traduction par contexte** — même source, traduction différente selon le path
3. **wolfDec + wolftrans** — outils Wolf RPG à intégrer
4. **Liste complète EVENT codes RPG Maker** — la plus exhaustive vue
5. **Kaitai Struct** — pour futurs formats binaires inconnus

**Score de pertinence : 9/10** — mine d'or en termes de fonctionnalités et d'outils tiers, à moderniser côté stack.
