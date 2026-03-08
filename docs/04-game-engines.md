# Moteurs de Jeu Supportés — NeoGlot

## Tableau de support

| Moteur | Format | Extraction | Réinjection | Déchiffrement | Statut |
|--------|--------|-----------|------------|--------------|--------|
| RPG Maker MV | JSON | rvpacker-txt-rs-lib (Rust) | rvpacker-txt-rs-lib (Rust) | — | Priorité 1 |
| RPG Maker MZ | JSON | rvpacker-txt-rs-lib (Rust) | rvpacker-txt-rs-lib (Rust) | — | Priorité 1 |
| RPG Maker XP | Ruby Marshal | rvpacker-txt-rs-lib + marshal-rs (Rust) | rvpacker-txt-rs-lib (Rust) | rpgmad-lib (Rust) pour .rgss | Priorité 2 |
| RPG Maker VX | Ruby Marshal | rvpacker-txt-rs-lib + marshal-rs (Rust) | rvpacker-txt-rs-lib (Rust) | rpgmad-lib (Rust) pour .rgss2 | Priorité 2 |
| RPG Maker VXAce | Ruby Marshal | rvpacker-txt-rs-lib + marshal-rs (Rust) | rvpacker-txt-rs-lib (Rust) | rpgmad-lib (Rust) pour .rgss3 | Priorité 2 |
| Wolf RPG Editor | Binaire .dat/.mps | WolfTL --create (subprocess) | WolfTL --patch (subprocess) | UberWolfCli (subprocess) | Priorité 3 |
| RPG Bakin | TBD | TBD | TBD | — | Priorité 4 |

---

## RPG Maker MV / MZ

### Détection

```
game_path/
  www/data/           ← dossier data JSON (MV)
  data/               ← dossier data JSON (MZ)
  Game.exe
  package.json        ← contient "gameTitle"
```

### Fichiers contenant du texte

| Fichier | Contenu | EVENT codes |
|---------|---------|-------------|
| `Map001.json`...`MapXXX.json` | Dialogues, événements de carte | 101, 102, 401, 402 |
| `CommonEvents.json` | Événements communs (réutilisables) | 101, 102, 401, 402 |
| `Troops.json` | Textes de combat | 101, 401 |
| `Actors.json` | Noms et bios personnages | — (champs directs) |
| `Skills.json` | Noms et descriptions compétences | — |
| `Items.json` | Noms et descriptions objets | — |
| `Weapons.json` | Noms et descriptions armes | — |
| `Armors.json` | Noms et descriptions armures | — |
| `Enemies.json` | Noms et descriptions ennemis | — |
| `Classes.json` | Noms de classes | — |
| `System.json` | Textes système (menus, termes) | — |
| `GameTitle.json` | Titre du jeu | — |

### EVENT codes importants

| Code | Signification | Traduit |
|------|--------------|---------|
| 101 | Afficher boîte de dialogue (header : nom perso + visage) | OUI |
| 102 | Afficher choix | OUI |
| 122 | SetString (variable texte) | OUI (si contient du texte narratif) |
| 356 | Plugin command | NON (sauf exceptions Yanfly) |
| 401 | Ligne de dialogue (suite du 101) | OUI |
| 402 | Option de choix (suite du 102) | OUI |
| 405 | Show Scroll Text | OUI |
| 108 | Commentaire | NON |

### Validation + Formatage (intégré à l'extraction)

| Module | Rôle |
|--------|------|
| `engines/validation.rs` | `ContentValidator` — filtrage universel (vide, placeholders, IDs techniques, code JS) |
| `rpgmv/validation.rs` | `RpgMakerTextValidator` — filtrage spécifique (ponctuation, chemins, commandes script) |
| `engines/formatter.rs` | `UniversalFormatter` — patterns communs (`%n` → `[ARG_n]`, guillemets japonais, whitespace) |
| `rpgmv/formatter.rs` | `RpgMakerFormatter` — codes `\C[n]` → `[COLOR_n]`, `\N[n]` → `[NAME_n]`, etc. |

### Crates Rust

```toml
rvpacker-txt-rs-lib = "11.1.2"   # extraction + réinjection MV/MZ/XP/VX/VXAce
marshal-rs = "*"                  # parsing Ruby Marshal (XP/VX/VXAce binaire)
rpgmad-lib = "4.0.0"             # déchiffrement .rgss/.rgss2/.rgss3
```

---

## RPG Maker XP / VX / VXAce

### Détection

```
game_path/
  Data/
    Scripts.rxdata    ← XP (.rxdata)
    Scripts.rvdata    ← VX (.rvdata)
    Scripts.rvdata2   ← VXAce (.rvdata2)
  Game.rgss            ← archive XP
  Game.rgss2           ← archive VX
  Game.rgss3           ← archive VXAce
```

### Spécificités

