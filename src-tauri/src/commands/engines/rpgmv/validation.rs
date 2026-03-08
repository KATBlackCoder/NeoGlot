// Validation spécifique RPG Maker MV/MZ — étend ContentValidator
use crate::commands::engines::validation::ContentValidator;

/// Validateur RPG Maker : applique la validation universelle puis les règles spécifiques.
pub struct RpgMakerTextValidator;

impl RpgMakerTextValidator {
    /// Retourne `true` si le texte est traduisible pour un projet RPG Maker
    pub fn validate_text(content: &str) -> bool {
        if !ContentValidator::validate_text(content) {
            return false;
        }

        // Ponctuation seule (sans lettres ni chiffres) → non-traduisible
        let has_letters_or_digits = content.chars().any(|c| c.is_alphanumeric());
        if !has_letters_or_digits {
            let has_punctuation = content.chars().any(|c| {
                c.is_ascii_punctuation()
                    || c == '？'
                    || c == '！'
                    || c == '。'
                    || c == '、'
                    || c == '：'
                    || c == '；'
                    || c == '…'
                    || c == '・'
                    || c == '〇'
                    || c == '○'
                    || c == 'ｘ'
                    || c == '×'
            });
            if has_punctuation {
                return false;
            }
        }

        // Chemins de fichiers (sauf codes RPG Maker \n[, \C[, \N[)
        if content.contains('/')
            || (content.contains('\\')
                && !content.contains("\\n[")
                && !content.contains("\\C[")
                && !content.contains("\\N["))
        {
            return false;
        }

        // Commandes de script RPG Maker (ASCII pur avec mots-clés courants)
        // Ex: "PSS start", "PSS end", "chara.useSkill(32);", "addState(1)"
        let trimmed = content.trim();
        let is_all_ascii = trimmed.chars().all(|c| c.is_ascii());
        if is_all_ascii {
            // Commandes plugin (MOT_CLE argument) — que des lettres majuscules + espace + chiffres
            if trimmed.chars().next().map_or(false, |c| c.is_ascii_uppercase())
                && trimmed.contains(' ')
                && trimmed.split_whitespace().all(|w| {
                    w.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                })
                && trimmed.len() <= 50
            {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_punctuation_only() {
        assert!(!RpgMakerTextValidator::validate_text("？！"));
        assert!(!RpgMakerTextValidator::validate_text("..."));
        assert!(!RpgMakerTextValidator::validate_text("。"));
    }

    #[test]
    fn test_pass_text_with_punctuation() {
        assert!(RpgMakerTextValidator::validate_text("こんにちは？"));
        assert!(RpgMakerTextValidator::validate_text("勇者！"));
    }

    #[test]
    fn test_filter_file_paths() {
        assert!(!RpgMakerTextValidator::validate_text("audio/bgm/battle"));
    }

    #[test]
    fn test_allow_rpgmaker_codes() {
        assert!(RpgMakerTextValidator::validate_text("\\C[1]勇者\\C[0]は薬草を使った"));
        assert!(RpgMakerTextValidator::validate_text("\\N[1]は\\n[2]に話しかけた"));
    }
}
