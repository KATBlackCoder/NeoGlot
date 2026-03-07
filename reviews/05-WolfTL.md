# Analyse : WolfTL

**Repo** : https://github.com/Sinflower/WolfTL
**Auteur** : Sinflower
**Licence** : MIT
**Langage** : C++ (98.9%) — header-only pour les parsers

---

## Présentation générale

WolfTL est un outil CLI en **C++** qui extrait les textes traduisibles des fichiers binaires Wolf RPG Editor (`.dat`, `.mps`) vers du **JSON**, et les réinjecte. C'est la référence technique la plus propre pour le format Wolf RPG.

Son code de parsing est basé sur **Wolf Trans** (le projet original Ruby), réécrit en C++ avec support chiffrement WolfPro.

---

## Stack technique

| Couche | Technologie |
|--------|-------------|
| Langage | C++17+ |
| Build | CMake + Visual Studio |
| Format sortie | JSON |
| Chiffrement | SHA-512 + NewWolfCrypt + ChaCha20 (WolfPro) |

---

## Architecture — WolfRPG Headers

Tout le parsing est dans `WolfTL/WolfRPG/` — une bibliothèque header-only en C++ :

```
WolfRPG/
├── WolfRPG.hpp         → Classe principale (charge tout le jeu)
├── GameDat.hpp         → Parsing Game.dat
├── Map.hpp             → Parsing fichiers .mps (Maps)
├── CommonEvents.hpp    → Parsing CommonEvent.dat
├── Database.hpp        → Parsing bases de données .db/.project
├── WolfDataBase.hpp    → Abstraction base de données Wolf
├── Command.hpp         → Enum CommandType (tous les codes événements)
├── RouteCommand.hpp    → Commandes de mouvement
├── FileCoder.hpp       → Encodage/décodage fichiers Wolf
├── FileAccess.hpp      → Accès fichiers bas niveau
├── NewWolfCrypt.hpp    → Chiffrement Wolf RPG récent
├── StringConv.hpp      → Conversion chaînes (Shift-JIS ↔ UTF-8)
├── Types.hpp           → Types de données Wolf RPG
├── WolfRPGUtils.hpp    → Utilitaires
└── WolfRPGException.hpp → Gestion d'erreurs
```

### Classe principale `WolfRPG`

```cpp
WolfRPG(path, skip_game_dat, save_uncompressed)
    → loadGameDat()          // Game.dat
    → loadMaps()             // tous les .mps récursivement
    → loadCommonEvents()     // CommonEvent.dat
    → loadDatabases()        // .db + .project
```

---

## Modes d'opération (CLI)

```bash
WolfTL.exe DATA_PATH OUTPUT_PATH --create    # binaire → JSON dump
WolfTL.exe DATA_PATH OUTPUT_PATH --patch     # JSON → fichiers patched/
WolfTL.exe DATA_PATH OUTPUT_PATH --patch --inplace  # JSON → fichiers originaux (in-place)

# Options
--skip-game_dat    # ignore Game.dat (parfois corrompu ou inutile)
-s, --save_uncompressed  # export non compressé pour debug
```

### Structure des fichiers JSON générés

```
dump/
├── mps/           → Maps (*.mps → *.json)
├── db/            → Databases (*.db → *.json)
└── common/        → CommonEvents (CommonEvent.dat → JSON)
    GameDat.json   → Game.dat

patched/
└── data/          → fichiers binaires mis à jour
```

---

## Liste complète des codes EVENT Wolf RPG (`Command.hpp`)

La liste la plus complète et précise pour Wolf RPG :

