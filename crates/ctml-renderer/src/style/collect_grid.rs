use super::{FlexBasis, GridTrack, TextStyle};

pub(super) fn item_basis(style: &TextStyle, item_index: usize) -> FlexBasis {
    let tracks = if style.grid_template_columns.is_empty() {
        &[GridTrack::Fraction(1.0)][..]
    } else {
        &style.grid_template_columns
    };
    match tracks[item_index % tracks.len()] {
        GridTrack::Length(length) => FlexBasis::Length(length),
        GridTrack::Fraction(fraction) => {
            let total = tracks
                .iter()
                .filter_map(|track| match track {
                    GridTrack::Fraction(fraction) => Some(*fraction),
                    GridTrack::Length(_) => None,
                })
                .sum::<f32>();
            FlexBasis::Percentage(fraction / total.max(fraction))
        }
    }
}

pub(super) fn column_count(style: &TextStyle) -> usize {
    style.grid_template_columns.len().max(1)
}

pub(super) fn row_gap(style: &TextStyle) -> f32 {
    let Some(gap) = style.row_gap.as_deref() else {
        return 0.0;
    };
    if ctml_css::parse_css_percentage(gap).is_some() {
        return 0.0;
    }
    ctml_css::parse_math_expression(gap, |component| {
        ctml_css::parse_nonnegative_font_relative_length_with_viewport(
            component,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
    })
    .or_else(|| {
        ctml_css::parse_nonnegative_font_relative_length_with_viewport(
            gap,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
    })
    .filter(|gap| gap.is_finite())
    .unwrap_or_default()
}
