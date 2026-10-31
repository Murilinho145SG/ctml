use crate::style::{GridTrack, TextStyle};

pub(super) fn apply_grid_property(style: &mut TextStyle, value: &str) {
    if let Some(tracks) = parse_grid_template_columns(value, style) {
        style.grid_template_columns = tracks;
    }
}

pub(super) fn reset_grid_property(style: &mut TextStyle) {
    style.grid_template_columns.clear();
}

pub(super) fn inherit_grid_property(style: &mut TextStyle, inherited: &TextStyle) {
    style
        .grid_template_columns
        .clone_from(&inherited.grid_template_columns);
}

fn parse_grid_template_columns(value: &str, style: &TextStyle) -> Option<Vec<GridTrack>> {
    let lowercase = value.trim().to_ascii_lowercase();
    let value = lowercase.as_str();
    let tracks = if let Some(repeat) = value
        .strip_prefix("repeat(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let (count, track) = repeat.split_once(',')?;
        let count = count.trim().parse::<usize>().ok()?;
        let track = parse_track(track.trim(), style)?;
        (count > 0 && count <= 64).then(|| vec![track; count])?
    } else if value.is_empty() || value == "none" {
        return Some(Vec::new());
    } else {
        value
            .split_ascii_whitespace()
            .map(|track| parse_track(track, style))
            .collect::<Option<Vec<_>>>()?
    };
    Some(tracks)
}

fn parse_track(value: &str, style: &TextStyle) -> Option<GridTrack> {
    match value {
        "auto" => return Some(GridTrack::Auto),
        "min-content" => return Some(GridTrack::MinContent),
        "max-content" => return Some(GridTrack::MaxContent),
        _ => {}
    }
    if let Some(limit) = value
        .strip_prefix("fit-content(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let limit = ctml_css::parse_nonnegative_font_relative_length_with_viewport(
            limit.trim(),
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
        .filter(|limit| limit.is_finite())?;
        return Some(GridTrack::FitContent(limit));
    }
    if let Some(fraction) = value
        .strip_suffix("fr")
        .and_then(|fraction| fraction.trim().parse::<f32>().ok())
    {
        return (fraction.is_finite() && fraction > 0.0).then_some(GridTrack::Fraction(fraction));
    }
    if let Some(percentage) = ctml_css::parse_css_percentage(value) {
        return (percentage.is_finite() && percentage >= 0.0)
            .then_some(GridTrack::Percentage(percentage / 100.0));
    }
    ctml_css::parse_nonnegative_font_relative_length_with_viewport(
        value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    )
    .filter(|length| length.is_finite())
    .map(GridTrack::Length)
}

#[cfg(test)]
mod tests {
    use super::{apply_grid_property, reset_grid_property};
    use crate::style::{GridTrack, TextStyle};

    #[test]
    fn parses_fixed_fractional_mixed_and_intrinsic_column_tracks() {
        let mut style = TextStyle::default();
        apply_grid_property(&mut style, "100px 120px");
        assert_eq!(
            style.grid_template_columns,
            [GridTrack::Length(100.0), GridTrack::Length(120.0)]
        );

        apply_grid_property(&mut style, "repeat(3, 1fr)");
        assert_eq!(style.grid_template_columns, [GridTrack::Fraction(1.0); 3]);

        apply_grid_property(&mut style, "100px 2fr 1fr");
        assert_eq!(
            style.grid_template_columns,
            [
                GridTrack::Length(100.0),
                GridTrack::Fraction(2.0),
                GridTrack::Fraction(1.0)
            ]
        );

        apply_grid_property(&mut style, "25% 100px 1fr");
        assert_eq!(
            style.grid_template_columns,
            [
                GridTrack::Percentage(0.25),
                GridTrack::Length(100.0),
                GridTrack::Fraction(1.0)
            ]
        );

        apply_grid_property(&mut style, "auto min-content max-content fit-content(55px)");
        assert_eq!(
            style.grid_template_columns,
            [
                GridTrack::Auto,
                GridTrack::MinContent,
                GridTrack::MaxContent,
                GridTrack::FitContent(55.0)
            ]
        );
    }

    #[test]
    fn invalid_grid_tracks_preserve_the_previous_value_and_initial_clears_it() {
        let mut style = TextStyle::default();
        apply_grid_property(&mut style, "20px");
        apply_grid_property(&mut style, "repeat(0, 1fr)");
        assert_eq!(style.grid_template_columns, [GridTrack::Length(20.0)]);

        reset_grid_property(&mut style);
        assert!(style.grid_template_columns.is_empty());
    }
}