- Les fichiers `.rvdata`/`.rxdata` sont au format **Ruby Marshal** (binaire)
- `rpgmad-lib` déchiffre l'archive `.rgss*` → extrait le dossier `Data/`
- `marshal-rs` parse les `.rvdata`/`.rxdata` → produit une structure JSON
- Ensuite, `rvpacker-txt-rs-lib` traite les fichiers comme MV/MZ

---

## Wolf RPG Editor

### Détection

```
game_path/
  Game.exe
  Data/
    BasicData/
      GameDat.wolf     ← fichier principal (crypté ou non)
      Database.wolf    ← base de données
    MapData/
      Map001.mps       ← cartes
    CommonEvent.wolf   ← événements communs
```

### Outils requis

| Outil | Source | Licence | Usage |
|-------|--------|---------|-------|
| **UberWolf** | github.com/Sinflower/UberWolf | MIT | Déchiffrement archives .wolf |
| **WolfTL** | github.com/Sinflower/WolfTL | MIT | Extraction + réinjection texte |

### Workflow Wolf RPG

```
1. UberWolfCli.exe game_path/ output_path/
   → lit la clé depuis Game.exe
   → déchiffre tous les *.wolf/.mps → dossiers déchiffrés

2. WolfTL --create game_path/ dump_path/
   → parse les binaires Wolf
   → produit un JSON par fichier dans dump_path/

3. [Traduction via Python + Ollama]

4. WolfTL --patch dump_path/ game_path/
   → ou WolfTL --patch dump_path/ game_path/ --inplace
   → réinjecte les textes traduits dans les fichiers binaires
```

### EVENT codes Wolf RPG traduits

| Code | Signification |
|------|--------------|
| 101 | ShowMessage (dialogue principal) |
| 102 | ShowChoices (choix) |
| 122 | SetStringVariable |
| 250 | CallDatabase (peut contenir texte affiché) |

### Validation + Formatage Wolf RPG

| Module | Rôle |
|--------|------|
| `wolf/validation.rs` | `WolfRpgTextValidator` — suppression itérative placeholders, chiffres seuls, debug `X[`, chemins `Data\` |
| `wolf/formatter.rs` | `WolfRpgFormatter` — codes `\E` → `[WOLF_END]`, `@n` → `[WOLF_VAR_n]`, `\cself[n]`, `\c[n]`/`\C[n]`, etc. |

### Problème Linux

UberWolf et WolfTL sont des `.exe` Windows uniquement.

**Solution retenue** : binaires placés dans `src-tauri/resources/` (bundlés via `"resources": ["resources/*"]` dans `tauri.conf.json`), exécutés via `wine <exe>` sur Linux et directement sur Windows. Wine est une dépendance à documenter dans le README.

**Alternative long terme** : portage Rust natif (sources MIT disponibles).

### Versions WolfPro supportées (UberWolf)

| Version | Algorithme | Support |
|---------|-----------|---------|
| v1.0-2.2 | DES/AES | OUI |
| v2.3-3.4 | NewWolfCrypt | OUI |
| v3.5+ | ChaCha20 + WolfX | OUI (UberWolf 0.6.2+) |

---

## RPG Bakin

### Détection

```
game_path/
  *.bakin              ← fichier projet Bakin
  data/
    projects/
      scenario/        ← dialogues en XML/JSON
```

### Statut

Format partiellement documenté. Priorité 4. À analyser après les autres moteurs.

**Sources potentielles** :
- MTool supporte Bakin (source fermée, non réutilisable)
- Analyse manuelle du format `.bakin`
- Potentiellement XML/JSON compatible avec un parser custom

---

## Détection automatique (Rust)

```rust
// commands/detect.rs
#[tauri::command]
pub fn detect_engine(game_path: &str) -> Result<String, String> {
    let path = Path::new(game_path);

    // RPG Maker MZ
    if path.join("data").join("System.json").exists() {
        return Ok("rpgmz".into());
    }
    // RPG Maker MV
    if path.join("www").join("data").join("System.json").exists() {
        return Ok("rpgmv".into());
    }
    // RPG Maker VXAce
    if path.join("Data").join("Scripts.rvdata2").exists() {
        return Ok("rpgmvxa".into());
    }
    // RPG Maker VX
    if path.join("Data").join("Scripts.rvdata").exists() {
        return Ok("rpgmvx".into());
    }
    // RPG Maker XP
    if path.join("Data").join("Scripts.rxdata").exists() {
        return Ok("rpgmxp".into());
    }
    // Wolf RPG
    if path.join("Data").join("BasicData").join("GameDat.wolf").exists() {
        return Ok("wolf".into());
    }
    // RPG Bakin
    if walkdir::WalkDir::new(path)
        .max_depth(1)
        .into_iter()
        .any(|e| e.ok().map(|e| e.path().extension() == Some("bakin".as_ref())).unwrap_or(false))
    {
        return Ok("bakin".into());
    }

    Err("Moteur non reconnu".into())
}
```
