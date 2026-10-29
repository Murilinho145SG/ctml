use std::collections::HashMap;

use cosmic_text::{Buffer, FontSystem};
use skia_safe::{
    AlphaType, Color, ColorSpace, ColorType, Font, FontHinting, FontMgr, GlyphId, ImageInfo, Paint,
    Point, TextBlobBuilder, font::Edging, surfaces,
};

use crate::page::PAGE_PADDING;

pub(super) fn draw(
    font_system: &FontSystem,
    buffer: &Buffer,
    pixels: &mut [u8],
    width: u32,
    height: u32,
) {
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        ColorSpace::new_srgb(),
    );
    let Some(mut surface) = surfaces::wrap_pixels(&info, pixels, Some(width as usize * 4), None)
    else {
        return;
    };

    let font_manager = FontMgr::new();
    let mut typefaces = HashMap::new();
    let canvas = surface.canvas();
    for run in buffer.layout_runs() {
        let mut start = 0;
        while start < run.glyphs.len() {
            let first = &run.glyphs[start];
            let color = first.color_opt.unwrap_or(cosmic_text::Color::rgb(0, 0, 0));
            let mut end = start + 1;
            while end < run.glyphs.len() {
                let glyph = &run.glyphs[end];
                if glyph.font_id != first.font_id
                    || glyph.font_size != first.font_size
                    || glyph.color_opt.unwrap_or(cosmic_text::Color::rgb(0, 0, 0)) != color
                {
                    break;
                }
                end += 1;
            }

            let typeface = typefaces.entry(first.font_id).or_insert_with(|| {
                font_system
                    .db()
                    .with_face_data(first.font_id, |data, index| {
                        font_manager.new_from_bytes(data, Some(index))
                    })
                    .flatten()
            });
            let typeface = typeface.clone();
            let Some(typeface) = typeface else {
                start = end;
                continue;
            };

            let mut paint = Paint::default();
            paint.set_color(Color::from_argb(color.a(), color.r(), color.g(), color.b()));

            let mut font = Font::from_typeface(typeface, first.font_size);
            font.set_edging(Edging::AntiAlias);
            font.set_hinting(FontHinting::Slight);
            font.set_force_auto_hinting(false);
            font.set_embedded_bitmaps(true);
            font.set_subpixel(true);
            font.set_linear_metrics(false);

            let group = &run.glyphs[start..end];
            let mut builder = TextBlobBuilder::new();
            let (glyph_ids, positions) = builder.alloc_run_pos(&font, group.len(), None);
            for (index, glyph) in group.iter().enumerate() {
                glyph_ids[index] = GlyphId::from(glyph.glyph_id);
                positions[index] = Point::new(
                    glyph.x + glyph.x_offset * glyph.font_size + PAGE_PADDING,
                    run.line_y + PAGE_PADDING - 1.0 - glyph.y_offset * glyph.font_size,
                );
            }
            if let Some(blob) = builder.make() {
                canvas.draw_text_blob(blob, Point::new(0.0, 0.0), &paint);
            }
            start = end;
        }
    }
}
