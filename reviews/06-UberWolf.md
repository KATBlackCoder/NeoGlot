# Analyse : UberWolf

**Repo** : https://github.com/Sinflower/UberWolf
**Auteur** : Sinflower (même auteur que WolfTL)
**Licence** : MIT
**Langage** : C++ (98%) + C (2%)
**Version** : v0.6.2

---

## Présentation générale

UberWolf est le **déchiffreur/dépackeur** de référence pour Wolf RPG Editor — le complément indispensable de WolfTL. Là où WolfTL parse et traduit, UberWolf **décrypte les archives** (.wolf, .data, .pak, .bin, etc.) pour les rendre accessibles.

Il existe en deux variantes : **GUI** (drag-and-drop) et **CLI** (`UberWolfCli`). La bibliothèque partagée `UberWolfLib` contient toute la logique.

---

## Stack technique

| Couche | Technologie |
|--------|-------------|
| Langage | C++17+ |
| Build | Visual Studio (.sln) |
| GUI | Win32/MFC ou Qt (non précisé, Windows) |
| CLI | `UberWolfCli.exe` |
| Bibliothèque | `UberWolfLib` (static/shared) |

---

## Architecture

```
UberWolf/
├── UberWolf/        → GUI application
├── UberWolfCli/     → CLI tool
├── UberWolfLib/     → Bibliothèque core
│   ├── WolfDec.cpp/h          → Décodeur principal
│   ├── Wolf35Unprotect.cpp/hpp → Déprotection Wolf v3.5
│   ├── WolfPro.cpp/h          → Support Wolf RPG Pro
│   ├── WolfXWrapper.cpp/h     → Wrapper WolfX (v3.5)
│   ├── WolfUtils.cpp/h        → Utilitaires
│   ├── UberLog.cpp/h          → Logging
│   ├── Localizer.cpp/h        → i18n (JA/KO/ZH/EN)
│   ├── WolfSha512.hpp         → SHA-512
│   ├── Defines.h              → Constantes
│   ├── Types.h                → Types de données
│   ├── WolfRPG/               → Parsers fichiers Wolf RPG
│   └── WolfX/                 → Support WolfX (nouveau format v3.5)
└── doc/             → Documentation + guides localisation
```

---

## Formats de fichiers supportés

UberWolf déchiffre/extrait les archives dans ces formats :

| Extension | Usage |
|-----------|-------|
| `.wolf` | Archive principale Wolf RPG |
| `.data` | Données binaires Wolf RPG |
| `.pak` | Pakage de ressources |
| `.bin` | Fichiers binaires génériques |
| `.assets` | Unity assets (bonus) |
| `.content` | Contenu packagé |
| `.res` | Ressources |
| `.resource` | Ressources étendues |

---

## Historique des versions — évolution du chiffrement Wolf RPG

L'évolution des releases raconte l'histoire des versions de chiffrement :

| Version UberWolf | Date | Changement clé |
|-----------------|------|----------------|
| v0.1.0 | Nov 2022 | Release initiale |
| v0.2.0 | Avr 2023 | Suppression protection Pro, localisation |
| v0.3.0 | Jun 2023 | Déchiffrement v3.31+ |
| v0.3.1 | Jun 2023 | Localisation JA/KO |
| v0.4.1 | Déc 2023 | Migration 32→64 bit, packing complet |
| v0.5.0 | Mai 2024 | **Wolf RPG v3.5 complet + WolfX + ChaCha20** |
| v0.6.0 | Aoû 2024 | Anti-unpacking Wolf v3.5, AVX removed |
| v0.6.1 | Fév 2025 | Fix crash Pro files |
| v0.6.2 | Mar 2025 | Améliorations parser + WolfX cracker fixes |

**Leçon clé** : Wolf RPG a changé son système de chiffrement à chaque version majeure. La v3.5 introduit **ChaCha20** et **WolfX**, les plus récents.

---

## Systèmes de chiffrement Wolf RPG

| Version Wolf RPG | Chiffrement | Composant UberWolf |
|-----------------|-------------|-------------------|
| Standard (< 3.1) | Clé XOR simple | `WolfDec` |
| v3.1 – v3.31 | Chiffrement propriétaire | `WolfDec` + `WolfUtils` |
| v3.31+ | Chiffrement renforcé | `Wolf35Unprotect` |
| v3.5 | **ChaCha20** + WolfX | `WolfPro` + `WolfXWrapper` |
| Wolf RPG Pro | Protection Pro (clé custom) | `WolfPro` + détection auto |

