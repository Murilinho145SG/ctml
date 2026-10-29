use cosmic_text::FontSystem;
use skia_safe::FontMgr;

use super::configure_font;
use super::load_typeface;

#[test]
fn compare_harfrust_and_skia_harfbuzz_advances() {
    use cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping};
    use skia_safe::shaper::run_handler::{Buffer as ShapeBuffer, RunHandler, RunInfo};
    use skia_safe::shapers::hb::shape_dont_wrap_or_reorder;
    use skia_safe::{Font, FontMgr, Shaper};

    struct RecordingRunHandler {
        advances: Vec<f32>,
        glyph_buffers: Vec<(Vec<skia_safe::GlyphId>, Vec<skia_safe::Point>)>,
        next_buffer: usize,
    }

    impl RunHandler for RecordingRunHandler {
        fn begin_line(&mut self) {}

        fn run_info(&mut self, info: &RunInfo) {
            self.advances.push(info.advance.x);
            self.glyph_buffers.push((
                vec![skia_safe::GlyphId::default(); info.glyph_count],
                vec![skia_safe::Point::default(); info.glyph_count],
            ));
        }

        fn commit_run_info(&mut self) {}

        fn run_buffer(&mut self, _: &RunInfo) -> ShapeBuffer<'_> {
            let (glyphs, positions) = &mut self.glyph_buffers[self.next_buffer];
            ShapeBuffer::new(glyphs, positions, None)
        }

        fn commit_run_buffer(&mut self, _: &RunInfo) {
            self.next_buffer += 1;
        }

        fn commit_line(&mut self) {}
    }

    let mut font_system = FontSystem::new();
    let Some(face) = font_system.db().faces().find(|face| {
        face.families
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("Liberation Serif"))
    }) else {
        return;
    };
    let font_id = face.id;
    let family = face
        .families
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("Liberation Serif"))
        .unwrap()
        .0
        .clone();
    let attrs = Attrs::new()
        .family(Family::Name(&family))
        .weight(face.weight)
        .stretch(face.stretch)
        .style(face.style);
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 20.0));
    buffer.set_text("AVATAR", &attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(&mut font_system, false);
    let harfrust_width = buffer.layout_runs().next().unwrap().line_w;

    let font_manager = FontMgr::new();
    let typeface = load_typeface(&font_system, &font_manager, font_id).unwrap();
    let mut font = Font::from_typeface(typeface, 16.0);
    configure_font(&mut font);
    let shaper = shape_dont_wrap_or_reorder(None::<FontMgr>).unwrap();
    let text = "AVATAR";
    let mut font_runs = Shaper::new_trivial_font_run_iterator(&font, text.len());
    let mut bidi_runs = Shaper::new_bidi_run_iterator(text, 0).unwrap();
    let mut script_runs = Shaper::new_hb_icu_script_run_iterator(text);
    let mut language_runs = Shaper::new_std_language_run_iterator(text).unwrap();
    let mut handler = RecordingRunHandler {
        advances: Vec::new(),
        glyph_buffers: Vec::new(),
        next_buffer: 0,
    };
    shaper.shape_with_iterators_and_features_and_options(
        text,
        &mut font_runs,
        &mut bidi_runs,
        &mut script_runs,
        &mut language_runs,
        &[],
        200.0,
        0.0,
        &mut handler,
    );
    let harfbuzz_width = handler.advances.iter().sum::<f32>();
    let cosmic_glyphs = buffer.layout_runs().next().unwrap().glyphs;
    let (harfbuzz_glyphs, harfbuzz_positions) = &handler.glyph_buffers[0];

    assert_eq!(harfrust_width, harfbuzz_width);
    assert_eq!(
        cosmic_glyphs
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>(),
        harfbuzz_glyphs
            .iter()
            .map(|glyph| u16::from(*glyph))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        cosmic_glyphs
            .iter()
            .map(|glyph| (
                glyph.x + glyph.x_offset * glyph.font_size,
                -glyph.y_offset * glyph.font_size
            ))
            .collect::<Vec<_>>(),
        harfbuzz_positions
            .iter()
            .map(|position| (position.x, position.y))
            .collect::<Vec<_>>()
    );
}

