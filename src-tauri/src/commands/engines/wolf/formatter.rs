// Formatter Wolf RPG Editor — convertit les codes moteur en placeholders AI-friendly
use crate::commands::engines::formatter::{EngineFormatter, UniversalFormatter};
use once_cell::sync::Lazy;
use regex::Regex;

// ═══ Regexes Wolf RPG (pré-compilées) ═══

static ICON_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\i\[(\d+)\]").unwrap());
static FONT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\f\[(\d+)\]").unwrap());
static AT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"@(\d+)").unwrap());
static SLOT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\s\[(\d+)\]").unwrap());
static CSELF_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\cself\[(\d+)\]").unwrap());
static COLOR_REGEX_LOWER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\c\[(\d+)\]").unwrap());
static COLOR_REGEX_UPPER: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\C\[(\d+)\]").unwrap());
static SYS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\sys\[(\d+)\]").unwrap());
static FONT_FULL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\font\[(\d+)\]").unwrap());
static AX_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\ax\[([^\]]+)\]").unwrap());
static AY_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\ay\[([^\]]+)\]").unwrap());
static V_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\v\[(\d+)\]").unwrap());
static F_SIMPLE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\f\[([^\]]+)\]").unwrap());
static CDB_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\cdb\[(\d+:\d+:\d+)\]").unwrap());
static INDENT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\-\[(\d+)\]").unwrap());
static SPACE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\space\[(\d+)\]").unwrap());

// Restoration
static ICON_RESTORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[ICON_(\d+)\]").unwrap());
static FONT_RESTORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[FONT_(\d+)\]").unwrap());
static AT_RESTORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[AT_(\d+)\]").unwrap());
static SLOT_RESTORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[SLOT_(\d+)\]").unwrap());
static CSELF_RESTORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[CSELF_(\d+)\]").unwrap());
static COLOR_RESTORE_LOWER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[COLOR_LOWER_(\d+)\]").unwrap());
static COLOR_RESTORE_UPPER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[COLOR_UPPER_(\d+)\]").unwrap());
static SYS_RESTORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[SYS_(\d+)\]").unwrap());
static FONT_FULL_RESTORE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[FONT_FULL_(\d+)\]").unwrap());
static AX_RESTORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[AX_([^\]]+)\]").unwrap());
static AY_RESTORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[AY_([^\]]+)\]").unwrap());
static V_RESTORE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[V_(\d+)\]").unwrap());
static F_SIMPLE_RESTORE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[F_SIMPLE_([^\]]+)\]").unwrap());
static CDB_RESTORE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[CDB_(\d+:\d+:\d+)\]").unwrap());
static INDENT_RESTORE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[INDENT_(\d+)\]").unwrap());
static SPACE_RESTORE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[SPACE_(\d+)\]").unwrap());

/// Formatter spécifique Wolf RPG Editor.
/// Gère les codes \E, \i, \f, \s, \cself, \c/\C, \sys, \font, \ax, \ay, \v,
/// \cdb, \-, \space, <C>, <R>, \>, <<, >>, etc.
pub struct WolfRpgFormatter;

impl EngineFormatter for WolfRpgFormatter {
    fn prepare_for_translation(text: &str) -> String {
        if !Self::has_formatting_codes(text) {
            return text.to_string();
        }

        let mut result = text.to_string();

        result = result.replace("\\E", "[WOLF_END]");
        result = ICON_REGEX.replace_all(&result, "[ICON_$1]").to_string();
        result = FONT_REGEX.replace_all(&result, "[FONT_$1]").to_string();
        result = AT_REGEX.replace_all(&result, "[AT_$1]").to_string();
        result = SLOT_REGEX.replace_all(&result, "[SLOT_$1]").to_string();
        result = CSELF_REGEX.replace_all(&result, "[CSELF_$1]").to_string();
        result = COLOR_REGEX_LOWER.replace_all(&result, "[COLOR_LOWER_$1]").to_string();
        result = COLOR_REGEX_UPPER.replace_all(&result, "[COLOR_UPPER_$1]").to_string();

        result = SYS_REGEX.replace_all(&result, "[SYS_$1]").to_string();
        result = FONT_FULL_REGEX.replace_all(&result, "[FONT_FULL_$1]").to_string();
        result = AX_REGEX.replace_all(&result, "[AX_$1]").to_string();
        result = AY_REGEX.replace_all(&result, "[AY_$1]").to_string();
        result = V_REGEX.replace_all(&result, "[V_$1]").to_string();
        result = F_SIMPLE_REGEX.replace_all(&result, "[F_SIMPLE_$1]").to_string();
        result = CDB_REGEX.replace_all(&result, "[CDB_$1]").to_string();
        result = INDENT_REGEX.replace_all(&result, "[INDENT_$1]").to_string();
        result = SPACE_REGEX.replace_all(&result, "[SPACE_$1]").to_string();
        result = result.replace("<C>", "[CENTER_TAG]");
        result = result.replace("\\>", "[RIGHT_ALIGN]");
        result = result.replace("<R>", "[RIGHT_TAG]");
        result = result.replace("<<", "[LEFT_BRACKETS]");
        result = result.replace(">>", "[RIGHT_BRACKETS]");

        result = result.replace("\\r", "[RUBY_START]");
        result = result.replace('\r', "[CARRIAGE_RETURN]");
        result = result.replace('\n', "[NEWLINE]");

        result = UniversalFormatter::prepare_for_translation(&result);

        result
    }

