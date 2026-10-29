use std::collections::HashMap;

use cosmic_text::{Buffer, FontSystem, fontdb};
use skia_safe::{
    AlphaType, Canvas, Color, Color4f, ColorSpace, ColorType, Font, FontHinting, FontMgr, GlyphId,
    ImageInfo, Paint, PaintStyle, Path, PathBuilder, PathFillType, PixelGeometry, Point, Rect,
    Shaper, SurfaceProps, SurfacePropsFlags, TextBlob, TextBlobBuilder,
    font::Edging,
    font_style::{FontStyle as SkiaFontStyle, Slant, Weight as SkiaWeight, Width as SkiaWidth},
    shaper::{Feature as SkiaFeature, TextBlobBuilderRunHandler},
    surfaces,
};

use crate::page::PAGE_PADDING;
use crate::style::{TextEmphasisPosition, TextEmphasisStyle, TextRun, TextStyle};

pub(super) fn draw(
    runs: &[TextRun],
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
    let style_ranges = text_style_ranges(runs);
    let line_starts = text_line_starts(runs);
    let shaper = skia_safe::shapers::hb::shape_dont_wrap_or_reorder(None::<FontMgr>);
    let canvas = surface.canvas();
    for run in buffer.layout_runs() {
        let line_start = line_starts.get(run.line_i).copied().unwrap_or(0);
        let mut start = 0;
        while start < run.glyphs.len() {
            let first = &run.glyphs[start];
            let color = first.color_opt.unwrap_or(cosmic_text::Color::rgb(0, 0, 0));
            let style_index = style_index_at(&style_ranges, line_start + first.start);
            let mut end = start + 1;
            while end < run.glyphs.len() {
                let glyph = &run.glyphs[end];
                if glyph.font_id != first.font_id
                    || glyph.font_size != first.font_size
                    || glyph.level != first.level
                    || style_index_at(&style_ranges, line_start + glyph.start) != style_index
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
            configure_font(&mut font);
            let text_style = style_index
                .and_then(|index| runs.get(index))
                .map(|run| &run.style);
            if let (Some(style), Some(face)) = (text_style, font_system.db().face(first.font_id)) {
                apply_synthetic_style(&mut font, face, style);
            }
            let baseline = run.line_y - 0.4;

            let group = &run.glyphs[start..end];
            let text_start = group.iter().map(|glyph| glyph.start).min().unwrap_or(0);
            let text_end = group
                .iter()
                .map(|glyph| glyph.end)
                .max()
                .unwrap_or(text_start);
            let group_text = run.text.get(text_start..text_end);
            let anchor = if first.level.number() % 2 == 0 {
                group.first().unwrap_or(first)
            } else {
                group.last().unwrap_or(first)
            };
            let blob = shaper.as_ref().and_then(|shaper| {
                let style = text_style?;
                let text = group_text?;
                shape_group(
                    shaper,
                    text,
                    &font,
                    style,
                    first.level.number(),
                    Point::new(
                        anchor.x + anchor.x_offset * anchor.font_size + PAGE_PADDING,
                        baseline + PAGE_PADDING + font.metrics().1.ascent,
                    ),
                )
            });
            if let Some(blob) = blob {
                if let Some(style) = text_style {
                    draw_text_shadows(canvas, &blob, style);
                }
                canvas.draw_text_blob(blob, Point::default(), &paint);
                if let Some(style) = text_style {
                    draw_text_emphasis(canvas, &run, group, style);
                }
            } else {
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
                    if let Some(style) = text_style {
                        draw_text_shadows(canvas, &blob, style);
                    }
                    canvas.draw_text_blob(blob, Point::default(), &paint);
                    if let Some(style) = text_style {
                        draw_text_emphasis(canvas, &run, group, style);
                    }
                }
            }
            start = end;
        }
    }
}

fn draw_text_shadows(canvas: &Canvas, blob: &TextBlob, style: &TextStyle) {
    for shadow in style.text_shadows.iter().rev() {
        let color = shadow.color;
        let color = Color4f::new(
            f32::from(color[0]) / 255.0,
            f32::from(color[1]) / 255.0,
            f32::from(color[2]) / 255.0,
            f32::from(color[3]) / 255.0,
        );
        let Some(filter) = skia_safe::image_filters::drop_shadow_only(
            (shadow.offset_x, shadow.offset_y),
            (shadow.blur, shadow.blur),
            color,
            None,
            None,
            None,
        ) else {
            continue;
        };
        let mut paint = Paint::default();
        paint.set_color(Color::WHITE);
        paint.set_image_filter(filter);
        canvas.draw_text_blob(blob, Point::default(), &paint);
    }
}

