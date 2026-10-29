use ctml_css::parse_length;

pub(in crate::css_style) fn parse_font_size(value: &str, relative_to: f32) -> Option<f32> {
    let absolute_size = match value.trim().to_ascii_lowercase().as_str() {
        "xx-small" => Some(9.0),
        "x-small" => Some(10.0),
        "small" => Some(13.0),
        "medium" => Some(16.0),
        "large" => Some(18.0),
        "x-large" => Some(24.0),
        "xx-large" => Some(32.0),
        "xxx-large" => Some(48.0),
        "larger" => Some(relative_to * 1.2),
        "smaller" => Some(relative_to / 1.2),
        _ => None,
    };
    absolute_size.or_else(|| parse_length(value, relative_to))
}