    fn restore_after_translation(text: &str) -> String {
        if !Self::has_placeholder_codes(text) {
            return text.to_string();
        }

        let mut result = text.to_string();

        result = result.replace("[WOLF_END]", "\\E");
        result = ICON_RESTORE_REGEX.replace_all(&result, "\\i[$1]").to_string();
        result = FONT_RESTORE_REGEX.replace_all(&result, "\\f[$1]").to_string();
        result = AT_RESTORE_REGEX.replace_all(&result, "@$1").to_string();
        result = SLOT_RESTORE_REGEX.replace_all(&result, "\\s[$1]").to_string();
        result = CSELF_RESTORE_REGEX.replace_all(&result, "\\cself[$1]").to_string();
        result = COLOR_RESTORE_LOWER_REGEX.replace_all(&result, "\\c[$1]").to_string();
        result = COLOR_RESTORE_UPPER_REGEX.replace_all(&result, "\\C[$1]").to_string();

        result = SYS_RESTORE_REGEX.replace_all(&result, "\\sys[$1]").to_string();
        result = FONT_FULL_RESTORE_REGEX.replace_all(&result, "\\font[$1]").to_string();
        result = AX_RESTORE_REGEX.replace_all(&result, "\\ax[$1]").to_string();
        result = AY_RESTORE_REGEX.replace_all(&result, "\\ay[$1]").to_string();
        result = V_RESTORE_REGEX.replace_all(&result, "\\v[$1]").to_string();
        result = F_SIMPLE_RESTORE_REGEX.replace_all(&result, "\\f[$1]").to_string();
        result = CDB_RESTORE_REGEX.replace_all(&result, "\\cdb[$1]").to_string();
        result = INDENT_RESTORE_REGEX.replace_all(&result, "\\-[$1]").to_string();
        result = SPACE_RESTORE_REGEX.replace_all(&result, "\\space[$1]").to_string();
        result = result.replace("[CENTER_TAG]", "<C>");
        result = result.replace("[RIGHT_ALIGN]", "\\>");
        result = result.replace("[RIGHT_TAG]", "<R>");
        result = result.replace("[LEFT_BRACKETS]", "<<");
        result = result.replace("[RIGHT_BRACKETS]", ">>");

        result = result.replace("[RUBY_START]", "\\r");
        result = result.replace("[CARRIAGE_RETURN]", "\\r");
        result = result.replace("[NEWLINE]", "\n");

        result = UniversalFormatter::restore_after_translation(&result);

        result
    }

    fn has_formatting_codes(text: &str) -> bool {
        text.contains('\\')
            || text.contains('@')
            || text.contains('%')
            || text.contains('％')
            || text.contains('[')
            || text.contains(']')
            || text.contains('「')
            || text.contains('」')
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
            || text.contains('@')
            || text.contains('「')
            || text.contains('」')
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
    fn test_wolf_roundtrip() {
        let input = "\\E\\i[1]テスト@1\\f[2]";
        let prepared = WolfRpgFormatter::prepare_for_translation(input);
        assert_eq!(prepared, "[WOLF_END][ICON_1]テスト[AT_1][FONT_2]");
        let restored = WolfRpgFormatter::restore_after_translation(&prepared);
        assert_eq!(restored, input);
    }

    #[test]
    fn test_plain_text_passthrough() {
        for text in &["勇者", "魔法使い", "薬草"] {
            assert_eq!(WolfRpgFormatter::prepare_for_translation(text), *text);
        }
    }

    #[test]
    fn test_color_case_distinction() {
        let lower = "\\c[2]テスト";
        let upper = "\\C[3]テスト";
        let prep_lower = WolfRpgFormatter::prepare_for_translation(lower);
        let prep_upper = WolfRpgFormatter::prepare_for_translation(upper);
        assert!(prep_lower.contains("[COLOR_LOWER_2]"));
        assert!(prep_upper.contains("[COLOR_UPPER_3]"));
        assert_eq!(WolfRpgFormatter::restore_after_translation(&prep_lower), lower);
        assert_eq!(WolfRpgFormatter::restore_after_translation(&prep_upper), upper);
    }
}
