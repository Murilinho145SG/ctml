use super::{FlexGapLine, FlexGapMarker};

pub(super) fn resolve_flex_basis_calculation(
    expression: &str,
    percentage_basis: f32,
    marker: &FlexGapMarker,
) -> Option<f32> {
    let value = ctml_css::parse_math_expression(expression, |component| {
        ctml_css::parse_css_percentage(component)
            .map(|percentage| percentage_basis * percentage / 100.0)
            .or_else(|| {
                ctml_css::parse_nonnegative_font_relative_length_with_viewport(
                    component,
                    marker.font_size,
                    marker.line_height,
                    marker.font_relative_units,
                    marker.viewport_size,
                )
            })
    })?;
    (value.is_finite() && value >= 0.0).then_some(value)
}

pub(super) fn item_intrinsic_size(
    marker_position: usize,
    group: &FlexGapLine,
    glyphs: Option<&Vec<(usize, usize, f32)>>,
    paragraph_text: Option<&String>,
    min_content: bool,
) -> f32 {
    let Some(glyphs) = glyphs else {
        return 0.0;
    };
    let item_start = group
        .markers
        .iter()
        .filter(|previous| previous.position < marker_position)
        .max_by_key(|previous| previous.position)
        .map(|previous| previous.position + '\u{200b}'.len_utf8())
        .unwrap_or_else(|| glyphs.iter().map(|glyph| glyph.0).min().unwrap_or_default());
    let item_glyphs = glyphs
        .iter()
        .filter(|(start, end, _)| *start >= item_start && *end <= marker_position)
        .collect::<Vec<_>>();
    if !min_content {
        return item_glyphs.iter().map(|(_, _, width)| *width).sum();
    }

    let mut current_word = 0.0;
    let mut min_content_size = 0.0f32;
    for (start, end, width) in item_glyphs {
        let is_break = paragraph_text
            .and_then(|text| text.get(*start..*end))
            .is_some_and(|text| text.chars().all(char::is_whitespace));
        if is_break {
            min_content_size = min_content_size.max(current_word);
            current_word = 0.0;
        } else {
            current_word += width;
        }
    }
    min_content_size.max(current_word)
}

pub(super) fn fit_content_size(min_content: f32, max_content: f32, available_size: f32) -> f32 {
    max_content.min(available_size.max(min_content))
}

pub(super) fn resolve_flex_size_limits(
    min_value: Option<&str>,
    max_value: Option<&str>,
    percentage_basis: Option<f32>,
    marker: &FlexGapMarker,
) -> (Option<f32>, Option<f32>) {
    let min = min_value.and_then(|value| resolve_flex_constraint(value, percentage_basis, marker));
    let max = max_value.and_then(|value| resolve_flex_constraint(value, percentage_basis, marker));
    (min, max)
}

pub(super) fn clamp_flex_size(size: f32, limits: (Option<f32>, Option<f32>)) -> f32 {
    let (min, max) = limits;
    match (min, max) {
        (Some(min), Some(max)) if min > max => size.max(min),
        (Some(min), Some(max)) => size.max(min).min(max),
        (Some(min), None) => size.max(min),
        (None, Some(max)) => size.min(max),
        (None, None) => size,
    }
}

pub(super) fn distribute_flex_sizes(
    markers: &[&FlexGapMarker],
    item_sizes: &[(f32, f32)],
    size_limits: &[(Option<f32>, Option<f32>)],
    free_space: f32,
) -> Vec<f32> {
    let mut targets = item_sizes
        .iter()
        .zip(size_limits)
        .map(|((_, base), limits)| clamp_flex_size(*base, *limits))
        .collect::<Vec<_>>();
    let grow = free_space > 0.0;
    let mut remaining_space = if grow {
        // Keep one pixel free so shaped advances cannot push the last item onto a new line.
        (free_space - 1.0).max(0.0)
    } else {
        free_space
    };
    let mut active = markers
        .iter()
        .enumerate()
        .map(|(index, marker)| {
            if grow {
                marker.grow > 0.0
            } else {
                marker.shrink > 0.0 && targets[index] > size_limits[index].0.unwrap_or(0.0)
            }
        })
        .collect::<Vec<_>>();

    if remaining_space != 0.0 {
        for _ in 0..markers.len() {
            let weights = markers
                .iter()
                .enumerate()
                .map(|(index, marker)| {
                    if !active[index] {
                        0.0
                    } else if grow {
                        marker.grow
                    } else {
                        marker.shrink * item_sizes[index].1
                    }
                })
                .collect::<Vec<_>>();
            let total_weight = weights.iter().sum::<f32>();
            if total_weight <= 0.0 {
                break;
            }
            let proposals = targets
                .iter()
                .zip(&weights)
                .map(|(target, weight)| target + remaining_space * weight / total_weight)
                .collect::<Vec<_>>();
            let clamped = proposals
                .iter()
                .zip(size_limits)
                .map(|(proposal, limits)| clamp_flex_size(*proposal, *limits))
                .collect::<Vec<_>>();
            let newly_frozen = (0..markers.len())
                .filter(|index| {
                    active[*index] && (clamped[*index] - proposals[*index]).abs() > 0.001
                })
                .collect::<Vec<_>>();
            if newly_frozen.is_empty() {
                targets = clamped;
                remaining_space = 0.0;
                break;
            }

            let mut consumed = 0.0;
            for index in newly_frozen {
                consumed += clamped[index] - targets[index];
                targets[index] = clamped[index];
                active[index] = false;
            }
            remaining_space -= consumed;
            if remaining_space.abs() <= 0.001 {
                break;
            }
        }
    }

    if grow && remaining_space > 0.0 {
        let justify_count = markers.iter().filter(|marker| marker.justify).count();
        let gap_extra = remaining_space / justify_count.max(1) as f32;
        for (index, marker) in markers.iter().enumerate() {
            if marker.justify {
                targets[index] += gap_extra;
            }
        }
    }
    targets
}

