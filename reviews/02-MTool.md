# Analyse : MTool

## Présentation générale

MTool est un outil de traduction de jeux **radicalement différent** de DazedMTLTool. Il ne modifie pas les fichiers du jeu : il **s'injecte dans le processus du jeu en cours d'exécution** et remplace le texte en mémoire en temps réel. L'approche est celle d'un **hook dynamique** plutôt que d'une traduction statique de fichiers.

C'est un outil **closed-source** (code compilé en bytecode obfusqué). Origine : projet chinois (interface en chinois/japonais/anglais), très populaire dans la communauté de traduction de jeux japonais vers le chinois.

---

## Stack technique

| Couche | Technologie |
|--------|-------------|
| UI | NW.js (Node-Webkit) — HTML/CSS/JS |
| Logique applicative | JavaScript compilé en bytecode (bytenode — `.bin`) |
| Hooks natifs | DLL injection (.dll) via processus Windows |
| Communication | WebSocket (`ws`) |
| Encodage | `iconv-lite`, `jconv` (japonais CJK) |
| PE editing | `pe-library` + `resedit` (patch .exe) |
| Archivage | `adm-zip` |
| Mascotte | Cirno (personnage Touhou) |

**Point critique** : le code source réel est dans `www/loader.bin.*` — des fichiers binaires bytecode V8 compilés via `bytenode`. Impossible à lire/auditer directement.

---

## Architecture — Approche "hooking temps réel"

### Principe fondamental

```
┌─────────────┐     DLL injection      ┌─────────────────────┐
│    MTool    │ ──────────────────────> │  Processus du jeu   │
│  (NW.js UI) │ <── texte intercepté── │  (RPG Maker, Wolf…) │
│             │ ──── traduction ──────> │  (texte remplacé)   │
└─────────────┘                        └─────────────────────┘
```

1. L'utilisateur ouvre MTool
2. MTool attend qu'un jeu compatible se lance
3. À la détection du processus, MTool injecte un hook DLL dans le jeu
4. Le hook intercepte les appels texte natifs (rendu de dialogue, menus, etc.)
5. MTool traduit le texte via son moteur choisi
6. Le texte traduit est réinjecté en mémoire avant l'affichage

**Aucune modification de fichiers** — tout se passe en RAM.

### DLLs de hooks par moteur

