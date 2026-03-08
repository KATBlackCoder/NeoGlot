// Modules de commandes Tauri — implémentés progressivement (T03 → T10)
pub mod db_commands; // T03 — CRUD projets, fichiers, strings
pub mod detect;      // T04 — détection moteur de jeu
pub mod translate;   // T06 — pipeline traduction Ollama + Channel
pub mod parse;       // T05 — extraction RPG Maker MV/MZ/XP/VX
pub mod write;       // T07 — réinjection RPG Maker
pub mod decrypt;     // T10 — déchiffrement .rgss*
pub mod glossary;    // T08 — CRUD glossaire
pub mod wolf;        // T09 — Wolf RPG (UberWolf + WolfTL subprocess)
