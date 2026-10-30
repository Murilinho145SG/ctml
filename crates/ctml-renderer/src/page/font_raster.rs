use std::collections::HashMap;

use cosmic_text::{Buffer, Cursor, FontSystem, fontdb};
use skia_safe::{
    AlphaType, Color, ColorSpace, ColorType, Font, FontHinting, FontMgr, GlyphId, ImageInfo, Paint,
    PixelGeometry, Point, Shaper, SurfaceProps, SurfacePropsFlags, TextBlobBuilder,
    font::Edging,
    font_style::{FontStyle as SkiaFontStyle, Slant, Weight as SkiaWeight, Width as SkiaWidth},
    shaper::{Feature as SkiaFeature, TextBlobBuilderRunHandler},
    surfaces,
};

use crate::page::{CursorSpan, PAGE_PADDING, local_writing};
use crate::style::{TextRun, TextStyle, WritingMode};

mod highlight;
mod orientation;
mod text_effects;

use highlight::{first_line_style_at, glyph_foreground, pseudo_opacity_at};
use orientation::{draw_upright_glyphs, glyph_is_upright};
use text_effects::{draw_text_emphasis, draw_text_shadows, scale_alpha};

pub(super) fn draw(
    runs: &[TextRun],
    font_system: &mut FontSystem,
    buffer: &Buffer,
    pixels: &mut [u8],
    width: u32,
    height: u32,
    selection: Option<(Cursor, Cursor)>,
    cursor_spans: &[CursorSpan],
    writing_mode: WritingMode,
) {
    let first_line_by_block = super::highlights::first_formatted_line_ranges(buffer, cursor_spans);
    let first_letters = super::highlights::first_formatted_letters(cursor_spans);
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
    let style_ranges = local_writing::text_style_ranges(runs);
    let line_starts = local_writing::text_line_starts(runs);
    let shaper = skia_safe::shapers::hb::shape_dont_wrap_or_reorder(None::<FontMgr>);
    let canvas = surface.canvas();
    let vertical_block_origins = local_writing::vertical_block_origins(runs, buffer, writing_mode);
    for run in buffer.layout_runs() {
        let line_start = line_starts.get(run.line_i).copied().unwrap_or(0);
        let mut start = 0;
        while start < run.glyphs.len() {
            let first = &run.glyphs[start];
            let color = glyph_foreground(
                selection,
                cursor_spans,
                &first_line_by_block,
                &first_letters,
                run.line_i,
                first.start,
                first.end,
            )
            .map(|color| cosmic_text::Color::rgba(color[0], color[1], color[2], color[3]))
            .or(first.color_opt)
            .unwrap_or(cosmic_text::Color::rgb(0, 0, 0));
            let style_index =
                local_writing::style_index_at(&style_ranges, line_start + first.start);
            let first_line_style =
                first_line_style_at(cursor_spans, &first_line_by_block, run.line_i, first.start);
            let orientation_style = first_line_style.or_else(|| {
                style_index
                    .and_then(|index| runs.get(index))
                    .map(|run| &run.style)
            });
            let orientation_mode = orientation_style
                .map(|style| style.writing_mode)
                .filter(|mode| {
                    *mode != WritingMode::HorizontalTb && writing_mode == WritingMode::HorizontalTb
                })
                .unwrap_or(writing_mode);
            let first_upright = glyph_is_upright(&run, first, orientation_style, orientation_mode);
            let opacity = pseudo_opacity_at(
                cursor_spans,
                &first_line_by_block,
                &first_letters,
                run.line_i,
                first.start,
            ) * style_index
                .and_then(|index| runs.get(index))
                .map_or(1.0, |run| {
                    if run.style.visibility_hidden {
                        0.0
                    } else {
                        run.style.opacity
                    }
                });
            let mut end = start + 1;
            while end < run.glyphs.len() {
                let glyph = &run.glyphs[end];
                let glyph_color = glyph_foreground(
                    selection,
                    cursor_spans,
                    &first_line_by_block,
                    &first_letters,
                    run.line_i,
                    glyph.start,
                    glyph.end,
                )
                .map(|color| cosmic_text::Color::rgba(color[0], color[1], color[2], color[3]))
                .or(glyph.color_opt);
                let glyph_style_index =
                    local_writing::style_index_at(&style_ranges, line_start + glyph.start);
                let glyph_first_line_style = first_line_style_at(
                    cursor_spans,
                    &first_line_by_block,
                    run.line_i,
                    glyph.start,
                );
                let glyph_orientation_style = glyph_first_line_style.or_else(|| {
                    glyph_style_index
                        .and_then(|index| runs.get(index))
                        .map(|run| &run.style)
                });
                if glyph.font_id != first.font_id
                    || glyph.font_size != first.font_size
                    || glyph.level != first.level
                    || glyph_style_index != style_index
                    || glyph_first_line_style.is_some() != first_line_style.is_some()
                    || glyph_is_upright(
                        &run,
                        glyph,
                        glyph_orientation_style,
                        glyph_orientation_style
                            .map(|style| style.writing_mode)
                            .filter(|mode| {
                                *mode != WritingMode::HorizontalTb
                                    && writing_mode == WritingMode::HorizontalTb
                            })
                            .unwrap_or(writing_mode),
                    ) != first_upright
                    || glyph_color.unwrap_or(cosmic_text::Color::rgb(0, 0, 0)) != color
                    || pseudo_opacity_at(
                        cursor_spans,
                        &first_line_by_block,
                        &first_letters,
                        run.line_i,
                        glyph.start,
                    ) != opacity
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
            let variation_font = font_system.get_font_with_variation_key(
                first.font_id,
                first.font_weight,
                first.variation_key,
            );
            let mut typeface = variation_font.as_ref().map_or(typeface.clone(), |font| {
                let variations = font
                    .variations()
                    .iter()
                    .map(|setting| (setting.tag(), setting.value()))
                    .collect::<Vec<_>>();
                crate::font_metrics::apply_variations(&typeface, &variations, first.font_weight)
            });
            if let Some(style) = first_line_style {
                let variations = crate::font_metrics::variation_settings(style);
                typeface = crate::font_metrics::apply_variations(
                    &typeface,
                    &variations,
                    fontdb::Weight(style.weight.0),
                );
            }

            let mut paint = Paint::default();
            paint.set_color(Color::from_argb(
                scale_alpha(color.a(), opacity),
                color.r(),
                color.g(),
                color.b(),
            ));

            let text_style = first_line_style.or_else(|| {
                style_index
                    .and_then(|index| runs.get(index))
                    .map(|run| &run.style)
            });
            let local_vertical_mode = text_style.map(|style| style.writing_mode).filter(|mode| {
                *mode != WritingMode::HorizontalTb && writing_mode == WritingMode::HorizontalTb
            });
            let has_local_vertical_transform = if let Some(mode) = local_vertical_mode {
                let origin_y = style_index
                    .and_then(|index| runs.get(index))
                    .and_then(|style_run| style_run.first_line_block)
                    .and_then(|block| vertical_block_origins.get(&block).copied())
                    .unwrap_or(run.line_y);
                let source = Point::new(PAGE_PADDING, origin_y);
                let target = match mode {
                    WritingMode::VerticalRl => Point::new(width as f32 - origin_y, PAGE_PADDING),
                    WritingMode::VerticalLr => Point::new(origin_y, height as f32 - PAGE_PADDING),
                    WritingMode::HorizontalTb => unreachable!(),
                };
                canvas.save();
                canvas.translate((target.x, target.y));
                canvas.rotate(
                    match mode {
                        WritingMode::VerticalRl => 90.0,
                        WritingMode::VerticalLr => -90.0,
                        WritingMode::HorizontalTb => unreachable!(),
                    },
                    None,
                );
                canvas.translate((-source.x, -source.y));
                true
            } else {
                false
            };
            let glyph_writing_mode = local_vertical_mode.unwrap_or(writing_mode);
            let font_size = text_style.map_or(first.font_size, |style| style.font_size);
            let mut font = Font::from_typeface(typeface, font_size);
            configure_font(&mut font);
            if let (Some(style), Some(face)) = (text_style, font_system.db().face(first.font_id)) {
                apply_synthetic_style(&mut font, face, style);
            }
            let baseline = run.line_y - 0.4 - text_style.map_or(0.0, |s| s.vertical_align_offset);

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
            if first_upright {
                draw_upright_glyphs(
                    canvas,
                    group,
                    &font,
                    baseline,
                    &paint,
                    text_style,
                    glyph_writing_mode,
                    opacity,
                );
                if let Some(style) = text_style {
                    draw_text_emphasis(
                        canvas,
                        &run,
                        group,
                        style,
                        glyph_writing_mode,
                        &font,
                        opacity,
                    );
                }
            } else if let Some(blob) = blob {
                if let Some(style) = text_style {
                    draw_text_shadows(canvas, &blob, style, opacity);
                }
                canvas.draw_text_blob(blob, Point::default(), &paint);
                if let Some(style) = text_style {
                    draw_text_emphasis(
                        canvas,
                        &run,
                        group,
                        style,
                        glyph_writing_mode,
                        &font,
                        opacity,
                    );
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
                        draw_text_shadows(canvas, &blob, style, opacity);
                    }
                    canvas.draw_text_blob(blob, Point::default(), &paint);
                    if let Some(style) = text_style {
                        draw_text_emphasis(
                            canvas,
                            &run,
                            group,
                            style,
                            glyph_writing_mode,
                            &font,
                            opacity,
                        );
                    }
                }
            }
            if let Some(style) = text_style {
                draw_conditional_hyphens(
                    canvas,
                    shaper.as_ref(),
                    &run,
                    group,
                    &font,
                    style,
                    baseline,
                    color,
                    opacity,
                );
            }
            if has_local_vertical_transform {
                canvas.restore();
            }
            start = end;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_conditional_hyphens(
    canvas: &skia_safe::Canvas,
    shaper: Option<&Shaper>,
    line: &cosmic_text::LayoutRun<'_>,
    glyphs: &[cosmic_text::LayoutGlyph],
    font: &Font,
    style: &TextStyle,
    baseline: f32,
    color: cosmic_text::Color,
    opacity: f32,
) {
    let Some(shaper) = shaper else {
        return;
    };
    let line_end = line.glyphs.iter().map(|glyph| glyph.end).max().unwrap_or(0);
    for glyph in glyphs {
        if glyph.end != line_end
            || line.text.get(glyph.start..glyph.end) != Some("\u{00ad}")
            || line
                .text
                .get(glyph.end..)
                .is_none_or(|remaining| remaining.is_empty() || remaining.starts_with('\n'))
        {
            continue;
        }
        let Some(blob) = shape_group(
            shaper,
            "\u{2010}",
            font,
            style,
            glyph.level.number(),
            Point::new(
                glyph.x + glyph.x_offset * glyph.font_size + PAGE_PADDING,
                baseline + PAGE_PADDING + font.metrics().1.ascent,
            ),
        ) else {
            continue;
        };
        let mut paint = Paint::default();
        paint.set_color(Color::from_argb(
            scale_alpha(color.a(), opacity),
            color.r(),
            color.g(),
            color.b(),
        ));
        draw_text_shadows(canvas, &blob, style, opacity);
        canvas.draw_text_blob(blob, Point::default(), &paint);
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
