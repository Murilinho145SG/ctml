use crate::style::TextStyle;
use ctml_css::parse_font_feature_settings;

pub(super) fn apply_font_feature_settings(style: &mut TextStyle, value: &str) {
    if let Some(features) = parse_font_feature_settings(value) {
        style.font_feature_settings = features;
    }
}
