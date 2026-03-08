// Validation spécifique Wolf RPG Editor — étend ContentValidator
use crate::commands::engines::validation::ContentValidator;
use once_cell::sync::Lazy;
use regex::Regex;

static PLACEHOLDER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[[A-Z_][A-Z0-9_]*\]").unwrap());

static NESTED_PLACEHOLDER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[[A-Z_][A-Z0-9_]*[^\]]*\]").unwrap());

static FILE_EXT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\.(png|jpg|jpeg|gif|bmp|wav|mp3|ogg|txt|json|dat)").unwrap());

/// Validateur Wolf RPG : applique la validation universelle puis les règles spécifiques.
pub struct WolfRpgTextValidator;

impl WolfRpgTextValidator {
    /// Retourne `true` si le texte est traduisible pour un projet Wolf RPG
    pub fn validate_text(content: &str) -> bool {
        if !ContentValidator::validate_text(content) {
            return false;
        }

        let trimmed = content.trim();
        if trimmed.is_empty() {
            return false;
        }

        // Suppression itérative des placeholders (simples et imbriqués)
        let mut without_placeholders = trimmed.to_string();
        let mut previous_len = without_placeholders.len();
        loop {
            without_placeholders = PLACEHOLDER_REGEX
                .replace_all(&without_placeholders, "")
                .to_string();
            without_placeholders = NESTED_PLACEHOLDER_REGEX
                .replace_all(&without_placeholders, "")
                .to_string();
            if without_placeholders.len() == previous_len {
                break;
            }
            previous_len = without_placeholders.len();
        }
        let without_placeholders = without_placeholders.trim();

        // Suppression des caractères de contrôle et séquences d'échappement
        let without_controls = without_placeholders
            .replace("\\n", "")
            .replace("\\r", "")
            .replace("\\t", "")
            .replace('\\', "")
            .replace('\n', "")
            .replace('\r', "")
            .replace('\t', "")
            .replace('@', "")
            .replace('<', "")
            .replace('>', "");
        let without_controls = without_controls.trim();

        if without_controls.is_empty() {
            return false;
        }

        // Contenu uniquement chiffres + caractères spéciaux → non-traduisible
        let only_digits_special: String = without_controls
            .chars()
            .filter(|c| {
                c.is_ascii_digit()
                    || matches!(
                        c,
                        '.' | ',' | ':' | ';' | '…' | '-' | '+' | '=' | '(' | ')'
                            | '[' | ']' | '{' | '}' | '/' | '\\' | '?' | '!' | ' '
                            | '\t' | '\n' | '\r'
                    )
            })
            .collect();

        if only_digits_special.trim() == without_controls {
            let has_text = without_controls.chars().any(|c| {
                c.is_alphabetic()
                    || (c as u32 >= 0x3040 && c as u32 <= 0x9FFF)
                    || (c as u32 >= 0x3400 && c as u32 <= 0x4DBF)
                    || (c as u32 >= 0x20000 && c as u32 <= 0x2A6DF)
            });
            if !has_text {
                return false;
            }
        }

        // Caractères spéciaux seuls (pas de lettres, chiffres ni CJK)
        let without_special: String = without_controls
            .chars()
            .filter(|c| {
                c.is_alphanumeric()
                    || ((*c as u32) >= 0x3040 && (*c as u32) <= 0x9FFF)
                    || ((*c as u32) >= 0x3400 && (*c as u32) <= 0x4DBF)
                    || ((*c as u32) >= 0x20000 && (*c as u32) <= 0x2A6DF)
                    || matches!(c, '！' | '？' | '。' | '、' | '，' | '「' | '」' | '『' | '』')
            })
            .collect();

        if without_special.trim().is_empty() {
            return false;
        }

        // Messages debug/erreur X[
        if content.contains("X[") {
            return false;
        }

        // Extensions de fichiers
        if FILE_EXT_REGEX.is_match(&content.to_lowercase()) {
            return false;
        }

        // Chemins Data\ ou Data/
        if content.starts_with("Data\\") || content.starts_with("Data/") {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_placeholder_only() {
        assert!(!WolfRpgTextValidator::validate_text("[AT_1][NEWLINE][CSELF_9]"));
        assert!(!WolfRpgTextValidator::validate_text("[F_SIMPLE_[CSELF_18]][CSELF_7]"));
        assert!(!WolfRpgTextValidator::validate_text("\\n"));
    }

    #[test]
    fn test_pass_text_with_placeholders() {
        assert!(WolfRpgTextValidator::validate_text("[AT_1]テスト"));
        assert!(WolfRpgTextValidator::validate_text("勇者"));
    }

    #[test]
    fn test_filter_data_paths() {
        assert!(!WolfRpgTextValidator::validate_text("Data\\SE\\attack.mp3"));
        assert!(!WolfRpgTextValidator::validate_text("Data/BGM/"));
    }

    #[test]
    fn test_filter_x_bracket() {
        assert!(!WolfRpgTextValidator::validate_text("X[戦]テスト"));
    }
}
