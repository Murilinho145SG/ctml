use crate::style::TextStyle;

pub(super) fn apply_font_variation_settings(style: &mut TextStyle, value: &str) {
    if let Some(settings) = ctml_css::parse_font_variation_settings(value) {
        style.font_variation_settings = settings;
    }
}
