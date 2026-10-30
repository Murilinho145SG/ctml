use crate::style::TextStyle;
use ctml_css::parse_color_with_opacity_and_profiles;

pub(crate) fn apply_color_declaration(
    style: &mut TextStyle,
    value: &str,
    inherited_color: ([u8; 4], f32),
) -> bool {
    let value = value.trim();
    let color = if value.eq_ignore_ascii_case("currentcolor") {
        Some(inherited_color)
    } else if matches!(value.to_ascii_lowercase().as_str(), "initial" | "revert") {
        Some(([0, 0, 0, 255], 1.0))
    } else if matches!(value.to_ascii_lowercase().as_str(), "inherit" | "unset") {
        Some(inherited_color)
    } else {
        parse_color_with_opacity_and_profiles(value, &style.color_profiles).map(
            |(channels, opacity)| {
                (
                    [
                        channels[0],
                        channels[1],
                        channels[2],
                        (opacity * 255.0).round() as u8,
                    ],
                    opacity,
                )
            },
        )
    };
    if let Some((color, opacity)) = color {
        style.color = color;
        style.color_opacity = opacity;
        true
    } else {
        false
    }
}
