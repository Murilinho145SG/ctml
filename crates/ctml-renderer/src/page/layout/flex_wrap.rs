use std::collections::HashMap;

use crate::page::PageRasterizer;

pub(super) fn apply_wrapped_row_gaps(page: &mut PageRasterizer) -> bool {
    let mut glyphs_by_paragraph = HashMap::<usize, Vec<(usize, usize, usize, f32)>>::new();
    let mut visual_line = 0;
    for line in page.buffer.layout_runs() {
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
                && run.style.flex_wrap_gap_px > 0.0
                && run.style.writing_mode == crate::style::WritingMode::HorizontalTb
                && character == '\u{200b}'
            {
                markers.push((run_index, paragraph, byte_offset, character.len_utf8()));
            }
            byte_offset += character.len_utf8();
        }
    }

    let mut applied_lines = HashMap::new();
    for (run_index, paragraph, marker_offset, marker_len) in markers {
        let Some(glyphs) = glyphs_by_paragraph.get(&paragraph) else {
            continue;
        };
        let marker_line = glyphs
            .iter()
            .find(|(start, end, _, _)| *start <= marker_offset && marker_offset < *end)
            .map(|(_, _, line, _)| *line);
        let previous = glyphs
            .iter()
            .filter(|(_, end, _, _)| *end <= marker_offset)
            .max_by_key(|(start, _, _, _)| *start);
        let next_offset = marker_offset + marker_len;
        let next = glyphs
            .iter()
            .filter(|(start, _, _, _)| *start >= next_offset)
            .min_by_key(|(start, _, _, _)| *start);
        let (Some(marker_line), Some(previous), Some(next)) = (marker_line, previous, next) else {
            continue;
        };
        if previous.2 == next.2
            || marker_line != previous.2
            || applied_lines.insert((paragraph, previous.2), ()).is_some()
        {
            continue;
        }
        let Some(run) = page.runs.get_mut(run_index) else {
            continue;
        };
        run.style.line_height = previous.3 + run.style.flex_wrap_gap_px;
        run.style.line_height_normal = false;
        run.style.flex_wrap_gap_applied = true;
        return true;
    }
    false
}
