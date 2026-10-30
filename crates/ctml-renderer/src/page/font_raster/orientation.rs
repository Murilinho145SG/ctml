use cosmic_text::LayoutGlyph;
use skia_safe::{Font, GlyphId, Paint, Point, TextBlobBuilder};
use unicode_properties::{UnicodeGeneralCategory, general_category::GeneralCategory};

use crate::page::PAGE_PADDING;
use crate::style::{TextOrientation, TextStyle, WritingMode};

use super::text_effects::draw_text_shadows;

pub(super) fn glyph_is_upright(
    line: &cosmic_text::LayoutRun<'_>,
    glyph: &LayoutGlyph,
    style: Option<&TextStyle>,
) -> bool {
    let Some(style) = style else {
        return false;
    };
    let Some(text) = line.text.get(glyph.start..glyph.end) else {
        return false;
    };
    let first = text.chars().next();
    let base = text
        .chars()
        .find(|character| !is_combining_mark(*character))
        .or(first);
    base.is_some_and(|character| is_upright_character(style, character))
}

fn is_combining_mark(character: char) -> bool {
    matches!(
        character.general_category(),
        GeneralCategory::NonspacingMark
            | GeneralCategory::SpacingMark
            | GeneralCategory::EnclosingMark
    )
}

fn is_upright_character(style: &TextStyle, character: char) -> bool {
    if matches!(style.writing_mode, WritingMode::HorizontalTb) {
        return false;
    }
    match style.text_orientation {
        TextOrientation::Sideways => false,
        TextOrientation::Upright => true,
        TextOrientation::Mixed => matches!(
            unicode_vo::char_orientation(character),
            unicode_vo::Orientation::Upright | unicode_vo::Orientation::TransformedOrUpright
        ),
    }
}

pub(super) fn draw_upright_glyphs(
    canvas: &skia_safe::Canvas,
    glyphs: &[LayoutGlyph],
    font: &Font,
    baseline: f32,
    paint: &Paint,
    style: Option<&TextStyle>,
    opacity: f32,
) {
    let angle = match style.map(|style| style.writing_mode) {
        Some(WritingMode::VerticalRl) => -90.0,
        Some(WritingMode::VerticalLr) => 90.0,
        _ => return,
    };
    for glyph in glyphs {
        let mut builder = TextBlobBuilder::new();
        let (glyph_ids, positions) = builder.alloc_run_pos(font, 1, None);
        glyph_ids[0] = GlyphId::from(glyph.glyph_id);
        let x = glyph.x + glyph.x_offset * glyph.font_size + PAGE_PADDING;
        let y = baseline + PAGE_PADDING - glyph.y_offset * glyph.font_size;
        positions[0] = Point::new(x, y);
        let Some(blob) = builder.make() else {
            continue;
        };
        let center = Point::new(x + glyph.w * 0.5, y - font.size() * 0.5);
        canvas.save();
        canvas.rotate(angle, Some(center));
        if let Some(style) = style {
            draw_text_shadows(canvas, &blob, style, opacity);
        }
        canvas.draw_text_blob(blob, Point::default(), paint);
        canvas.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::is_upright_character;
    use crate::style::{TextOrientation, TextStyle, WritingMode};

    #[test]
    fn text_orientation_selects_unicode_vertical_glyph_orientation() {
        let mut style = TextStyle {
            writing_mode: WritingMode::VerticalRl,
            ..TextStyle::default()
        };
        assert!(!is_upright_character(&style, 'A'));
        assert!(is_upright_character(&style, '日'));

        style.text_orientation = TextOrientation::Upright;
        assert!(is_upright_character(&style, 'A'));
        style.text_orientation = TextOrientation::Sideways;
        assert!(!is_upright_character(&style, '日'));

        style.writing_mode = WritingMode::HorizontalTb;
        style.text_orientation = TextOrientation::Upright;
        assert!(!is_upright_character(&style, 'A'));
    }
}