| Code | Nom |
|------|-----|
| 0 | Blank |
| 99 | Checkpoint |
| 101 | **Message** ← texte dialogue |
| 102 | **Choices** ← options de choix |
| 103 | Comment |
| 105 | ForceStopMessage |
| 106 | DebugMessage |
| 107 | ClearDebugText |
| 111 | VariableCondition |
| 112 | StringCondition |
| 121 | **SetVariable** ← variables (parfois texte) |
| 122 | **SetString** ← chaînes (souvent texte) |
| 123 | InputKey |
| 124 | SetVariableEx |
| 125 | AutoInput |
| 126 | BanInput |
| 130 | Teleport |
| 140 | Sound |
| 150 | Picture |
| 151 | ChangeColor |
| 160 | SetTransition |
| 161 | PrepareTransition |
| 162 | ExecuteTransition |
| 170 | StartLoop |
| 171 | BreakLoop |
| 172 | BreakEvent |
| 174 | ReturnToTitle |
| 175 | EndGame |
| 176 | StartLoop2 |
| 201 | Move |
| 202 | WaitForMove |
| 210 | **CommonEvent** |
| 211 | CommonEventReserve |
| 212 | SetLabel |
| 213 | JumpLabel |
| 220 | SaveLoad |
| 221 | LoadGame |
| 222 | SaveGame |
| 230 | MoveDuringEventOn |
| 231 | MoveDuringEventOff |
| 240 | Chip |
| 241 | ChipSet |
| 250 | **Database** ← accès BDD (peut contenir texte) |
| 251 | ImportDatabase |
| 270 | Party |
| 280 | MapEffect |
| 281 | ScrollScreen |
| 290 | Effect |
| 300 | **CommonEventByName** |
| 401 | ChoiceCase |
| 402 | SpecialChoiceCase |
| 420 | ElseCase |
| 421 | CancelCase |
| 498 | LoopEnd |
| 499 | BranchEnd |
| 1000 | **ProFeature** ← Wolf RPG Pro uniquement |

**Codes traduisibles** : 101 (Message), 102 (Choices), 122 (SetString), 250 (Database text).

---

## Support WolfPro

Wolf RPG Pro chiffre les fichiers avec :
- **SHA-512** (`WolfSha512.hpp`) pour vérification d'intégrité
- **NewWolfCrypt** (`NewWolfCrypt.hpp`) — chiffrement propriétaire Wolf RPG récent
- **ChaCha20** — v3.5+ (voir UberWolf)
- **Wolf35Unprotect** (`Wolf35Unprotect.hpp`) — déprotection spécifique v3.5

Après patch, les fichiers sont **non-chiffrés** et ouvrable dans tout éditeur Wolf RPG.

---

## Intégration pour NeoGlot

### Option 1 — Appel CLI direct (recommandée)

```python
# backend/engines/wolf.py
import subprocess

def extract_wolf(data_path: str, output_path: str) -> dict:
    """Extrait les textes Wolf RPG vers JSON via WolfTL CLI"""
    subprocess.run([
        "WolfTL", data_path, output_path, "--create"
    ], check=True)
    # Lire les JSON dans output_path/dump/
    return parse_wolf_json_dump(output_path)

def inject_wolf(data_path: str, dump_path: str, inplace=False) -> None:
    """Réinjecte les traductions via WolfTL CLI"""
    args = ["WolfTL", data_path, dump_path, "--patch"]
    if inplace:
        args.append("--inplace")
    subprocess.run(args, check=True)
```

### Option 2 — Portager les headers C++ en Rust

Les 15 headers `.hpp` sont bien structurés et portables. On pourrait créer une crate `wolfrpg-rs` en s'en inspirant directement.

### Fichiers à traduire

| Fichier | Contenu | Codes EVENT |
|---------|---------|------------|
| `*.mps` | Cartes + événements | 101, 102, 122 |
| `CommonEvent.dat` | Événements communs | 101, 102, 122 |
| `*.db` + `*.project` | Bases de données | textes dans les champs |
| `Game.dat` | Config globale (noms, titres) | textes fixes |

---

## Avantages vs outils existants

| vs wolfDec (dans SLR) | WolfTL est plus récent, supporte WolfPro v3.5, produit du JSON exploitable |
| vs wolftrans (dans SLR) | WolfTL est maintenu activement, même approche mais en C++ moderne |
| vs wolf.py (DazedMTL) | WolfTL est plus robuste, gère les archives chiffrées |

---

## Score de pertinence pour NeoGlot : 10/10

C'est **LA référence** pour Wolf RPG. Header-only C++, MIT, bien documenté, support WolfPro. À intégrer via CLI ou à portager en Rust.
