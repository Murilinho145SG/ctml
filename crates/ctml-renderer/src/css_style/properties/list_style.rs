use crate::style::TextStyle;

const SUPPORTED_LIST_STYLE_TYPES: &[&str] = &[
    "disc",
    "circle",
    "square",
    "decimal",
    "decimal-leading-zero",
    "lower-alpha",
    "upper-alpha",
    "lower-greek",
    "lower-roman",
    "upper-roman",
    "armenian",
    "upper-armenian",
    "lower-armenian",
    "hebrew",
    "georgian",
    "cjk-decimal",
    "cjk-ideographic",
    "cjk-earthly-branch",
    "cjk-heavenly-stem",
    "arabic-indic",
    "bengali",
    "cambodian",
    "khmer",
    "devanagari",
    "gujarati",
    "gurmukhi",
    "kannada",
    "lao",
    "malayalam",
    "mongolian",
    "myanmar",
    "oriya",
    "persian",
    "tamil",
    "telugu",
    "thai",
    "tibetan",
    "lower-latin",
    "upper-latin",
    "hiragana",
    "hiragana-iroha",
    "katakana",
    "katakana-iroha",
    "none",
];

pub(super) fn apply_list_style_type(style: &mut TextStyle, value: &str) {
    let value = value.trim();
    let keyword = value.to_ascii_lowercase();
    if SUPPORTED_LIST_STYLE_TYPES.contains(&keyword.as_str()) {
        style.list_style_type = Some(keyword);
    } else if let Some(marker) = ctml_css::parse_css_string(value) {
        style.list_style_type = Some(format!("custom:{marker}"));
    }
}

pub(super) fn apply_list_style_shorthand(style: &mut TextStyle, value: &str) {
    apply_list_style_type(style, value);
}

pub(super) fn reset_list_style_type(style: &mut TextStyle) {
    style.list_style_type = Some("disc".to_owned());
}
