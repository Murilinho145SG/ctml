use std::collections::HashMap;

use crate::{page::PageRasterizer, style::FlexBasis};

mod sizing;
use sizing::{
    clamp_flex_size, distribute_flex_sizes, fit_content_size, item_intrinsic_size,
    resolve_flex_basis_calculation, resolve_flex_size_limits,
};

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
    min_main_size: Option<String>,
    max_main_size: Option<String>,
    font_size: f32,
    line_height: f32,
    font_relative_units: ctml_css::FontRelativeUnits,
    viewport_size: ctml_css::ViewportSize,
    grid_item: bool,
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
                        min_main_size: run.style.flex_min_main_size.clone(),
                        max_main_size: run.style.flex_max_main_size.clone(),
                        font_size: run.style.font_size,
                        line_height: run.style.line_height,
                        font_relative_units: run.style.font_relative_units,
                        viewport_size: run.style.viewport_size,
                        grid_item: run.style.display_grid,
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
        let mut size_limits = Vec::with_capacity(line_markers.len());
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
                Some(FlexBasis::GridFitContent(limit)) => {
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
                    fit_content_size(min_content, max_content, *limit)
                }
                Some(FlexBasis::GridMinMax { min, max }) => {
                    let max_content = item_intrinsic_size(
                        marker.position,
                        group,
                        glyphs_by_paragraph.get(&line.line_i),
                        text_by_paragraph.get(&line.line_i),
                        false,
                    );
                    max_content.max(*min).min(*max)
                }
                Some(FlexBasis::Percentage(percentage)) => group
                    .basis_size
                    .map_or(content_size, |size| size * percentage),
                Some(FlexBasis::GridFraction { .. }) => content_size,
                Some(FlexBasis::Calculation(expression)) => group
                    .basis_size
                    .and_then(|basis_size| {
                        resolve_flex_basis_calculation(expression, basis_size, marker)
                    })
                    .map_or(content_size, |size| size),
                Some(FlexBasis::Auto) | None => content_size,
            };
            let mut limits = resolve_flex_size_limits(
                marker.min_main_size.as_deref(),
                marker.max_main_size.as_deref(),
                group.basis_size,
                marker,
            );
            if !marker.grid_item
                && marker
                    .min_main_size
                    .as_deref()
                    .is_none_or(|value| value.eq_ignore_ascii_case("auto"))
            {
                let min_content = item_intrinsic_size(
                    marker.position,
                    group,
                    glyphs_by_paragraph.get(&line.line_i),
                    text_by_paragraph.get(&line.line_i),
                    true,
                );
                limits.0 = Some(min_content);
            }
            let base_size = clamp_flex_size(base_size, limits);
            item_sizes.push((content_size, base_size));
            size_limits.push(limits);
            item_start = marker.position + '\u{200b}'.len_utf8();
        }
        let intrinsic_track_size = line_markers
            .iter()
            .zip(&item_sizes)
            .filter_map(|(marker, (_, base_size))| match marker.basis.as_ref() {
                Some(
                    FlexBasis::Length(_)
                    | FlexBasis::Percentage(_)
                    | FlexBasis::GridFraction { .. },
                )
                | None => None,
                Some(_) => Some(*base_size),
            })
            .sum::<f32>();
        for (index, marker) in line_markers.iter().enumerate() {
            if let Some(FlexBasis::GridFraction {
                fraction,
                fixed_space,
                percentage_space,
            }) = marker.basis.as_ref()
            {
                item_sizes[index].1 = group.basis_size.map_or(item_sizes[index].0, |size| {
                    (size - fixed_space - size * percentage_space - intrinsic_track_size).max(0.0)
                        * fraction
                });
            }
        }
        let basis_adjustment = item_sizes
            .iter()
            .map(|(content_size, base_size)| base_size - content_size)
            .sum::<f32>();
        let free_space = group.available_width - used_width - basis_adjustment;
        let target_sizes =
            distribute_flex_sizes(&line_markers, &item_sizes, &size_limits, free_space);
        for (index, marker) in line_markers.iter().enumerate() {
            let extra = target_sizes[index] - item_sizes[index].0;
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
