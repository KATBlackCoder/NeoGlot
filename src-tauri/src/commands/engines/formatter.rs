// Trait commun pour les formatters moteur + UniversalFormatter (patterns partagés)
use once_cell::sync::Lazy;
use regex::{Captures, Regex};

/// Convertit les chiffres pleine largeur en chiffres ASCII
fn to_ascii_digits(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '０'..='９' => char::from_u32('0' as u32 + (c as u32 - '０' as u32)).unwrap(),
            d => d,
        })
        .collect()
}

// ═══ Regexes universelles pré-compilées ═══

static ARG_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[%％]([0-9０-９]+)").unwrap());
static NUM_PREFIX_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([0-9０-９]{3})[＿_](.+)$").unwrap());

static FW_SPACE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(　+)").unwrap());
static LEADING_SPACES_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^( +)").unwrap());
static TRAILING_SPACES_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"( +)$").unwrap());
static MULTI_SPACES_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"( {2,})").unwrap());
static TABS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\t+)").unwrap());

static ARG_RESTORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[ARG_(\d+)\]").unwrap());
static NUM_PREFIX_RESTORE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[NUM_PREFIX_(\d{3})\]").unwrap());
static FW_SPACE_RESTORE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[FWSPC_(\d+)\]").unwrap());
static SPC_RESTORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[SPC_(\d+)\]").unwrap());
static TAB_RESTORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[TAB_(\d+)\]").unwrap());

// ═══ Trait EngineFormatter ═══

/// Interface commune pour le formatage moteur-spécifique.
/// Chaque moteur implémente ce trait pour convertir ses codes en placeholders
/// avant traduction AI, puis restaurer les codes originaux après.
pub trait EngineFormatter {
    /// Codes moteur → placeholders lisibles pour l'AI
    fn prepare_for_translation(text: &str) -> String;

    /// Placeholders → codes moteur originaux après traduction
    fn restore_after_translation(text: &str) -> String;

    /// Vérifie rapidement si le texte contient des codes moteur
    fn has_formatting_codes(text: &str) -> bool;

    /// Vérifie rapidement si le texte contient des placeholders
    fn has_placeholder_codes(text: &str) -> bool;
}

// ═══ UniversalFormatter ═══

/// Patterns universels partagés par tous les moteurs :
/// %n / ％n, préfixes numériques, whitespace, guillemets japonais, codes contrôle
pub struct UniversalFormatter;

impl EngineFormatter for UniversalFormatter {
    fn prepare_for_translation(text: &str) -> String {
        if !Self::has_formatting_codes(text) {
            return text.to_string();
        }

        let mut result = text.to_string();

        // Préfixes numériques de maps/zones
        if let Some(caps) = NUM_PREFIX_REGEX.captures(&result) {
            let prefix_ascii = to_ascii_digits(&caps[1]);
            let tail = caps[2].to_string();
            result = format!("[NUM_PREFIX_{}]{}", prefix_ascii, tail);
        }

        // Paramètres %1 / ％1
        result = ARG_REGEX
            .replace_all(&result, |caps: &Captures| {
                format!("[ARG_{}]", to_ascii_digits(&caps[1]))
            })
            .to_string();

        // Codes contrôle
        result = result.replace("\\.", "[CTRL_DOT]");
        result = result.replace("\\|", "[CTRL_WAIT]");
        result = result.replace("\\^", "[CTRL_INSTANT]");
        result = result.replace("\\!", "[CTRL_INPUT]");
        result = result.replace("\\{", "[CTRL_OPEN_BRACE]");

        // Caractères de contrôle littéraux
        result = result.replace('\n', "[CTRL_NEWLINE]");
        result = result.replace('\r', "[CTRL_CARRIAGE_RETURN]");
        result = result.replace('\t', "[CTRL_TAB]");

        // Normalisation guillemets japonais (transformation unidirectionnelle)
        result = result.replace('「', "\"");
        result = result.replace('」', "\"");
        result = result.replace('『', "'");
        result = result.replace('』', "'");

        // Encodage whitespace
        result = Self::encode_whitespace(&result);

        result
    }

