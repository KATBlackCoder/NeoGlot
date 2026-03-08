// Détection automatique du moteur de jeu par inspection des fichiers marqueurs
// Implémenté en T04
use walkdir::WalkDir;

#[tauri::command]
pub fn detect_engine(game_path: String) -> Result<String, String> {
    let path = std::path::Path::new(&game_path);

    if !path.exists() {
        return Err("Dossier introuvable".into());
    }

    // RPG Maker MZ (data/System.json à la racine)
    if path.join("data").join("System.json").exists() {
        return Ok("rpgmz".into());
    }
    // RPG Maker MV (www/data/System.json)
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
    // Wolf RPG (GameDat.wolf ou GameDat.dat dans Data/BasicData/)
    if path.join("Data").join("BasicData").join("GameDat.wolf").exists()
        || path.join("Data").join("BasicData").join("GameDat.dat").exists()
    {
        return Ok("wolf".into());
    }
    // RPG Bakin (fichier .bakin à la racine)
    for entry in WalkDir::new(path).max_depth(1).into_iter().flatten() {
        if entry.path().extension().map(|e| e == "bakin").unwrap_or(false) {
            return Ok("bakin".into());
        }
    }

    Err("Moteur non reconnu. Vérifiez que c'est bien le dossier racine du jeu.".into())
}