---

## Modes d'utilisation CLI

```bash
# Option 1 — Détection automatique depuis l'exécutable du jeu
UberWolfCli.exe "D:\Path to Game\Game.exe"
UberWolfCli.exe "D:\Path to Game\GamePro.exe"

# Option 2 — Dossier complet (extrait tous les fichiers supportés)
UberWolfCli.exe "D:\Path to Game\"

# Option 3 — Fichier individuel
UberWolfCli.exe "D:\Path to Game\Data\BasicData.wolf"
```

**Détection automatique de la clé** : quand on passe `Game.exe`, UberWolf extrait la clé de déchiffrement directement depuis l'exécutable.

---

## Intégration pour NeoGlot

### Workflow Wolf RPG complet avec UberWolf + WolfTL

```python
# backend/engines/wolf.py

import subprocess
from pathlib import Path

class WolfRPGEngine:
    """Module Wolf RPG Editor pour NeoGlot"""

    def __init__(self, uber_wolf_cli: str, wolf_tl: str):
        self.uber_wolf = uber_wolf_cli  # chemin vers UberWolfCli.exe
        self.wolf_tl = wolf_tl          # chemin vers WolfTL.exe

    def decrypt(self, game_exe: str, output_dir: str) -> None:
        """Étape 1 : déchiffrer les archives Wolf RPG"""
        subprocess.run([
            self.uber_wolf, game_exe
        ], cwd=output_dir, check=True)

    def extract(self, data_path: str, dump_path: str) -> None:
        """Étape 2 : extraire les textes vers JSON"""
        subprocess.run([
            self.wolf_tl, data_path, dump_path, "--create"
        ], check=True)

    def inject(self, data_path: str, dump_path: str) -> None:
        """Étape 3 : réinjecter les traductions"""
        subprocess.run([
            self.wolf_tl, data_path, dump_path, "--patch"
        ], check=True)
```

### Workflow complet

```
Game.exe / GamePro.exe
        ↓ UberWolfCli (déchiffrement)
Data/*.wolf → Data/*.dat + *.mps (déchiffrés)
        ↓ WolfTL --create
dump/mps/*.json + dump/common/*.json + dump/db/*.json
        ↓ NeoGlot (traduction Ollama)
dump/mps/*.json (traduits)
        ↓ WolfTL --patch
patched/data/ (binaires avec traduction)
```

---

## Considérations Linux

UberWolf et WolfTL sont des exécutables Windows (`.exe`). Pour NeoGlot (Linux + Windows) :

**Option A — Wine** (Linux) :
```python
if platform == "linux":
    cmd = ["wine", self.uber_wolf, game_exe]
else:
    cmd = [self.uber_wolf, game_exe]
```

**Option B — Portage Rust** :
Le code source C++ est MIT et bien structuré. Créer une crate `uberwolf-rs` + `wolftl-rs` en reprenant les algorithmes ChaCha20 (crate `chacha20` disponible sur crates.io).

**Option C — Bundler les .exe sous Wine** :
Pour les utilisateurs Linux, bundler les `.exe` et les exécuter via Wine transparent.

**Recommandation** : Option B (portage Rust) pour une solution vraiment cross-platform, en reprenant les algorithmes depuis les sources MIT. Option A comme fallback rapide.

---

## Relation avec WolfTL

Les deux projets partagent :
- Le même auteur (Sinflower)
- Le dossier `WolfRPG/` avec les mêmes headers (légèrement différents)
- La même logique de déchiffrement (`Wolf35Unprotect`, `WolfSha512`)
- La licence MIT

**UberWolf = déchiffrement d'archives** → **WolfTL = parsing/traduction des fichiers**

Ce sont deux outils complémentaires à utiliser ensemble.

---

## Score de pertinence pour NeoGlot : 10/10

Indispensable pour les jeux Wolf RPG Pro et v3.5 (qui représentent la majorité des jeux récents). Sans UberWolf, les archives `.wolf` chiffrées sont inaccessibles. À intégrer comme étape de pré-traitement avant WolfTL.
