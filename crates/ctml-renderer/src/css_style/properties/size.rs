use crate::style::TextStyle;
use ctml_css::parse_nonnegative_length;

pub(super) fn apply_width(style: &mut TextStyle, value: &str) {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        style.width = None;
    } else if parse_nonnegative_length(value, style.font_size).is_some() {
        style.width = Some(value.to_ascii_lowercase());
    }
}

pub(super) fn inherit_width(style: &mut TextStyle, inherited: &TextStyle) {
    style.width.clone_from(&inherited.width);
}
