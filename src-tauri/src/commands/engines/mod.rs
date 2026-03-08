// Modules par moteur de jeu — chacun isole extraction + réinjection
pub mod formatter;    // Trait EngineFormatter + UniversalFormatter (partagé)
pub mod validation;   // ContentValidator (validation universelle partagée)
pub mod rpgmv;        // T05/T07 — RPG Maker MV/MZ (JSON via rvpacker-txt-rs-lib)
pub mod rpgm_classic; // T10     — RPG Maker XP/VX/VXAce (marshal-rs + rpgmad-lib)
pub mod wolf;         // T09     — Wolf RPG Editor (WolfTL + UberWolf subprocess)