fn draw_text_emphasis(
    canvas: &Canvas,
    line: &cosmic_text::LayoutRun<'_>,
    glyphs: &[cosmic_text::LayoutGlyph],
    style: &TextStyle,
) {
    if style.text_emphasis_style == TextEmphasisStyle::None {
        return;
    }
    let color = style.text_emphasis_color.unwrap_or(style.color);
    let mut paint = Paint::default();
    paint.set_color(Color::from_argb(color[3], color[0], color[1], color[2]));
    let mut seen = std::collections::HashSet::new();
    for glyph in glyphs {
        if !seen.insert((glyph.start, glyph.end))
            || line
                .text
                .get(glyph.start..glyph.end)
                .is_none_or(|text| text.chars().all(char::is_whitespace))
        {
            continue;
        }
        let radius = (glyph.font_size * 0.14).clamp(1.0, 2.5);
        let center = Point::new(
            glyph.x + glyph.w * 0.5 + PAGE_PADDING,
            match style.text_emphasis_position {
                TextEmphasisPosition::Over => line.line_top + radius + PAGE_PADDING,
                TextEmphasisPosition::Under => {
                    line.line_top + line.line_height - radius - 1.0 + PAGE_PADDING
                }
            },
        );
        draw_emphasis_mark(
            canvas,
            center,
            radius,
            style.text_emphasis_style,
            &mut paint,
        );
    }
}

fn draw_emphasis_mark(
    canvas: &Canvas,
    center: Point,
    radius: f32,
    style: TextEmphasisStyle,
    paint: &mut Paint,
) {
    let open = matches!(
        style,
        TextEmphasisStyle::OpenDot
            | TextEmphasisStyle::OpenCircle
            | TextEmphasisStyle::OpenDoubleCircle
            | TextEmphasisStyle::OpenTriangle
            | TextEmphasisStyle::OpenSesame
    );
    paint.set_style(if open {
        PaintStyle::Stroke
    } else {
        PaintStyle::Fill
    });
    paint.set_stroke_width((radius * 0.4).max(0.7));
    match style {
        TextEmphasisStyle::FilledDot | TextEmphasisStyle::OpenDot => {
            canvas.draw_circle(center, radius * 0.58, paint);
        }
        TextEmphasisStyle::FilledCircle | TextEmphasisStyle::OpenCircle => {
            canvas.draw_circle(center, radius, paint);
        }
        TextEmphasisStyle::FilledDoubleCircle => {
            let mut rings = PathBuilder::new();
            rings.set_fill_type(PathFillType::EvenOdd);
            rings.add_circle(center, radius, None);
            rings.add_circle(center, radius * 0.55, None);
            let rings = Path::from(rings);
            canvas.draw_path(&rings, paint);
            canvas.draw_circle(center, radius * 0.2, paint);
        }
        TextEmphasisStyle::OpenDoubleCircle => {
            paint.set_style(PaintStyle::Stroke);
            canvas.draw_circle(center, radius, paint);
            canvas.draw_circle(center, radius * 0.52, paint);
        }
        TextEmphasisStyle::FilledTriangle | TextEmphasisStyle::OpenTriangle => {
            let path = Path::polygon(
                &[
                    Point::new(center.x, center.y - radius),
                    Point::new(center.x + radius, center.y + radius * 0.8),
                    Point::new(center.x - radius, center.y + radius * 0.8),
                ],
                true,
                None,
                None,
            );
            canvas.draw_path(&path, paint);
        }
        TextEmphasisStyle::FilledSesame | TextEmphasisStyle::OpenSesame => {
            let bounds = Rect::from_xywh(
                center.x - radius * 0.65,
                center.y - radius,
                radius * 1.3,
                radius * 2.0,
            );
            canvas.draw_oval(bounds, paint);
        }
        TextEmphasisStyle::None => {}
    }
}

fn shape_group(
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
    let mut language_runs = Shaper::new_std_language_run_iterator(text)?;
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

fn text_style_ranges(runs: &[TextRun]) -> Vec<std::ops::Range<usize>> {
    let mut offset = 0;
    runs.iter()
        .map(|run| {
            let range = offset..offset + run.text.len();
            offset = range.end;
            range
        })
        .collect()
}

fn style_index_at(ranges: &[std::ops::Range<usize>], offset: usize) -> Option<usize> {
    ranges
        .iter()
        .position(|range| range.start <= offset && offset < range.end)
}

fn text_line_starts(runs: &[TextRun]) -> Vec<usize> {
    let mut starts = vec![0];
    let mut offset = 0;
    for run in runs {
        for (index, character) in run.text.char_indices() {
            if character == '\n' {
                starts.push(offset + index + character.len_utf8());
            }
        }
        offset += run.text.len();
    }
    starts
}

fn configure_font(font: &mut Font) {
    font.set_edging(Edging::AntiAlias);
    font.set_hinting(FontHinting::None);
    font.set_force_auto_hinting(true);
    font.set_embedded_bitmaps(true);
    font.set_subpixel(true);
    font.set_linear_metrics(true);
}

fn apply_synthetic_style(font: &mut Font, face: &fontdb::FaceInfo, style: &TextStyle) {
    font.set_embolden(
        style.weight >= fontdb::Weight::SEMIBOLD && face.weight < fontdb::Weight::SEMIBOLD,
    );
    font.set_skew_x(
        if style.font_style != fontdb::Style::Normal && face.style == fontdb::Style::Normal {
            -0.25
        } else {
            0.0
        },
    );
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
#[path = "font_raster/tests.rs"]
mod tests;
