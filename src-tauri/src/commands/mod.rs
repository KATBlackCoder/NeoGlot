// Modules de commandes Tauri — organisés par domaine et moteur de jeu
pub mod db_commands; // T03 — CRUD projets, fichiers, strings
pub mod detect;      // T04 — détection moteur de jeu
pub mod translate;   // T06 — pipeline traduction Ollama + Channel
pub mod glossary;    // T08 — CRUD glossaire

// Modules par moteur de jeu (extract + inject isolés par engine)
pub mod engines;