#[test]
fn compares_rtl_harfrust_and_skia_harfbuzz_positions() {
    use cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping};
    use skia_safe::shaper::run_handler::{Buffer as ShapeBuffer, RunHandler, RunInfo};
    use skia_safe::shapers::hb::shape_dont_wrap_or_reorder;
    use skia_safe::{Font, Shaper};

    struct Handler {
        advance: f32,
        glyphs: Vec<skia_safe::GlyphId>,
        positions: Vec<skia_safe::Point>,
    }

    impl RunHandler for Handler {
        fn begin_line(&mut self) {}
        fn run_info(&mut self, info: &RunInfo) {
            self.advance += info.advance.x;
            self.glyphs
                .resize(info.glyph_count, skia_safe::GlyphId::default());
            self.positions
                .resize(info.glyph_count, skia_safe::Point::default());
        }
        fn commit_run_info(&mut self) {}
        fn run_buffer(&mut self, _: &RunInfo) -> ShapeBuffer<'_> {
            ShapeBuffer::new(&mut self.glyphs, &mut self.positions, None)
        }
        fn commit_run_buffer(&mut self, _: &RunInfo) {}
        fn commit_line(&mut self) {}
    }

    let mut font_system = FontSystem::new();
    let Some(face) = font_system.db().faces().find(|face| {
        face.families
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("Noto Sans Arabic"))
            && font_system
                .db()
                .with_face_data(face.id, |_, _| ())
                .is_some()
    }) else {
        return;
    };
    let font_id = face.id;
    let family = face
        .families
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("Noto Sans Arabic"))
        .unwrap()
        .0
        .clone();
    let text = "مرحبا بالعالم";
    let attrs = Attrs::new()
        .family(Family::Name(&family))
        .weight(face.weight)
        .stretch(face.stretch)
        .style(face.style);
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 20.0));
    buffer.set_text(text, &attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(&mut font_system, false);

    let font_manager = FontMgr::new();
    let typeface = load_typeface(&font_system, &font_manager, font_id).unwrap();
    let mut font = Font::from_typeface(typeface, 16.0);
    configure_font(&mut font);
    let shaper = shape_dont_wrap_or_reorder(None::<FontMgr>).unwrap();
    let mut fonts = Shaper::new_trivial_font_run_iterator(&font, text.len());
    let mut bidi = Shaper::new_bidi_run_iterator(text, 1).unwrap();
    let mut scripts = Shaper::new_hb_icu_script_run_iterator(text);
    let mut languages = Shaper::new_std_language_run_iterator(text).unwrap();
    let mut handler = Handler {
        advance: 0.0,
        glyphs: Vec::new(),
        positions: Vec::new(),
    };
    shaper.shape_with_iterators_and_features_and_options(
        text,
        &mut fonts,
        &mut bidi,
        &mut scripts,
        &mut languages,
        &[],
        f32::MAX,
        0.0,
        &mut handler,
    );
    let cosmic = buffer.layout_runs().next().unwrap();

    assert!((cosmic.line_w - handler.advance).abs() < 0.001);
    assert_eq!(
        cosmic
            .glyphs
            .iter()
            .rev()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>(),
        handler
            .glyphs
            .iter()
            .map(|glyph| u16::from(*glyph))
            .collect::<Vec<_>>()
    );
    let cosmic_positions = cosmic
        .glyphs
        .iter()
        .rev()
        .map(|glyph| {
            (
                glyph.x + glyph.x_offset * glyph.font_size,
                -glyph.y_offset * glyph.font_size,
            )
        })
        .collect::<Vec<_>>();
    let harfbuzz_positions = handler
        .positions
        .iter()
        .map(|position| (position.x, position.y))
        .collect::<Vec<_>>();
    assert_eq!(cosmic_positions.len(), harfbuzz_positions.len());
    for ((cosmic_x, cosmic_y), (harfbuzz_x, harfbuzz_y)) in
        cosmic_positions.into_iter().zip(harfbuzz_positions)
    {
        assert!((cosmic_x - harfbuzz_x).abs() < 0.001);
        assert!((cosmic_y - harfbuzz_y).abs() < 0.001);
    }
}

#[test]
fn configures_skia_font_to_match_chromium_parity_capture_flags() {
    let mut font = skia_safe::Font::default();

    configure_font(&mut font);

    assert_eq!(font.edging(), skia_safe::font::Edging::AntiAlias);
    assert_eq!(font.hinting(), skia_safe::FontHinting::None);
    assert!(!font.is_force_auto_hinting());
    assert!(font.is_embedded_bitmaps());
    assert!(font.is_subpixel());
    assert!(!font.is_linear_metrics());
}

#[test]
fn loaded_typeface_preserves_the_selected_font_face() {
    let font_system = FontSystem::new();
    let font_manager = FontMgr::new();
    let (font_id, expected_post_script_name) = font_system
        .db()
        .faces()
        .find(|face| {
            !face.post_script_name.is_empty()
                && font_system
                    .db()
                    .with_face_data(face.id, |_, _| ())
                    .is_some()
        })
        .map(|face| (face.id, face.post_script_name.clone()))
        .expect("system font database should contain a readable face");

    let typeface = load_typeface(&font_system, &font_manager, font_id)
        .expect("Skia should load the selected font face");

    assert_eq!(
        typeface.post_script_name().as_deref(),
        Some(expected_post_script_name.as_str())
    );
}
