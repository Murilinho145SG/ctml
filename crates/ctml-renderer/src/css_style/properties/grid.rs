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
    if value.is_empty() || value == "none" {
        return Some(Vec::new());
    }
    let mut tracks = Vec::new();
    for token in split_grid_tracks(value)? {
        if let Some(repeat) = token
            .strip_prefix("repeat(")
            .and_then(|value| value.strip_suffix(')'))
        {
            let (count, track) = repeat.split_once(',')?;
            let count = count.trim().parse::<usize>().ok()?;
            let track = parse_track(track.trim(), style)?;
            if count == 0 || tracks.len().saturating_add(count) > 64 {
                return None;
            }
            tracks.extend(std::iter::repeat_n(track, count));
        } else {
            tracks.push(parse_track(token, style)?);
        }
        if tracks.len() > 64 {
            return None;
        }
    }
    Some(tracks)
}

fn split_grid_tracks(value: &str) -> Option<Vec<&str>> {
    let mut tracks = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            whitespace if whitespace.is_ascii_whitespace() && depth == 0 => {
                if let Some(start) = start.take() {
                    tracks.push(&value[start..index]);
                }
                continue;
            }
            _ => {}
        }
        start.get_or_insert(index);
    }
    if depth != 0 {
        return None;
    }
    if let Some(start) = start {
        tracks.push(&value[start..]);
    }
    (!tracks.is_empty()).then_some(tracks)
}

fn parse_track(value: &str, style: &TextStyle) -> Option<GridTrack> {
    match value {
        "auto" => return Some(GridTrack::Auto),
        "min-content" => return Some(GridTrack::MinContent),
        "max-content" => return Some(GridTrack::MaxContent),
        _ => {}
    }
    if let Some(bounds) = value
        .strip_prefix("minmax(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let (min, max) = bounds.split_once(',')?;
        let GridTrack::Length(min) = parse_track(min.trim(), style)? else {
            return None;
        };
        let GridTrack::Length(max) = parse_track(max.trim(), style)? else {
            return None;
        };
        return Some(GridTrack::MinMax {
            min,
            max: max.max(min),
        });
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

        apply_grid_property(
            &mut style,
            "auto min-content max-content fit-content(55px) minmax(60px, 120px)",
        );
        assert_eq!(
            style.grid_template_columns,
            [
                GridTrack::Auto,
                GridTrack::MinContent,
                GridTrack::MaxContent,
                GridTrack::FitContent(55.0),
                GridTrack::MinMax {
                    min: 60.0,
                    max: 120.0
                }
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
