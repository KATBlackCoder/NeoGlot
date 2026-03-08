// Formatter RPG Maker MV/MZ — convertit les codes moteur en placeholders AI-friendly
use crate::commands::engines::formatter::{EngineFormatter, UniversalFormatter};
use once_cell::sync::Lazy;
use regex::Regex;

// ═══ Regexes RPG Maker (pré-compilées) ═══

static COLOR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\C\[(\d+)\]").unwrap());
static COLOR_REGEX_LOWER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\c\[(\d+)\]").unwrap());
static NAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\N\[(\d+)\]").unwrap());
static NEWLINE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\n\[(\d+)\]").unwrap());
static CONDITIONAL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"en\(v\[(\d+)\]>(\d+)\)").unwrap());
static F_CODE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\\F([A-Za-z0-9]*)\[(\d+)\]").unwrap());
static AA_CODE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\AA\[(\d+)\]").unwrap());
static CLOSE_BRACE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\}").unwrap());

static COLOR_RESTORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[COLOR_(\d+)\]").unwrap());
static NAME_RESTORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[NAME_(\d+)\]").unwrap());
static NEWLINE_RESTORE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[NEWLINE_(\d+)\]").unwrap());
static CONDITIONAL_RESTORE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[CONDITIONAL_v(\d+)>(\d+)\]").unwrap());
static F_CODE_RESTORE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[F_([A-Za-z0-9]*)_(\d+)\]|\[F_(\d+)\]").unwrap());
static AA_CODE_RESTORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[AA_(\d+)\]").unwrap());
static CLOSE_BRACE_RESTORE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[CLOSE_BRACE\]").unwrap());

/// Formatter spécifique RPG Maker MV/MZ.
/// Gère les codes \C, \N, \V, \I, \W, \A, \P, \G, \$, \F, \AA, etc.
pub struct RpgMakerFormatter;

impl EngineFormatter for RpgMakerFormatter {
    fn prepare_for_translation(text: &str) -> String {
        if !Self::has_formatting_codes(text) {
            return text.to_string();
        }

        let mut result = text.to_string();

        // Codes couleur \C[n] et \c[n]
        result = COLOR_REGEX.replace_all(&result, "[COLOR_$1]").to_string();
        result = COLOR_REGEX_LOWER.replace_all(&result, "[COLOR_$1]").to_string();
        result = result.replace("\\C", "[COLOR_SIMPLE]");

        // Noms \N[n], retours \n[n]
        result = NAME_REGEX.replace_all(&result, "[NAME_$1]").to_string();
        result = NEWLINE_REGEX.replace_all(&result, "[NEWLINE_$1]").to_string();

        // Codes F (polices)
        result = F_CODE_REGEX
            .replace_all(&result, |caps: &regex::Captures| {
                let letters = caps.get(1).map_or("", |m| m.as_str());
                let number = caps.get(2).map_or("", |m| m.as_str());
                if letters.is_empty() {
                    format!("[F_{}]", number)
                } else {
                    format!("[F_{}_{}]", letters, number)
                }
            })
            .to_string();

        result = AA_CODE_REGEX.replace_all(&result, "[AA_$1]").to_string();
        result = CLOSE_BRACE_REGEX.replace_all(&result, "[CLOSE_BRACE]").to_string();

        // Codes simples (remplacement de chaînes)
        result = result.replace("\\V[", "[VARIABLE_");
        result = result.replace("\\v[", "[variable_");
        result = result.replace("\\S[", "[SWITCH_");
        result = result.replace("\\I[", "[ITEM_");
        result = result.replace("\\W[", "[WEAPON_");
        result = result.replace("\\A[", "[ARMOR_");
        result = result.replace("\\P[", "[ACTOR_");
        result = result.replace("\\G", "[GOLD]");
        result = result.replace("\\$", "[CURRENCY]");

        // Expressions conditionnelles
        result = CONDITIONAL_REGEX
            .replace_all(&result, "[CONDITIONAL_v$1>$2]")
            .to_string();

        // Délégation aux patterns universels
        result = UniversalFormatter::prepare_for_translation(&result);

        result
    }

