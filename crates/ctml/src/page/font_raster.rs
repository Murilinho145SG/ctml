use std::collections::HashMap;

use cosmic_text::{Buffer, FontSystem, fontdb};
use skia_safe::{
    AlphaType, Color, ColorSpace, ColorType, Font, FontHinting, FontMgr, GlyphId, ImageInfo, Paint,
    PixelGeometry, Point, SurfaceProps, SurfacePropsFlags, TextBlobBuilder,
    font::Edging,
    font_style::{FontStyle as SkiaFontStyle, Slant, Weight as SkiaWeight, Width as SkiaWidth},
    surfaces,
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
    let surface_props = SurfaceProps::new_with_text_properties(
        SurfacePropsFlags::DEFAULT,
        PixelGeometry::Unknown,
        0.2,
        1.2,
    );
    let Some(mut surface) = surfaces::wrap_pixels(
        &info,
        pixels,
        Some(width as usize * 4),
        Some(&surface_props),
    ) else {
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

            let typeface = typefaces
                .entry(first.font_id)
                .or_insert_with(|| load_typeface(font_system, &font_manager, first.font_id));
            let typeface = typeface.clone();
            let Some(typeface) = typeface else {
                start = end;
                continue;
            };

            let mut paint = Paint::default();
            paint.set_color(Color::from_argb(color.a(), color.r(), color.g(), color.b()));

            let mut font = Font::from_typeface(typeface, first.font_size);
            let render_params = font_system.db().face(first.font_id).and_then(|face| {
                let family = face.families.first()?.0.as_str();
                #[cfg(target_os = "linux")]
                {
                    crate::fontconfig::render_params(
                        family,
                        first.font_size,
                        first.font_weight.0,
                        face.style != fontdb::Style::Normal,
                    )
                }
                #[cfg(not(target_os = "linux"))]
                {
                    let _ = (family, face);
                    None
                }
            });
            let antialiasing = render_params.is_none_or(|params| params.antialiasing);
            font.set_edging(if antialiasing {
                Edging::AntiAlias
            } else {
                Edging::Alias
            });
            let hinting =
                render_params.map_or(FontHinting::Slight, |params| match params.hinting {
                    0 => FontHinting::None,
                    1 => FontHinting::Slight,
                    2 => FontHinting::Normal,
                    _ => FontHinting::Full,
                });
            font.set_hinting(hinting);
            font.set_force_auto_hinting(render_params.is_some_and(|params| params.autohinter));
            font.set_embedded_bitmaps(render_params.is_none_or(|params| params.use_bitmaps));
            font.set_subpixel(true);
            font.set_linear_metrics(false);
            let baseline = run.line_y - 0.4;

            let group = &run.glyphs[start..end];
            let mut builder = TextBlobBuilder::new();
            let (glyph_ids, positions) = builder.alloc_run_pos(&font, group.len(), None);
            for (index, glyph) in group.iter().enumerate() {
                glyph_ids[index] = GlyphId::from(glyph.glyph_id);
                positions[index] = Point::new(
                    glyph.x + glyph.x_offset * glyph.font_size + PAGE_PADDING,
                    baseline + PAGE_PADDING - glyph.y_offset * glyph.font_size,
                );
            }
            if let Some(blob) = builder.make() {
                canvas.draw_text_blob(blob, Point::new(0.0, 0.0), &paint);
            }
            start = end;
        }
    }
}

fn load_typeface(
    font_system: &FontSystem,
    font_manager: &FontMgr,
    font_id: fontdb::ID,
) -> Option<skia_safe::Typeface> {
    let from_database = font_system
        .db()
        .with_face_data(font_id, |data, index| {
            font_manager.new_from_bytes(data, Some(index))
        })
        .flatten();
    if from_database.is_some() {
        return from_database;
    }

    let matched = font_system.db().face(font_id).and_then(|face| {
        let family = face.families.first()?.0.as_str();
        let width = match face.stretch {
            fontdb::Stretch::UltraCondensed => SkiaWidth::ULTRA_CONDENSED,
            fontdb::Stretch::ExtraCondensed => SkiaWidth::EXTRA_CONDENSED,
            fontdb::Stretch::Condensed => SkiaWidth::CONDENSED,
            fontdb::Stretch::SemiCondensed => SkiaWidth::SEMI_CONDENSED,
            fontdb::Stretch::Normal => SkiaWidth::NORMAL,
            fontdb::Stretch::SemiExpanded => SkiaWidth::SEMI_EXPANDED,
            fontdb::Stretch::Expanded => SkiaWidth::EXPANDED,
            fontdb::Stretch::ExtraExpanded => SkiaWidth::EXTRA_EXPANDED,
            fontdb::Stretch::UltraExpanded => SkiaWidth::ULTRA_EXPANDED,
        };
        let slant = match face.style {
            fontdb::Style::Normal => Slant::Upright,
            fontdb::Style::Italic => Slant::Italic,
            fontdb::Style::Oblique => Slant::Oblique,
        };
        let style = SkiaFontStyle::new(SkiaWeight::from(i32::from(face.weight.0)), width, slant);
        font_manager.match_family_style(family, style)
    });
    matched
}

#[cfg(test)]
mod tests {
    use cosmic_text::FontSystem;
    use skia_safe::FontMgr;

    use super::load_typeface;

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
}
