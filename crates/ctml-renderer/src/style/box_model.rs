use ctml_css::parse_nonnegative_font_relative_length_with_viewport;

use super::TextStyle;

pub(super) fn resolve_padding(style: &TextStyle) -> [f32; 4] {
    std::array::from_fn(|index| {
        style.padding[index]
            .as_deref()
            .and_then(|value| {
                parse_nonnegative_font_relative_length_with_viewport(
                    value,
                    style.font_size,
                    style.line_height,
                    style.font_relative_units,
                    style.viewport_size,
                )
            })
            .unwrap_or_default()
    })
}
