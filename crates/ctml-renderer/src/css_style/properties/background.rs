use crate::style::TextStyle;
use ctml_css::parse_color_with_opacity_and_profiles;

pub(super) fn apply_background_shorthand(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("none") {
        style.background = [255, 255, 255];
        style.background_color = None;
    } else {
        apply_background_color(style, value);
    }
}

pub(super) fn apply_background_color(style: &mut TextStyle, value: &str) {
    let color = if value.eq_ignore_ascii_case("currentcolor") {
        Some((
            [style.color[0], style.color[1], style.color[2]],
            style.color_opacity,
        ))
    } else {
        parse_color_with_opacity_and_profiles(value, &style.color_profiles)
    };
    let Some((channels, alpha)) = color else {
        return;
    };
    style.background_color = Some([
        channels[0],
        channels[1],
        channels[2],
        (alpha * 255.0).round() as u8,
    ]);
    style.background = [
        composite_over_white(channels[0], alpha),
        composite_over_white(channels[1], alpha),
        composite_over_white(channels[2], alpha),
    ];
}

fn composite_over_white(channel: u8, alpha: f32) -> u8 {
    (channel as f32 * alpha + 255.0 * (1.0 - alpha)).round() as u8
}
