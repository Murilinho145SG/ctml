use crate::style::TextStyle;
use skia_safe::{
    Font, Point, Shaper,
    shaper::{Feature as SkiaFeature, TextBlobBuilderRunHandler},
};

pub(super) fn shape_group(
    shaper: &Shaper,
    text: &str,
    font: &Font,
    style: &TextStyle,
    bidi_level: u8,
    offset: Point,
) -> Option<skia_safe::TextBlob> {
    let mut font_runs = Shaper::new_trivial_font_run_iterator(font, text.len());
    let mut bidi_runs = Shaper::new_bidi_run_iterator(text, bidi_level)?;
    let mut script_runs = Shaper::new_hb_icu_script_run_iterator(text);
    let mut language_runs = match style.language.as_deref() {
        Some(language) => Shaper::new_trivial_language_run_iterator(language, text.len()),
        None => Shaper::new_std_language_run_iterator(text)?,
    };
    let features = style
        .font_features()
        .features
        .into_iter()
        .map(|feature| SkiaFeature {
            tag: u32::from_be_bytes(*feature.tag.as_bytes()),
            value: feature.value,
            start: 0,
            end: text.len(),
        })
        .collect::<Vec<_>>();
    let tracking = if style.letter_spacing_px {
        style.letter_spacing / style.font_size
    } else {
        style.letter_spacing
    };
    let mut handler = TextBlobBuilderRunHandler::new(text, offset);
    shaper.shape_with_iterators_and_features_and_options(
        text,
        &mut font_runs,
        &mut bidi_runs,
        &mut script_runs,
        &mut language_runs,
        &features,
        f32::MAX,
        tracking,
        &mut handler,
    );
    handler.make_blob()
}

// Painting and picking must share the same shaped glyph positions, including
// kerning, word spacing, justification, and ligature advances.
pub(super) fn positioned_group(
    font: &Font,
    glyphs: &[cosmic_text::LayoutGlyph],
    baseline: f32,
) -> Option<skia_safe::TextBlob> {
    let mut builder = skia_safe::TextBlobBuilder::new();
    let (glyph_ids, positions) = builder.alloc_run_pos(font, glyphs.len(), None);
    for (index, glyph) in glyphs.iter().enumerate() {
        glyph_ids[index] = skia_safe::GlyphId::from(glyph.glyph_id);
        positions[index] = Point::new(
            glyph.x + glyph.x_offset * glyph.font_size + crate::page::page_padding(),
            baseline + crate::page::page_padding() - glyph.y_offset * glyph.font_size,
        );
    }
    builder.make()
}