    fn restore_after_translation(text: &str) -> String {
        if !Self::has_placeholder_codes(text) {
            return text.to_string();
        }

        let mut result = text.to_string();

        result = Self::decode_whitespace(&result);

        result = result.replace("[CTRL_NEWLINE]", "\n");
        result = result.replace("[CTRL_CARRIAGE_RETURN]", "\r");
        result = result.replace("[CTRL_TAB]", "\t");

        result = ARG_RESTORE_REGEX.replace_all(&result, "%$1").to_string();
        result = NUM_PREFIX_RESTORE_REGEX
            .replace_all(&result, |caps: &Captures| format!("{}＿", &caps[1]))
            .to_string();

        result = result.replace("[CTRL_DOT]", "\\.");
        result = result.replace("[CTRL_WAIT]", "\\|");
        result = result.replace("[CTRL_INSTANT]", "\\^");
        result = result.replace("[CTRL_INPUT]", "\\!");
        result = result.replace("[CTRL_OPEN_BRACE]", "\\{");

        result
    }

    fn has_formatting_codes(text: &str) -> bool {
        text.contains('%')
            || text.contains('％')
            || text.contains('\\')
            || text.contains('\r')
            || text.contains('\n')
            || text.contains('\t')
            || text.contains('　')
            || text.contains('＿')
            || text.contains('_')
            || text.contains('「')
            || text.contains('」')
            || text.contains('『')
            || text.contains('』')
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

impl UniversalFormatter {
    fn encode_whitespace(input: &str) -> String {
        let mut result = input.to_string();

        result = FW_SPACE_REGEX
            .replace_all(&result, |caps: &Captures| {
                format!("[FWSPC_{}]", caps[1].chars().count())
            })
            .to_string();

        result = LEADING_SPACES_REGEX
            .replace(&result, |caps: &Captures| format!("[SPC_{}]", caps[1].len()))
            .to_string();

        result = TRAILING_SPACES_REGEX
            .replace(&result, |caps: &Captures| format!("[SPC_{}]", caps[1].len()))
            .to_string();

        result = MULTI_SPACES_REGEX
            .replace_all(&result, |caps: &Captures| format!("[SPC_{}]", caps[1].len()))
            .to_string();

        result = TABS_REGEX
            .replace_all(&result, |caps: &Captures| format!("[TAB_{}]", caps[1].len()))
            .to_string();

        result
    }

    fn decode_whitespace(input: &str) -> String {
        let mut result = input.to_string();

        result = FW_SPACE_RESTORE_REGEX
            .replace_all(&result, |caps: &Captures| {
                "　".repeat(caps[1].parse().unwrap_or(0))
            })
            .to_string();

        result = SPC_RESTORE_REGEX
            .replace_all(&result, |caps: &Captures| {
                " ".repeat(caps[1].parse().unwrap_or(0))
            })
            .to_string();

        result = TAB_RESTORE_REGEX
            .replace_all(&result, |caps: &Captures| {
                "\t".repeat(caps[1].parse().unwrap_or(0))
            })
            .to_string();

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_arg_roundtrip() {
        let input = "%1は瞑想した！";
        let prepared = UniversalFormatter::prepare_for_translation(input);
        assert_eq!(prepared, "[ARG_1]は瞑想した！");
        let restored = UniversalFormatter::restore_after_translation(&prepared);
        assert_eq!(restored, "%1は瞑想した！");
    }

    #[test]
    fn test_plain_text_passthrough() {
        for text in &["勇者", "魔法使い", "薬草", "はい", "いいえ"] {
            assert_eq!(UniversalFormatter::prepare_for_translation(text), *text);
        }
    }

    #[test]
    fn test_japanese_quotes_normalization() {
        let input = "勇者「こんにちは」と言った";
        let prepared = UniversalFormatter::prepare_for_translation(input);
        assert_eq!(prepared, "勇者\"こんにちは\"と言った");
    }
}