| DLL | Moteur ciblé |
|-----|-------------|
| `mzHook.dll` / `mzHook32.dll` | RPG Maker MZ |
| `RGSSHook.dll` / `RGSSHook64.dll` | RPG Maker XP/VX/VXAce (Ruby RGSS) |
| `wolfHook.dll` / `wolfHook3.dll` | Wolf RPG Editor |
| `krkrzHook32.dll` / `krkrzHook64.dll` | Kirikiri |
| `AgtkHook.dll` | Agiletortoise (engine indie) |
| `MonoJunkiex64.dll` / `MonoJunkiex86.dll` | Unity (Mono runtime) |
| `PythonHook64.dll` / `PythonHook.dll` | Engines Python (Ren'Py?) |
| `SRPGHook.dll` | SRPG Studio |
| `BakinLauncher.exe` / `BakinPlayerWrapper.exe` | RPG Bakin |
| `kmyHook.exe` | Inconnu (hook générique?) |
| `PIDDLLInject64.exe` | Injecteur générique 64-bit |
| `ele64.exe` | Electron wrapper |

### `www/data/` — Cache de traductions

Les ~250 fichiers nommés avec des hashes numériques (`1002938825`, `-297262159`, etc.) semblent être un **cache de traductions pré-enregistrées** ou des **données de jeu indexées par CRC**. Le fichier `crcData` et `updateCRC` à la racine confirment l'usage d'un système CRC pour identifier les textes.

---

## Moteurs de traduction supportés

| Moteur | Type | Niveau requis |
|--------|------|---------------|
| TrsNT | Interne MTool (IA locale?) | lv.2+ |
| DeepL | Cloud | lv.1+ |
| Bing | Cloud | lv.1 |
| Google | Cloud | Suspendu |
| ChatGPT 3.5 | Cloud API | lv.2+ |
| ChatGPT 5 Mini | Cloud API | lv.3 (test) |
| DeepSeek | Cloud API | lv.1+ |
| Llama3 (mod) | Local | lv.1+ (test) |
| Sugoi | Local (modèle japonais) | lv.1 |
| Sugoi 32B | Local | lv.2+ (test) |
| Baidu | Cloud CN | lv.2+, JP→ZH seulement |
| Tencent | Cloud CN | lv.1+ |
| Sogou | Cloud CN | lv.1+ |
| iCIBA | Cloud CN | lv.1+ |
| LibreTranslate | Self-hosted | lv.1 |
| Caiyun (彩云) | Cloud CN | lv.1+, JP→ZH seulement |
| Claude 3 Haiku | Cloud | Actuellement indisponible |

**Système de niveaux (lv)** : MTool a un système de compte/niveau qui limite l'accès à certains moteurs premium.

---

## Fonctionnalités notables

### Multi-langues UI
`LangTL.json` contient les traductions de l'interface en japonais, chinois, anglais au minimum. L'UI est entièrement internationalisée.

### PE editing (patch d'exécutables)
Les packages `pe-library` + `resedit` permettent de patcher directement les `.exe` des jeux — probablement pour changer l'encodage (UTF-8) ou insérer des polices japonaises/CJK.

### fontSetting.exe
Outil séparé pour la configuration de polices — important pour l'affichage des caractères traduits dans les jeux qui utilisent des polices d'icônes ou bitmap fonts.

### `gecko_pref.yaml`
Fichier de préférences pour le moteur de rendu — indique une gestion fine du rendu WebView.

### Icône / mascotte Cirno
Utilise Cirno (personnage de Touhou Project) comme mascotte avec 11 expressions faciales et animations. Renforce l'identité communautaire de l'outil.

---

## Points forts

| Aspect | Valeur |
|--------|--------|
| **Zéro modification de fichiers** | Idéal pour jeux avec archives cryptées/packées |
| **Fonctionne sur tout jeu compatible** | Pas besoin de connaître le format de fichiers |
| **Traduction immédiate** | Voir la traduction pendant le jeu |
| **Aucune réinjection** | Pas de risque de corrompre les fichiers |
| **Support RPG Bakin natif** | Rare — très peu d'outils le supportent |
| **Cache de traductions** | Réutilise les traductions précédentes |

---

## Points faibles / limitations

| Problème | Impact |
|----------|--------|
| **Windows uniquement** | DLL injection = Windows exclusif |
| **Closed-source obfusqué** | Impossible à auditer, dépendance totale à l'auteur |
| **Latence de traduction** | Délai visible pendant le jeu (attente API) |
| **Système de niveaux payant** | Certains moteurs bloqués derrière un compte |
| **Pas de traduction offline batch** | Impossible de préparer une traduction à l'avance |
| **Pas de base de données projet** | Pas de suivi de progression par jeu |
| **Pas de contrôle qualité** | Aucun moyen d'éditer/corriger les traductions |
| **Dépend du jeu actif** | Ne fonctionne que pendant l'exécution du jeu |

---

## Pertinence pour NeoGlot

### Approche incompatible

L'approche hooking temps réel de MTool est **fondamentalement incompatible** avec l'objectif de NeoGlot (traduction statique de fichiers + réinjection). Ce sont deux philosophies opposées :

| | MTool | NeoGlot |
|--|-------|---------|
| Moment de traduction | Pendant le jeu (runtime) | Avant le jeu (pré-traduction) |
| Modification fichiers | Jamais | Toujours (sur copies) |
| Qualité contrôlable | Non | Oui |
| Hors ligne possible | Limité | Oui (Ollama) |
| Linux compatible | Non | Oui |

### Ce qu'on peut retenir

| Pattern | Application pour NeoGlot |
|---------|--------------------------|
| **Cache par CRC/hash** | Déjà prévu dans NeoGlot via SQLite |
| **Support RPG Bakin** | Confirme la demande pour ce moteur |
| **fontSetting** | NeoGlot pourrait proposer un guide d'installation de polices |
| **Liste des DLL hooks** | Donne la liste exhaustive des **fichiers à hooker par moteur** (à ne pas utiliser en DLL injection, mais utile pour identifier les points d'entrée des formats) |
| **Encodages multiples** | `iconv-lite`, `jconv` — NeoGlot doit gérer UTF-8, Shift-JIS, CP932 |

---

## Résumé pour NeoGlot

MTool est une **référence UX** (traduction transparente, zéro friction) mais une **anti-référence technique** pour NeoGlot : son approche hooking est Windows-only, closed-source, et ne permet pas de contrôle qualité.

Son principal apport est de **confirmer la liste des moteurs à supporter** et de montrer que RPG Bakin et les hooks Unity/Mono sont techniquement faisables.

**Score pertinence** : 3/10 — architecture incompatible, mais bonne source d'information sur les moteurs de jeux.
