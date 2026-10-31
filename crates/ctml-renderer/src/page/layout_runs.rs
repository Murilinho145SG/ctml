use std::{borrow::Cow, collections::HashMap};

use cosmic_text::{Buffer, Cursor, DecorationSpan, LayoutGlyph};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum FlexLayoutOffsetKey {
    Line(usize),
    Glyph(usize),
}

pub(crate) type FlexLayoutOffsets = HashMap<(usize, FlexLayoutOffsetKey), f32>;

pub(crate) struct PageLayoutRun<'a> {
    pub(super) line_i: usize,
    pub(super) text: &'a str,
    pub(super) glyphs: Cow<'a, [LayoutGlyph]>,
    glyph_vertical_offsets: Cow<'a, [f32]>,
    pub(super) decorations: &'a [DecorationSpan],
    pub(super) line_y: f32,
    pub(super) line_top: f32,
    pub(super) line_height: f32,
    pub(super) line_w: f32,
}

impl PageLayoutRun<'_> {
    pub(super) fn offset_for_range(&self, start: usize, end: usize) -> f32 {
        self.glyphs
            .iter()
            .enumerate()
            .find(|(_, glyph)| glyph.start < end && glyph.end > start)
            .and_then(|(index, _)| self.glyph_vertical_offsets.get(index))
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn highlight(
        &self,
        cursor_start: Cursor,
        cursor_end: Cursor,
    ) -> impl Iterator<Item = (f32, f32)> {
        let mut results = Vec::new();
        let mut range: Option<(f32, f32)> = None;
        for glyph in self.glyphs.iter() {
            let cluster = &self.text[glyph.start..glyph.end];
            let grapheme_count = cluster.graphemes(true).count().max(1);
            let grapheme_width = glyph.w / grapheme_count as f32;
            let mut x = glyph.x;
            for (offset, grapheme) in cluster.grapheme_indices(true) {
                let start = glyph.start + offset;
                let end = start + grapheme.len();
                let selected = (cursor_start.line != self.line_i || end > cursor_start.index)
                    && (cursor_end.line != self.line_i || start < cursor_end.index);
                if selected {
                    range = Some(match range {
                        Some((min, max)) => (min.min(x), max.max(x + grapheme_width)),
                        None => (x, x + grapheme_width),
                    });
                } else if let Some((min, max)) = range.take() {
                    if max > min {
                        results.push((min, max - min));
                    }
                }
                x += grapheme_width;
            }
        }
        if let Some((min, max)) = range
            && max > min
        {
            results.push((min, max - min));
        }
        results.into_iter()
    }
}

pub(crate) fn adjusted_layout_runs<'a>(
    buffer: &'a Buffer,
    offsets: &'a FlexLayoutOffsets,
) -> impl Iterator<Item = PageLayoutRun<'a>> + 'a {
    buffer.layout_runs().map(move |run| {
        let first_start = run.glyphs.first().map(|glyph| glyph.start);
        let line_offset = first_start
            .and_then(|start| offsets.get(&(run.line_i, FlexLayoutOffsetKey::Line(start))))
            .copied()
            .unwrap_or_default();
        let has_glyph_offsets = run.glyphs.iter().any(|glyph| {
            offsets.contains_key(&(run.line_i, FlexLayoutOffsetKey::Glyph(glyph.start)))
        });
        let mut glyphs = if has_glyph_offsets {
            Cow::Owned(run.glyphs.to_vec())
        } else {
            Cow::Borrowed(run.glyphs)
        };
        let glyph_vertical_offsets: Cow<'a, [f32]> = if has_glyph_offsets {
            Cow::Owned(
                run.glyphs
                    .iter()
                    .map(|glyph| {
                        offsets
                            .get(&(run.line_i, FlexLayoutOffsetKey::Glyph(glyph.start)))
                            .copied()
                            .unwrap_or_default()
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            Cow::Borrowed(&[])
        };
        if has_glyph_offsets {
            for glyph in glyphs.to_mut() {
                if let Some(offset) =
                    offsets.get(&(run.line_i, FlexLayoutOffsetKey::Glyph(glyph.start)))
                {
                    glyph.y += *offset;
                }
            }
        }
        PageLayoutRun {
            line_i: run.line_i,
            text: run.text,
            glyphs,
            glyph_vertical_offsets,
            decorations: run.decorations,
            line_y: run.line_y + line_offset,
            line_top: run.line_top + line_offset,
            line_height: run.line_height,
            line_w: run.line_w,
        }
    })
}
