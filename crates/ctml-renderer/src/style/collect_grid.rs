use super::{FlexBasis, GridTrack, TextStyle};

pub(super) fn item_basis(style: &TextStyle, item_index: usize) -> FlexBasis {
    let tracks = if style.grid_template_columns.is_empty() {
        &[GridTrack::Fraction(1.0)][..]
    } else {
        &style.grid_template_columns
    };
    match tracks[item_index % tracks.len()] {
        GridTrack::Length(length) => FlexBasis::Length(length),
        GridTrack::Percentage(percentage) => FlexBasis::Percentage(percentage),
        GridTrack::Auto => FlexBasis::MaxContent,
        GridTrack::MinContent => FlexBasis::MinContent,
        GridTrack::MaxContent => FlexBasis::MaxContent,
        GridTrack::FitContent(limit) => FlexBasis::GridFitContent(limit),
        GridTrack::MinMax { min, max } => FlexBasis::GridMinMax { min, max },
        GridTrack::Fraction(fraction) => {
            let total_fraction = tracks
                .iter()
                .filter_map(|track| match track {
                    GridTrack::Fraction(fraction) => Some(*fraction),
                    GridTrack::Length(_)
                    | GridTrack::Percentage(_)
                    | GridTrack::Auto
                    | GridTrack::MinContent
                    | GridTrack::MaxContent
                    | GridTrack::FitContent(_)
                    | GridTrack::MinMax { .. } => None,
                })
                .sum::<f32>();
            let fractional_share = fraction / total_fraction.max(fraction);
            if super::collect_flex::flex_main_size(style).is_none() {
                return FlexBasis::Percentage(fractional_share);
            }
            let fixed_tracks = tracks
                .iter()
                .filter_map(|track| match track {
                    GridTrack::Length(length) => Some(*length),
                    GridTrack::Fraction(_)
                    | GridTrack::Percentage(_)
                    | GridTrack::Auto
                    | GridTrack::MinContent
                    | GridTrack::MaxContent
                    | GridTrack::FitContent(_)
                    | GridTrack::MinMax { .. } => None,
                })
                .sum::<f32>();
            let gaps = resolve_column_gap(style) * tracks.len().saturating_sub(1) as f32;
            let percentage_space = tracks
                .iter()
                .filter_map(|track| match track {
                    GridTrack::Percentage(percentage) => Some(*percentage),
                    _ => None,
                })
                .sum::<f32>();
            FlexBasis::GridFraction {
                fraction: fractional_share,
                fixed_space: fixed_tracks + gaps + 1.0,
                percentage_space,
            }
        }
    }
}

pub(super) fn item_grow(style: &TextStyle, item_index: usize) -> f32 {
    if style.grid_template_columns.is_empty() {
        return 0.0;
    }
    let Some(track) = style
        .grid_template_columns
        .get(item_index % style.grid_template_columns.len())
    else {
        return 0.0;
    };
    if matches!(track, GridTrack::Auto) {
        1.0
    } else {
        0.0
    }
}

fn resolve_column_gap(style: &TextStyle) -> f32 {
    let Some(gap) = style.column_gap.as_deref() else {
        return 0.0;
    };
    let percentage_basis = super::collect_flex::flex_main_size(style).unwrap_or_default();
    let resolve = |component: &str| {
        if let Some(percentage) = ctml_css::parse_css_percentage(component) {
            return (percentage >= 0.0).then_some(percentage_basis * percentage / 100.0);
        }
        ctml_css::parse_nonnegative_font_relative_length_with_viewport(
            component,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
    };
    ctml_css::parse_math_expression(gap, resolve)
        .or_else(|| resolve(gap))
        .filter(|gap| gap.is_finite())
        .unwrap_or_default()
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

#[cfg(test)]
mod tests {
    use super::{item_basis, item_grow};
    use crate::style::{FlexBasis, GridTrack, TextStyle};

    #[test]
    fn fractional_tracks_share_space_remaining_after_fixed_tracks_and_gaps() {
        let mut style = TextStyle::default();
        style.flex_items_row = true;
        style.display_block = Some(true);
        style.viewport_size.width = 400.0;
        style.grid_template_columns = [GridTrack::Length(100.0), GridTrack::Fraction(1.0)].into();
        style.column_gap = Some("20px".to_owned());

        assert!(matches!(item_basis(&style, 0), FlexBasis::Length(width) if width == 100.0));
        assert!(matches!(
            item_basis(&style, 1),
            FlexBasis::GridFraction {
                fraction,
                fixed_space,
                percentage_space
            } if (fraction - 1.0).abs() < 0.001
                && (fixed_space - 121.0).abs() < 0.001
                && percentage_space == 0.0
        ));

        style.grid_template_columns = [GridTrack::Auto].into();
        assert!(matches!(item_basis(&style, 0), FlexBasis::MaxContent));
        assert_eq!(item_grow(&style, 0), 1.0);
        style.grid_template_columns = [GridTrack::MinContent, GridTrack::MaxContent].into();
        assert!(matches!(item_basis(&style, 0), FlexBasis::MinContent));
        assert!(matches!(item_basis(&style, 1), FlexBasis::MaxContent));
        assert_eq!(item_grow(&style, 1), 0.0);
    }
}
