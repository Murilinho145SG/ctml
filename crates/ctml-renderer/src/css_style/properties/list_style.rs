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
    "none",
];

pub(super) fn apply_list_style_type(style: &mut TextStyle, value: &str) {
    let value = value.trim().to_ascii_lowercase();
    if SUPPORTED_LIST_STYLE_TYPES.contains(&value.as_str()) {
        style.list_style_type = Some(value);
    }
}

pub(super) fn apply_list_style_shorthand(style: &mut TextStyle, value: &str) {
    apply_list_style_type(style, value);
}

pub(super) fn reset_list_style_type(style: &mut TextStyle) {
    style.list_style_type = Some("disc".to_owned());
}
