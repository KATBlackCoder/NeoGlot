// RPG Maker XP/VX/VXAce — archives .rgss* (marshal-rs + rpgmad-lib)
pub mod extract;    // T10 — extraction textes depuis Ruby Marshal
pub mod decrypt;    // T10 — déchiffrement archives .rgss*
pub mod formatter;  // Réutilise RpgMakerFormatter (mêmes codes moteur)
pub mod validation; // Réutilise RpgMakerTextValidator (mêmes règles)
