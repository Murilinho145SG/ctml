use std::collections::HashMap;

use crate::{page::PageRasterizer, style::FlexBasis};

struct FlexGapLine {
    markers: Vec<FlexGapMarker>,
    available_width: f32,
    basis_size: Option<f32>,
    container_id: Option<usize>,
    mixed_containers: bool,
}

#[derive(Clone)]
struct FlexGapMarker {
    position: usize,
    grow: f32,
    shrink: f32,
    basis: Option<FlexBasis>,
    font_size: f32,
    line_height: f32,
    font_relative_units: ctml_css::FontRelativeUnits,
    viewport_size: ctml_css::ViewportSize,
    justify: bool,
}

pub(super) fn apply_distributed_flex_gaps(page: &mut PageRasterizer, content_width: f32) -> bool {
    let mut markers_by_line = HashMap::<usize, FlexGapLine>::new();
    let mut glyphs_by_paragraph = HashMap::<usize, Vec<(usize, usize, f32)>>::new();
    let mut text_by_paragraph = HashMap::<usize, String>::new();
    for line in page.buffer.layout_runs() {
        glyphs_by_paragraph.entry(line.line_i).or_default().extend(
            line.glyphs
                .iter()
                .map(|glyph| (glyph.start, glyph.end, glyph.w)),
        );
    }
    let mut line_index = 0;
    let mut byte_index = 0;
    for run in &page.runs {
        for character in run.text.chars() {
            if character == '\n' {
                line_index += 1;
                byte_index = 0;
            } else {
                text_by_paragraph
                    .entry(line_index)
                    .or_default()
                    .push(character);
                if character == '\u{200b}' && run.style.flex_distributed_gap {
                    let container_id = run.element_path.last().copied();
                    let line = markers_by_line.entry(line_index).or_insert_with(|| {
                        let writing_mode = page.writing_mode();
                        let inline_size = super::super::sizing::resolve_layout_width(
                            &run.style.layout_sizes,
                            content_width,
                            writing_mode,
                        );
                        let (padding_start, _) = super::super::sizing_padding::inline_padding(
                            &run.style.layout_sizes,
                            content_width,
                            writing_mode,
                        );
                        let (border_start, _) = super::super::sizing_padding::inline_borders(
                            &run.style.layout_sizes,
                            content_width,
                            writing_mode,
                        );
                        let (margin_start, _) = super::super::sizing_padding::inline_margin(
                            &run.style.layout_sizes,
                            content_width,
                            writing_mode,
                        );
                        FlexGapLine {
                            markers: Vec::new(),
                            available_width: inline_size
                                + padding_start
                                + border_start
                                + margin_start,
                            basis_size: flex_basis_container_size(&run.style, inline_size),
                            container_id,
                            mixed_containers: false,
                        }
                    });
                    if line.container_id != container_id {
                        line.mixed_containers = true;
                    }
                    line.markers.push(FlexGapMarker {
                        position: byte_index,
                        grow: run.style.flex_grow_weight,
                        shrink: run.style.flex_shrink_weight,
                        basis: run.style.flex_basis.clone(),
                        font_size: run.style.font_size,
                        line_height: run.style.line_height,
                        font_relative_units: run.style.font_relative_units,
                        viewport_size: run.style.viewport_size,
                        justify: run.style.flex_justify_distributed,
                    });
                }
                byte_index += character.len_utf8();
            }
        }
    }

    let mut extras = HashMap::new();
    for line in page.buffer.layout_runs() {
        let Some(group) = markers_by_line.get(&line.line_i) else {
            continue;
        };
        let Some(container_id) = group.container_id else {
            continue;
        };
        if group.mixed_containers {
            continue;
        }
        let line_markers = group
            .markers
            .iter()
            .filter(|marker| {
                line.glyphs
                    .iter()
                    .any(|glyph| marker.position == glyph.start)
            })
            .collect::<Vec<_>>();
        if line_markers.is_empty() {
            continue;
        }
        let left = line
            .glyphs
            .iter()
            .map(|glyph| glyph.x)
            .fold(f32::INFINITY, f32::min);
        let right = line
            .glyphs
            .iter()
            .map(|glyph| glyph.x + glyph.w)
            .fold(f32::NEG_INFINITY, f32::max);
        let used_width = right - left;
        let mut item_start = line
            .glyphs
            .iter()
            .map(|glyph| glyph.start)
            .min()
            .unwrap_or_default();
        let mut item_sizes = Vec::with_capacity(line_markers.len());
        for marker in &line_markers {
            let content_size = line
                .glyphs
                .iter()
                .filter(|glyph| glyph.start >= item_start && glyph.end <= marker.position)
                .map(|glyph| glyph.w)
                .sum::<f32>();
            let base_size = match marker.basis.as_ref() {
                Some(FlexBasis::Length(length)) => *length,
                Some(FlexBasis::Content | FlexBasis::MaxContent) => item_intrinsic_size(
                    marker.position,
                    group,
                    glyphs_by_paragraph.get(&line.line_i),
                    text_by_paragraph.get(&line.line_i),
                    false,
                ),
                Some(FlexBasis::MinContent) => item_intrinsic_size(
                    marker.position,
                    group,
                    glyphs_by_paragraph.get(&line.line_i),
                    text_by_paragraph.get(&line.line_i),
                    true,
                ),
                Some(FlexBasis::FitContent) => {
                    let max_content = item_intrinsic_size(
                        marker.position,
                        group,
                        glyphs_by_paragraph.get(&line.line_i),
                        text_by_paragraph.get(&line.line_i),
                        false,
                    );
                    let min_content = item_intrinsic_size(
                        marker.position,
                        group,
                        glyphs_by_paragraph.get(&line.line_i),
                        text_by_paragraph.get(&line.line_i),
                        true,
                    );
                    fit_content_size(min_content, max_content, group.available_width)
                }
                Some(FlexBasis::Percentage(percentage)) => group
                    .basis_size
                    .map_or(content_size, |size| size * percentage),
                Some(FlexBasis::Calculation(expression)) => group
                    .basis_size
                    .and_then(|basis_size| {
                        resolve_flex_basis_calculation(expression, basis_size, marker)
                    })
                    .map_or(content_size, |size| size),
                Some(FlexBasis::Auto) | None => content_size,
            };
            item_sizes.push((content_size, base_size));
            item_start = marker.position + '\u{200b}'.len_utf8();
        }
        let basis_adjustment = item_sizes
            .iter()
            .map(|(content_size, base_size)| base_size - content_size)
            .sum::<f32>();
        let free_space = group.available_width - used_width - basis_adjustment;
        // Keep one pixel free so shaped advances cannot push the last item onto a new line.
        let grow_space = (free_space - 1.0).max(0.0);
        let total_grow = line_markers
            .iter()
            .map(|marker| f64::from(marker.grow))
            .sum::<f64>();
        let justify_count = line_markers.iter().filter(|marker| marker.justify).count();
        let shrink_weights = line_markers
            .iter()
            .zip(&item_sizes)
            .map(|(marker, (_, base_size))| (marker.shrink, *base_size))
            .collect::<Vec<_>>();
        let total_shrink = shrink_weights
            .iter()
            .map(|(factor, base_size)| f64::from(*factor) * f64::from(*base_size))
            .sum::<f64>();
        for (index, marker) in line_markers.iter().enumerate() {
            let basis_extra = item_sizes[index].1 - item_sizes[index].0;
            let distributed_extra = if grow_space > 0.0 && total_grow > 0.0 {
                (f64::from(grow_space) * f64::from(marker.grow) / total_grow) as f32
            } else if grow_space > 0.0 && total_grow == 0.0 && marker.justify {
                grow_space / justify_count.max(1) as f32
            } else if free_space < 0.0 && total_shrink > 0.0 {
                let (factor, base_size) = shrink_weights[index];
                let weighted_size = f64::from(factor) * f64::from(base_size);
                -((f64::from(-free_space) * weighted_size / total_shrink) as f32).min(base_size)
            } else {
                0.0
            };
            let extra = basis_extra + distributed_extra;
            extras.insert((line.line_i, container_id, marker.position), extra);
        }
    }

    let mut changed = false;
    let mut line_index = 0;
    let mut byte_index = 0;
    for run in &mut page.runs {
        for character in run.text.chars() {
            if character == '\n' {
                line_index += 1;
                byte_index = 0;
                continue;
            }
            if character == '\u{200b}'
                && run.style.flex_distributed_gap
                && !run.style.flex_distributed_gap_applied
                && let Some(container_id) = run.element_path.last().copied()
                && let Some(extra) = extras.get(&(line_index, container_id, byte_index)).copied()
            {
                run.style.letter_spacing += if run.style.flex_distributed_gap_leading {
                    extra / 2.0
                } else {
                    extra
                };
                run.style.letter_spacing_px = true;
                run.style.flex_distributed_gap_applied = true;
                changed |= extra.abs() > 0.0;
            }
            byte_index += character.len_utf8();
        }
    }
    changed
}

fn flex_basis_container_size(style: &crate::style::TextStyle, inline_size: f32) -> Option<f32> {
    if style.display_block != Some(false) {
        return Some(inline_size);
    }
    let declared_size = style.inline_size.as_deref().or(style.width.as_deref())?;
    (!declared_size.eq_ignore_ascii_case("auto")).then_some(inline_size)
}

fn resolve_flex_basis_calculation(
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

fn item_intrinsic_size(
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

fn fit_content_size(min_content: f32, max_content: f32, available_size: f32) -> f32 {
    max_content.min(available_size.max(min_content))
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
}
