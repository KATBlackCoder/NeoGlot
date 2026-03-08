// Validation universelle du contenu — logique commune à tous les moteurs

/// Validation universelle : filtre les textes non-traduisibles
/// (vide, placeholders seuls, identifiants techniques, code JS, etc.)
pub struct ContentValidator;

impl ContentValidator {
    /// Détermine si un texte déjà ASCII doit être ignoré quand on traduit vers l'anglais
    pub fn get_initial_status_is_ignored(content: &str, target_language: &str) -> bool {
        let content = content.trim();
        let is_ascii_target =
            target_language.eq_ignore_ascii_case("en") || target_language.eq_ignore_ascii_case("english");

        if is_ascii_target {
            let is_ascii_text = content
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c.is_ascii_punctuation() || c.is_ascii_whitespace());

            if is_ascii_text && content.chars().any(|c| c.is_alphabetic()) {
                return true;
            }
        }
        false
    }

    /// Retourne `true` si le texte est traduisible, `false` s'il doit être filtré
    pub fn validate_text(content: &str) -> bool {
        let content = content.trim();

        if content.is_empty() {
            return false;
        }

        // Placeholder unique [A-Z_][A-Z0-9_]*
        if content.starts_with('[') && content.ends_with(']') && content.len() > 2 {
            let inner = &content[1..content.len() - 1];
            if inner
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                && inner.chars().next().map_or(false, |c| c.is_ascii_uppercase() || c == '_')
            {
                return false;
            }
        }

        // Multiples placeholders sans contenu réel
        if content.contains('[') && content.contains(']') {
            let mut remaining = content.to_string();
            let mut found = false;
            loop {
                let mut changed = false;
                if let Some(start) = remaining.find('[') {
                    if let Some(end) = remaining[start..].find(']') {
                        let inner = &remaining[start + 1..start + end];
                        if !inner.is_empty()
                            && inner
                                .chars()
                                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                            && inner.chars().next().map_or(false, |c| c.is_ascii_uppercase() || c == '_')
                        {
                            remaining.replace_range(start..start + end + 1, "");
                            found = true;
                            changed = true;
                        }
                    }
                }
                if !changed {
                    break;
                }
            }
            if found && remaining.trim().is_empty() {
                return false;
            }
        }

        // Marqueurs techniques japonais
        if content == "〇" || content == "ｘ" || content == "○" || content == "×" {
            return false;
        }

        let looks_cjk = content.chars().any(|c| {
            ('\u{4E00}'..='\u{9FFF}').contains(&c)
                || ('\u{3040}'..='\u{309F}').contains(&c)
                || ('\u{30A0}'..='\u{30FF}').contains(&c)
                || ('\u{AC00}'..='\u{D7AF}').contains(&c)
                || Self::contains_japanese_punctuation(content)
        });

        // Identifiants EV / MAP
        if content.starts_with("EV") && content.len() >= 3 {
            if content[2..].chars().next().map_or(false, |c| c.is_ascii_digit()) {
                return false;
            }
        }
        if content.starts_with("MAP") && content.len() >= 4 {
            if content[3..].chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
        }

        // Guillemets japonais seuls
        let trimmed = content.trim();
        if trimmed == "「" || trimmed == "」" || trimmed == "「」" {
            return false;
        }
        if trimmed.chars().all(|c| c == '「' || c == '」' || c == ' ')
            && trimmed.chars().any(|c| c == '「' || c == '」')
        {
            return false;
        }

        // Codes formatage purs
        if content == "\\n[1]" || content == "\\n[2]" || content == "\\n[3]"
            || content == "\\n[4]" || content == "\\n[5]"
        {
            return false;
        }

        // Extensions de fichiers
        if content.contains('.') {
            let parts: Vec<&str> = content.split('.').collect();
            if parts.len() == 2 {
                let ext = parts[1];
                if ext.len() >= 2 && ext.len() <= 4
                    && ext.chars().all(|c| c.is_ascii_alphanumeric())
                    && ext.chars().any(|c| c.is_ascii_alphabetic())
                {
                    return false;
                }
            }
            if parts.len() > 2 {
                return false;
            }
        }

        // Code JavaScript / appels de méthodes (pattern: identifier.method( ou identifier.property)
        if content.contains("user.") || content.contains("use.")
            || content.contains("&&") || content.contains("==")
            || content.contains("();") || content.contains(");")
            || content.contains("function ") || content.contains("var ")
            || content.contains("this.") || content.contains("self.")
        {
            return false;
        }

        // Appels de méthodes : pattern "word.word(" — ex: chara.useSkill(32)
        if content.contains('.') && content.contains('(') {
            let has_method_call = content
                .split('.')
                .skip(1)
                .any(|part| part.contains('('));
            if has_method_call && content.chars().all(|c| c.is_ascii() || c.is_ascii_whitespace()) {
                return false;
            }
        }

        // Marqueurs techniques
        if content == "終わり" || content == "==" || content.starts_with("==") {
            return false;
        }

        // Pipes (coordonnées / données séparées)
        if content.contains('|') {
            return false;
        }

        // Mots ASCII courts dans contexte CJK
        if looks_cjk && content.chars().all(|c| c.is_ascii_alphabetic()) && content.len() <= 20 {
            return false;
        }

        // Texte ASCII pur dans contexte CJK
        if looks_cjk && content.chars().all(Self::is_ascii_or_fullwidth_latin) {
            return false;
        }

        // Identifiants techniques (snake_case)
        if content.contains('_')
            && content
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == 'x' || c == 'X')
        {
            return false;
        }

        // Nombres purs
        if content.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        // Contenu très court en contexte CJK
        if looks_cjk && content.len() <= 3 {
            if content.chars().any(|c| c.is_alphabetic() && !c.is_ascii())
                || Self::contains_japanese_punctuation(content)
            {
                return true;
            }
            return false;
        }

        // Caractères non-ASCII → traduisible
        if content.chars().any(|c| c.is_alphabetic() && !c.is_ascii()) {
            return true;
        }

        // Ponctuation japonaise → traduisible
        if Self::contains_japanese_punctuation(content) {
            return true;
        }

        // Texte ASCII raisonnable
        if content.chars().any(|c| c.is_alphabetic())
            && content.len() >= 2
            && content.len() <= 100
            && !content.chars().all(|c| c.is_ascii_digit())
        {
            return true;
        }

        false
    }

    fn is_ascii_or_fullwidth_latin(c: char) -> bool {
        c.is_ascii_alphanumeric()
            || c.is_ascii_punctuation()
            || c.is_ascii_whitespace()
            || ('\u{FF21}'..='\u{FF5A}').contains(&c)
            || ('\u{FF41}'..='\u{FF5A}').contains(&c)
            || ('\u{FF10}'..='\u{FF19}').contains(&c)
    }

    fn contains_japanese_punctuation(content: &str) -> bool {
        content.contains('「')
            || content.contains('」')
            || content.contains('、')
            || content.contains('。')
            || content.contains('・')
            || content.contains('…')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_technical() {
        assert!(!ContentValidator::validate_text("EV002"));
        assert!(!ContentValidator::validate_text("MAP001"));
        assert!(!ContentValidator::validate_text("image.png"));
        assert!(!ContentValidator::validate_text("user.name && true"));
        assert!(!ContentValidator::validate_text("はい|262|380"));
        assert!(!ContentValidator::validate_text("[NEWLINE_1]"));
    }

    #[test]
    fn test_pass_translatable() {
        assert!(ContentValidator::validate_text("勇者"));
        assert!(ContentValidator::validate_text("[COLOR_1]勇者[COLOR_0]"));
        assert!(ContentValidator::validate_text("Hello World"));
    }
}
