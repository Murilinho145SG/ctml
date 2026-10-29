use std::collections::HashMap;

use cosmic_text::Cursor;

use crate::page::CursorSpan;
use crate::style::TextStyle;

pub(super) fn first_line_style_at<'a>(
    spans: &'a [CursorSpan],
    first_line_by_block: &HashMap<usize, (usize, usize)>,
    line: usize,
    index: usize,
) -> Option<&'a TextStyle> {
    let first_span =
        spans.partition_point(|span| span.line < line || (span.line == line && span.end <= index));
    spans[first_span..]
        .iter()
        .take_while(|span| span.line == line && span.start <= index)
        .find_map(|span| {
            let block = span.first_line_block?;
            let (first_line, end) = first_line_by_block.get(&block)?;
            (*first_line == line && index < *end)
                .then_some(span.first_line_font_style.as_ref())
                .flatten()
        })
}

pub(super) fn glyph_foreground(
    selection: Option<(Cursor, Cursor)>,
    spans: &[CursorSpan],
    first_line_by_block: &HashMap<usize, (usize, usize)>,
    first_letters: &HashMap<usize, (usize, usize, usize)>,
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
        if let Some((letter_line, letter_start, letter_end)) = span
            .first_letter_block
            .and_then(|block| first_letters.get(&block))
            && *letter_line == line
            && span.start == *letter_start
            && span.end == *letter_end
            && glyph_end > *letter_start
            && glyph_start < *letter_end
            && span.first_letter_foreground.is_some()
        {
            return span.first_letter_foreground;
        }
    }
    None
}
