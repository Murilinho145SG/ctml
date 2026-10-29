use crate::style::TextStyle;
use ctml_css::parse_signed_font_relative_length_with_viewport;

pub(super) fn apply_letter_spacing(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("normal") {
        style.letter_spacing = 0.0;
        style.letter_spacing_px = false;
    } else if let Some(px) = value.strip_suffix("px")
        && let Ok(px) = px.trim().parse::<f32>()
    {
        style.letter_spacing = px;
        style.letter_spacing_px = true;
    } else if let Some(em) = value.strip_suffix("em")
        && let Ok(em) = em.trim().parse::<f32>()
    {
        style.letter_spacing = em;
        style.letter_spacing_px = false;
    } else if !value.contains('%')
        && let Some(length) = parse_signed_font_relative_length_with_viewport(
            value,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
    {
        style.letter_spacing = length;
        style.letter_spacing_px = true;
    }
}