    fn restore_after_translation(text: &str) -> String {
        if !Self::has_placeholder_codes(text) {
            return text.to_string();
        }

        let mut result = text.to_string();

        result = COLOR_RESTORE_REGEX.replace_all(&result, "\\C[$1]").to_string();
        result = result.replace("[COLOR_SIMPLE]", "\\C");
        result = NAME_RESTORE_REGEX.replace_all(&result, "\\N[$1]").to_string();
        result = NEWLINE_RESTORE_REGEX.replace_all(&result, "\\n[$1]").to_string();

        result = F_CODE_RESTORE_REGEX
            .replace_all(&result, |caps: &regex::Captures| {
                if let Some(letters_match) = caps.get(1) {
                    let letters = letters_match.as_str();
                    let number = caps.get(2).map_or("", |m| m.as_str());
                    if letters.is_empty() {
                        format!("\\F[{}]", number)
                    } else {
                        format!("\\F{}[{}]", letters, number)
                    }
                } else {
                    let number = caps.get(3).map_or("", |m| m.as_str());
                    format!("\\F[{}]", number)
                }
            })
            .to_string();

        result = AA_CODE_RESTORE_REGEX.replace_all(&result, "\\AA[$1]").to_string();
        result = CLOSE_BRACE_RESTORE_REGEX.replace_all(&result, "\\}").to_string();

        result = result.replace("[VARIABLE_", "\\V[");
        result = result.replace("[variable_", "\\v[");
        result = result.replace("[SWITCH_", "\\S[");
        result = result.replace("[ITEM_", "\\I[");
        result = result.replace("[WEAPON_", "\\W[");
        result = result.replace("[ARMOR_", "\\A[");
        result = result.replace("[ACTOR_", "\\P[");
        result = result.replace("[GOLD]", "\\G");
        result = result.replace("[CURRENCY]", "\\$");

        result = CONDITIONAL_RESTORE_REGEX
            .replace_all(&result, "en(v[$1]>$2)")
            .to_string();

        result = UniversalFormatter::restore_after_translation(&result);

        result
    }

    fn has_formatting_codes(text: &str) -> bool {
        text.contains('\\')
            || text.contains('%')
            || text.contains('％')
            || text.contains('[')
            || text.contains(']')
            || text.contains('\r')
            || text.contains('\n')
            || text.contains('\t')
            || text.contains('　')
    }

    fn has_placeholder_codes(text: &str) -> bool {
        text.contains('[')
            || text.contains(']')
            || text.contains('％')
            || text.contains('\\')
            || text.contains('\r')
            || text.contains('\n')
            || text.contains('\t')
            || text.contains('　')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpg_maker_roundtrip() {
        let input = "\\C[1]勇者\\C[0]は\\I[317]薬草\\I[317]を使った！";
        let prepared = RpgMakerFormatter::prepare_for_translation(input);
        assert_eq!(prepared, "[COLOR_1]勇者[COLOR_0]は[ITEM_317]薬草[ITEM_317]を使った！");
        let restored = RpgMakerFormatter::restore_after_translation(&prepared);
        assert_eq!(restored, input);
    }

    #[test]
    fn test_plain_text_passthrough() {
        for text in &["勇者", "魔法使い", "薬草"] {
            assert_eq!(RpgMakerFormatter::prepare_for_translation(text), *text);
        }
    }

    #[test]
    fn test_f_code_variations() {
        let cases = vec![
            ("\\F[5]", "[F_5]", "\\F[5]"),
            ("\\F1[10]", "[F_1_10]", "\\F1[10]"),
            ("\\FS[15]", "[F_S_15]", "\\FS[15]"),
        ];
        for (input, expected_prep, expected_rest) in cases {
            let prepared = RpgMakerFormatter::prepare_for_translation(input);
            assert_eq!(prepared, expected_prep);
            let restored = RpgMakerFormatter::restore_after_translation(&prepared);
            assert_eq!(restored, expected_rest);
        }
    }
}
