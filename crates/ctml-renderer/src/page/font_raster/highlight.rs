use std::collections::HashMap;

use cosmic_text::Cursor;

use crate::page::CursorSpan;

pub(super) fn glyph_foreground(
    selection: Option<(Cursor, Cursor)>,
    spans: &[CursorSpan],
    first_line_by_block: &HashMap<usize, (usize, usize)>,
    line: usize,
    glyph_start: usize,
    glyph_end: usize,
) -> Option<[u8; 4]> {
    let first_span = spans
        .partition_point(|span| span.line < line || (span.line == line && span.end <= glyph_start));
    for span in spans[first_span..]
        .iter()
        .take_while(|span| span.line == line && span.start < glyph_end)
    {
        if let Some((start, end)) = selection {
            let (start, end) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };
            if span.end > glyph_start
                && (start.line != line || span.end > start.index)
                && (end.line != line || span.start < end.index)
                && span.selection_foreground.is_some()
            {
                return span.selection_foreground;
            }
        }
        if let Some((first_line, end)) = span
            .first_line_block
            .and_then(|block| first_line_by_block.get(&block))
            && *first_line == line
            && glyph_start < *end
        {
            if span.first_line_foreground.is_some() {
                return span.first_line_foreground;
            }
        }
    }
    None
}