fn resolve_flex_constraint(
    value: &str,
    percentage_basis: Option<f32>,
    marker: &FlexGapMarker,
) -> Option<f32> {
    let resolve_component = |component: &str| {
        ctml_css::parse_css_percentage(component)
            .and_then(|percentage| percentage_basis.map(|basis| basis * percentage / 100.0))
            .or_else(|| {
                ctml_css::parse_nonnegative_font_relative_length_with_viewport(
                    component,
                    marker.font_size,
                    marker.line_height,
                    marker.font_relative_units,
                    marker.viewport_size,
                )
            })
    };
    ctml_css::parse_math_expression(value, resolve_component)
        .or_else(|| resolve_component(value))
        .filter(|size| size.is_finite() && *size >= 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_content_uses_the_longest_whitespace_separated_segment() {
        let group = FlexGapLine {
            markers: vec![],
            available_width: 100.0,
            basis_size: Some(100.0),
            container_id: Some(1),
            mixed_containers: false,
        };
        let glyphs = (0..7)
            .map(|index| (index, index + 1, 1.0))
            .collect::<Vec<_>>();

        assert_eq!(
            item_intrinsic_size(7, &group, Some(&glyphs), Some(&"one two".to_owned()), false),
            7.0
        );
        assert_eq!(
            item_intrinsic_size(7, &group, Some(&glyphs), Some(&"one two".to_owned()), true),
            3.0
        );
    }

    #[test]
    fn fit_content_clamps_max_content_to_available_space_with_min_content_floor() {
        assert_eq!(fit_content_size(20.0, 100.0, 60.0), 60.0);
        assert_eq!(fit_content_size(20.0, 100.0, 120.0), 100.0);
        assert_eq!(fit_content_size(20.0, 100.0, 10.0), 20.0);
    }

    #[test]
    fn flex_growth_redistributes_space_after_an_item_hits_maximum() {
        let markers = [test_marker(1.0, 1.0), test_marker(1.0, 1.0)];
        let marker_refs = markers.iter().collect::<Vec<_>>();
        let targets = distribute_flex_sizes(
            &marker_refs,
            &[(20.0, 20.0), (20.0, 20.0)],
            &[(None, Some(25.0)), (None, None)],
            60.0,
        );
        assert_eq!(targets, vec![25.0, 74.0]);
    }

    #[test]
    fn flex_shrink_redistributes_space_after_an_item_hits_minimum() {
        let markers = [test_marker(0.0, 1.0), test_marker(0.0, 1.0)];
        let marker_refs = markers.iter().collect::<Vec<_>>();
        let targets = distribute_flex_sizes(
            &marker_refs,
            &[(80.0, 80.0), (80.0, 80.0)],
            &[(Some(70.0), None), (None, None)],
            -60.0,
        );
        assert_eq!(targets, vec![70.0, 30.0]);
    }

    fn test_marker(grow: f32, shrink: f32) -> FlexGapMarker {
        FlexGapMarker {
            position: 0,
            grow,
            shrink,
            basis: None,
            min_main_size: None,
            max_main_size: None,
            font_size: 16.0,
            line_height: 20.0,
            font_relative_units: ctml_css::FontRelativeUnits::default(),
            viewport_size: ctml_css::ViewportSize::default(),
            justify: false,
        }
    }
}
