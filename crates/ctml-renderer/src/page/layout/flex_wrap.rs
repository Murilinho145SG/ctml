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
    reverse: bool,
    applied: bool,
    adjust_line_height: bool,
}

#[derive(Clone, Copy)]
struct WrappedBreak {
    run_index: usize,
    previous_line: usize,
    previous_line_height: f32,
    gap: f32,
    applied: bool,
    adjust_line_height: bool,
}

#[derive(Clone, Copy)]
struct LeadingSpacer {
    run_index: usize,
    paragraph: usize,
    container_id: Option<usize>,
    cross_size: f32,
    alignment: FlexAlignment,
}

pub(super) fn apply_wrapped_row_gaps(page: &mut PageRasterizer) -> bool {
    let mut glyphs_by_paragraph = HashMap::<usize, Vec<(usize, usize, usize, f32)>>::new();
    let mut line_heights_by_paragraph = HashMap::<usize, BTreeMap<usize, f32>>::new();
    let mut line_tops_by_paragraph = HashMap::<usize, BTreeMap<usize, f32>>::new();
    let mut line_keys_by_visual = HashMap::<usize, (usize, usize)>::new();
    let mut visual_line = 0;
    for line in page.buffer.layout_runs() {
        line_heights_by_paragraph
            .entry(line.line_i)
            .or_default()
            .insert(visual_line, line.line_height);
        line_tops_by_paragraph
            .entry(line.line_i)
            .or_default()
            .insert(visual_line, line.line_top);
        if let Some(glyph) = line.glyphs.first() {
            line_keys_by_visual.insert(visual_line, (line.line_i, glyph.start));
        }
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
    let mut leading_spacers = Vec::new();
    let mut paragraph = 0;
    let mut byte_offset = 0;
    for (run_index, run) in page.runs.iter().enumerate() {
        let alignment = run.style.align_content.unwrap_or(FlexAlignment::Stretch);
        if run.style.flex_wrap_leading_spacer
            && let Some(cross_size) = run.style.flex_wrap_cross_size_px
            && let Some(alignment) = run.style.align_content
        {
            leading_spacers.push(LeadingSpacer {
                run_index,
                paragraph: paragraph + run.text.matches('\n').count(),
                container_id: run.element_path.last().copied(),
                cross_size,
                alignment,
            });
        }
        for character in run.text.chars() {
            if character == '\n' {
                paragraph += 1;
                byte_offset = 0;
                continue;
            }
            if run.style.flex_wrap_break
                && (!run.style.flex_wrap_gap_applied || run.style.flex_wrap_reverse)
                && character == '\u{200b}'
                && (run.style.flex_wrap_reverse
                    || run.style.flex_wrap_gap_px > 0.0
                    || (run.style.flex_wrap_cross_size_px.is_some()
                        && matches!(
                            alignment,
                            FlexAlignment::SpaceBetween
                                | FlexAlignment::Stretch
                                | FlexAlignment::SpaceAround
                                | FlexAlignment::SpaceEvenly
                        )))
            {
                markers.push(WrapMarker {
                    run_index,
                    paragraph,
                    byte_offset,
                    marker_len: character.len_utf8(),
                    container_id: run.element_path.last().copied(),
                    gap: run.style.flex_wrap_gap_px,
                    cross_size: run.style.flex_wrap_cross_size_px,
                    alignment: Some(alignment),
                    reverse: run.style.flex_wrap_reverse,
                    applied: run.style.flex_wrap_gap_applied,
                    adjust_line_height: run.style.flex_wrap_gap_px > 0.0
                        || (run.style.flex_wrap_cross_size_px.is_some()
                            && matches!(
                                alignment,
                                FlexAlignment::SpaceBetween
                                    | FlexAlignment::Stretch
                                    | FlexAlignment::SpaceAround
                                    | FlexAlignment::SpaceEvenly
                            )),
                });
            }
            byte_offset += character.len_utf8();
        }
    }

    let mut breaks_by_container = HashMap::<(usize, Option<usize>), Vec<WrappedBreak>>::new();
    let mut reverse_lines_by_container =
        HashMap::<(usize, Option<usize>), (usize, usize, f32, FlexAlignment)>::new();
    let mut row_markers_by_container =
        HashMap::<(usize, Option<usize>), BTreeMap<usize, (usize, f32, bool)>>::new();
    for marker in &markers {
        let Some(glyphs) = glyphs_by_paragraph.get(&marker.paragraph) else {
            continue;
        };
        let marker_line = glyphs
            .iter()
            .find(|(start, end, _, _)| *start <= marker.byte_offset && marker.byte_offset < *end)
            .map(|(_, _, line, _)| *line);
        if marker.alignment == Some(FlexAlignment::Stretch)
            && marker.cross_size.is_some()
            && let Some(line) = marker_line
            && let Some((_, _, _, line_height)) = glyphs.iter().find(|(start, end, _, _)| {
                *start <= marker.byte_offset && marker.byte_offset < *end
            })
        {
            row_markers_by_container
                .entry((marker.paragraph, marker.container_id))
                .or_default()
                .entry(line)
                .or_insert((marker.run_index, *line_height, marker.applied));
        }
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
        if marker.reverse {
            let lines = reverse_lines_by_container
                .entry((marker.paragraph, marker.container_id))
                .or_insert((
                    previous.2,
                    next.2,
                    marker.cross_size.unwrap_or_default(),
                    marker.alignment.unwrap_or(FlexAlignment::Stretch),
                ));
            lines.0 = lines.0.min(previous.2).min(next.2);
            lines.1 = lines.1.max(previous.2).max(next.2);
            if let Some(cross_size) = marker.cross_size {
                lines.2 = cross_size;
            }
        }
        breaks_by_container
            .entry((marker.paragraph, marker.container_id))
            .or_default()
            .push(WrappedBreak {
                run_index: marker.run_index,
                previous_line: previous.2,
                previous_line_height: previous.3,
                gap: marker.gap,
                applied: marker.applied,
                adjust_line_height: marker.adjust_line_height,
            });
    }

    let mut stretch_extra_by_container = HashMap::new();
    for (key @ (paragraph, _), rows) in &row_markers_by_container {
        let Some(marker) = markers
            .iter()
            .find(|marker| (marker.paragraph, marker.container_id) == *key)
        else {
            continue;
        };
        let Some(cross_size) = marker.cross_size else {
            continue;
        };
        let Some(line_heights) = line_heights_by_paragraph.get(paragraph) else {
            continue;
        };
        let used_size = line_heights.values().sum::<f32>();
        let base_gaps = breaks_by_container
            .get(key)
            .into_iter()
            .flatten()
            .map(|wrapped| wrapped.gap)
            .sum::<f32>();
        let free_space = (cross_size - used_size - base_gaps).max(0.0);
        stretch_extra_by_container.insert(*key, free_space / rows.len().max(1) as f32);
    }

    let mut extra_by_container = HashMap::new();
    for ((paragraph, container_id), wrapped_breaks) in &breaks_by_container {
        let Some(marker) = markers
            .iter()
            .find(|marker| marker.paragraph == *paragraph && marker.container_id == *container_id)
        else {
            continue;
        };
        if !matches!(
            marker.alignment,
            Some(
                FlexAlignment::SpaceBetween
                    | FlexAlignment::SpaceAround
                    | FlexAlignment::SpaceEvenly
            )
        ) {
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
        let row_count = line_heights.len().max(1) as f32;
        let extra = match marker.alignment {
            Some(FlexAlignment::SpaceBetween) => free_space / wrapped_breaks.len().max(1) as f32,
            Some(FlexAlignment::SpaceAround) => free_space / row_count,
            Some(FlexAlignment::SpaceEvenly) => free_space / (row_count + 1.0),
            _ => 0.0,
        };
        extra_by_container.insert((*paragraph, *container_id), extra);
    }

    let mut leading_offsets_by_container = HashMap::new();
    let mut leading_amounts_by_container = HashMap::new();
    for spacer in leading_spacers {
        let key = (spacer.paragraph, spacer.container_id);
        let Some(line_heights) = line_heights_by_paragraph.get(&spacer.paragraph) else {
            continue;
        };
        let row_count = line_heights.len();
        if row_count < 2 {
            continue;
        }
        let used_size = line_heights.values().sum::<f32>();
        let base_gaps = breaks_by_container
            .get(&key)
            .into_iter()
            .flatten()
            .map(|wrapped| wrapped.gap)
            .sum::<f32>();
        let free_space = (spacer.cross_size - used_size - base_gaps).max(0.0);
        let leading = match spacer.alignment {
            FlexAlignment::Center => free_space / 2.0,
            FlexAlignment::FlexEnd | FlexAlignment::End => free_space,
            FlexAlignment::SpaceAround => free_space / (2.0 * row_count as f32),
            FlexAlignment::SpaceEvenly => free_space / (row_count as f32 + 1.0),
            _ => 0.0,
        };
        leading_amounts_by_container.insert(key, leading);
        leading_offsets_by_container.insert(key, (spacer.run_index, leading));
    }

    let mut applied_lines = HashSet::new();
    let mut changed = false;
    for ((paragraph, container_id), wrapped_breaks) in breaks_by_container {
        let stretch_extra = stretch_extra_by_container
            .get(&(paragraph, container_id))
            .copied()
            .unwrap_or_default();
        let extra = extra_by_container
            .get(&(paragraph, container_id))
            .copied()
            .unwrap_or_default();
        for wrapped in wrapped_breaks {
            if wrapped.applied || !wrapped.adjust_line_height {
                continue;
            }
            if !applied_lines.insert((paragraph, wrapped.previous_line)) {
                continue;
            }
            if let Some(run) = page.runs.get_mut(wrapped.run_index) {
                run.style.line_height =
                    wrapped.previous_line_height + wrapped.gap + extra + stretch_extra;
                run.style.line_height_normal = false;
                run.style.flex_wrap_gap_applied = true;
                changed = true;
            }
        }
    }
    for (key @ (paragraph, _), rows) in row_markers_by_container {
        let stretch_extra = stretch_extra_by_container
            .get(&key)
            .copied()
            .unwrap_or_default();
        for (line, (run_index, line_height, already_applied)) in rows {
            if already_applied {
                continue;
            }
            if !applied_lines.insert((paragraph, line)) {
                continue;
            }
            if let Some(run) = page.runs.get_mut(run_index) {
                run.style.line_height = line_height + stretch_extra;
                run.style.line_height_normal = false;
                run.style.flex_wrap_gap_applied = true;
                changed = true;
            }
        }
    }
    for (run_index, leading) in leading_offsets_by_container.into_values() {
        if let Some(run) = page.runs.get_mut(run_index)
            && (run.style.line_height - leading).abs() > 0.01
        {
            run.style.line_height = leading.max(0.01);
            run.style.line_height_normal = false;
            changed = true;
        }
    }
    page.flex_wrap_line_offsets.clear();
    for ((paragraph, container_id), (first_line, last_line, cross_size, alignment)) in
        reverse_lines_by_container
    {
        let (Some(line_heights), Some(line_tops)) = (
            line_heights_by_paragraph.get(&paragraph),
            line_tops_by_paragraph.get(&paragraph),
        ) else {
            continue;
        };
        let Some(start_top) = line_tops.get(&first_line).copied() else {
            continue;
        };
        let used = (first_line..=last_line)
            .filter_map(|line| line_heights.get(&line))
            .sum::<f32>();
        let extent = if cross_size > 0.0 { cross_size } else { used };
        let free_space = (extent - used).max(0.0);
        let leading = if cross_size > 0.0 {
            match alignment {
                FlexAlignment::FlexStart | FlexAlignment::Start => free_space,
                FlexAlignment::Center | FlexAlignment::SpaceAround | FlexAlignment::SpaceEvenly => {
                    free_space / 2.0
                }
                FlexAlignment::FlexEnd | FlexAlignment::End => 0.0,
                FlexAlignment::SpaceBetween | FlexAlignment::Stretch => 0.0,
            }
        } else {
            0.0
        };
        let natural_leading = leading_amounts_by_container
            .get(&(paragraph, container_id))
            .copied()
            .unwrap_or_default();
        let origin = start_top - natural_leading;
        let mut accumulated = 0.0;
        for line in first_line..=last_line {
            let Some(height) = line_heights.get(&line).copied() else {
                continue;
            };
            accumulated += height;
            let Some(line_top) = line_tops.get(&line).copied() else {
                continue;
            };
            let target_top = origin + leading + used - accumulated;
            let offset = target_top - line_top;
            if offset.abs() > 0.01
                && let Some(key) = line_keys_by_visual.get(&line).copied()
            {
                page.flex_wrap_line_offsets.insert(key, offset);
            }
        }
    }
    changed
}
