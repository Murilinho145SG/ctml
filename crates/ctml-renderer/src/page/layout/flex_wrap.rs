use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{page::PageRasterizer, style::FlexAlignment};

#[derive(Clone, Copy)]
struct WrapMarker {
    run_index: usize,
    paragraph: usize,
    byte_offset: usize,
    marker_len: usize,
    container_id: Option<usize>,
    gap: f32,
    cross_size: Option<f32>,
    alignment: Option<FlexAlignment>,
}

#[derive(Clone, Copy)]
struct WrappedBreak {
    run_index: usize,
    previous_line: usize,
    previous_line_height: f32,
    gap: f32,
}

pub(super) fn apply_wrapped_row_gaps(page: &mut PageRasterizer) -> bool {
    let mut glyphs_by_paragraph = HashMap::<usize, Vec<(usize, usize, usize, f32)>>::new();
    let mut line_heights_by_paragraph = HashMap::<usize, BTreeMap<usize, f32>>::new();
    let mut visual_line = 0;
    for line in page.buffer.layout_runs() {
        line_heights_by_paragraph
            .entry(line.line_i)
            .or_default()
            .insert(visual_line, line.line_height);
        for glyph in line.glyphs {
            glyphs_by_paragraph.entry(line.line_i).or_default().push((
                glyph.start,
                glyph.end,
                visual_line,
                line.line_height,
            ));
        }
        visual_line += 1;
    }

    let mut markers = Vec::new();
    let mut paragraph = 0;
    let mut byte_offset = 0;
    for (run_index, run) in page.runs.iter().enumerate() {
        for character in run.text.chars() {
            if character == '\n' {
                paragraph += 1;
                byte_offset = 0;
                continue;
            }
            if run.style.flex_wrap_break
                && !run.style.flex_wrap_gap_applied
                && character == '\u{200b}'
                && (run.style.flex_wrap_gap_px > 0.0
                    || (run.style.flex_wrap_cross_size_px.is_some()
                        && run.style.align_content == Some(FlexAlignment::SpaceBetween)))
            {
                markers.push(WrapMarker {
                    run_index,
                    paragraph,
                    byte_offset,
                    marker_len: character.len_utf8(),
                    container_id: run.element_path.last().copied(),
                    gap: run.style.flex_wrap_gap_px,
                    cross_size: run.style.flex_wrap_cross_size_px,
                    alignment: run.style.align_content,
                });
            }
            byte_offset += character.len_utf8();
        }
    }

    let mut breaks_by_container = HashMap::<(usize, Option<usize>), Vec<WrappedBreak>>::new();
    for marker in &markers {
        let Some(glyphs) = glyphs_by_paragraph.get(&marker.paragraph) else {
            continue;
        };
        let marker_line = glyphs
            .iter()
            .find(|(start, end, _, _)| *start <= marker.byte_offset && marker.byte_offset < *end)
            .map(|(_, _, line, _)| *line);
        let previous = glyphs
            .iter()
            .filter(|(_, end, _, _)| *end <= marker.byte_offset)
            .max_by_key(|(start, _, _, _)| *start);
        let next_offset = marker.byte_offset + marker.marker_len;
        let next = glyphs
            .iter()
            .filter(|(start, _, _, _)| *start >= next_offset)
            .min_by_key(|(start, _, _, _)| *start);
        let (Some(marker_line), Some(previous), Some(next)) = (marker_line, previous, next) else {
            continue;
        };
        if previous.2 == next.2 || marker_line != previous.2 {
            continue;
        }
        breaks_by_container
            .entry((marker.paragraph, marker.container_id))
            .or_default()
            .push(WrappedBreak {
                run_index: marker.run_index,
                previous_line: previous.2,
                previous_line_height: previous.3,
                gap: marker.gap,
            });
    }

    let mut extra_by_container = HashMap::new();
    for ((paragraph, container_id), wrapped_breaks) in &breaks_by_container {
        let Some(marker) = markers
            .iter()
            .find(|marker| marker.paragraph == *paragraph && marker.container_id == *container_id)
        else {
            continue;
        };
        if marker.alignment != Some(FlexAlignment::SpaceBetween) {
            continue;
        }
        let Some(cross_size) = marker.cross_size else {
            continue;
        };
        let Some(line_heights) = line_heights_by_paragraph.get(paragraph) else {
            continue;
        };
        let used_size = line_heights.values().sum::<f32>();
        let base_gaps = wrapped_breaks
            .iter()
            .map(|wrapped| wrapped.gap)
            .sum::<f32>();
        let free_space = (cross_size - used_size - base_gaps).max(0.0);
        extra_by_container.insert(
            (*paragraph, *container_id),
            free_space / wrapped_breaks.len().max(1) as f32,
        );
    }

    let mut applied_lines = HashSet::new();
    let mut changed = false;
    for ((paragraph, container_id), wrapped_breaks) in breaks_by_container {
        let extra = extra_by_container
            .get(&(paragraph, container_id))
            .copied()
            .unwrap_or_default();
        for wrapped in wrapped_breaks {
            if !applied_lines.insert((paragraph, wrapped.previous_line)) {
                continue;
            }
            if let Some(run) = page.runs.get_mut(wrapped.run_index) {
                run.style.line_height = wrapped.previous_line_height + wrapped.gap + extra;
                run.style.line_height_normal = false;
                run.style.flex_wrap_gap_applied = true;
                changed = true;
            }
        }
    }
    changed
}
