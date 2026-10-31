use std::collections::HashMap;

use crate::page::PageRasterizer;

struct FlexGapLine {
    markers: Vec<FlexGapMarker>,
    available_width: f32,
    container_id: Option<usize>,
    mixed_containers: bool,
}

#[derive(Clone, Copy)]
struct FlexGapMarker {
    position: usize,
    grow: f32,
    shrink: f32,
    justify: bool,
}

pub(super) fn apply_distributed_flex_gaps(page: &mut PageRasterizer, content_width: f32) -> bool {
    let mut markers_by_line = HashMap::<usize, FlexGapLine>::new();
    let mut line_index = 0;
    let mut byte_index = 0;
    for run in &page.runs {
        for character in run.text.chars() {
            if character == '\n' {
                line_index += 1;
                byte_index = 0;
            } else {
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
        let free_space = group.available_width - used_width;
        // Keep one pixel free so shaped advances cannot push the last item onto a new line.
        let grow_space = (free_space - 1.0).max(0.0);
        let total_grow = line_markers
            .iter()
            .map(|marker| f64::from(marker.grow))
            .sum::<f64>();
        let justify_count = line_markers.iter().filter(|marker| marker.justify).count();
        let mut shrink_weights = Vec::new();
        if free_space < 0.0 {
            let mut item_start = line
                .glyphs
                .iter()
                .map(|glyph| glyph.start)
                .min()
                .unwrap_or_default();
            for marker in &line_markers {
                let base_size = line
                    .glyphs
                    .iter()
                    .filter(|glyph| glyph.start >= item_start && glyph.end <= marker.position)
                    .map(|glyph| glyph.w)
                    .sum::<f32>();
                shrink_weights.push((marker.shrink, base_size));
                item_start = marker.position + '\u{200b}'.len_utf8();
            }
        }
        let total_shrink = shrink_weights
            .iter()
            .map(|(factor, base_size)| f64::from(*factor) * f64::from(*base_size))
            .sum::<f64>();
        for (index, marker) in line_markers.iter().enumerate() {
            let extra = if grow_space > 0.0 && total_grow > 0.0 {
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
