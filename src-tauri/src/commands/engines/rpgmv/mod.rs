// RPG Maker MV/MZ — fichiers JSON traités via rvpacker-txt-rs-lib
pub mod extract;    // T05 — extraction textes + speakers
pub mod inject;     // T07 — réinjection traductions
pub mod formatter;  // Codes moteur → placeholders AI (\C, \N, \V, \I, etc.)
pub mod validation; // Filtrage textes non-traduisibles (ponctuation, chemins, etc.)
