use super::{
    PAGE_PADDING, PageRasterizer, PixelRenderer, backgrounds, borders, box_shadows, decorations,
    font_raster, inline_boxes, outlines,
};
use cosmic_text::Color;

impl PageRasterizer {
    pub(super) fn render_layout_pixels_raw(
        &mut self,
        width: u32,
        height: u32,
        writing_mode: crate::style::WritingMode,
        background: [u8; 3],
        scoped: bool,
        apply_root_opacity: bool,
        transparent: bool,
    ) -> Vec<u8> {
        let mut pixels = vec![0_u8; width as usize * height as usize * 4];
        for pixel in pixels.chunks_exact_mut(4) {
            if transparent {
                pixel[..3].fill(0);
                pixel[3] = 0;
            } else {
                pixel[..3].copy_from_slice(&background);
                pixel[3] = 255;
            }
        }
        let overflow_clips = &self.overflow_clips;
        let nested_scroll = &self.nested_scroll;
        let mut background_image_cache = super::background_images::ImageCache::default();
        box_shadows::draw_outer(
            &self.runs,
            &mut pixels,
            width,
            height,
            &self.overflow_geometry,
            &overflow_clips,
        );
        box_shadows::draw_first_letter(
            &self.runs,
            &self.buffer,
            &self.cursor_spans,
            &mut pixels,
            width,
            height,
            writing_mode,
            &overflow_clips,
            &self.nested_scroll,
            false,
        );
        backgrounds::draw_block_boxes(
            &self.runs,
            &self.overflow_geometry,
            &mut pixels,
            width,
            height,
            writing_mode,
            &overflow_clips,
            nested_scroll,
            &mut background_image_cache,
        );
        backgrounds::draw(
            &self.runs,
            &self.buffer,
            &mut pixels,
            width,
            height,
            scoped || (self.root_style.background_color.is_some() && !transparent),
            writing_mode,
            &overflow_clips,
            nested_scroll,
            &mut background_image_cache,
        );
        inline_boxes::draw(
            &self.runs,
            &self.buffer,
            &mut pixels,
            width,
            height,
            writing_mode,
            &overflow_clips,
            nested_scroll,
            &mut background_image_cache,
        );
        box_shadows::draw_inset(
            &self.runs,
            &mut pixels,
            width,
            height,
            &self.overflow_geometry,
            &overflow_clips,
        );
        borders::draw(
            &self.runs,
            &mut pixels,
            width,
            height,
            writing_mode,
            &self.overflow_geometry,
            &self.overflow_clips,
        );
        if !scoped {
            borders::draw_root_border(&mut pixels, width, height, &self.root_style);
        }
        box_shadows::draw_first_line(
            &self.runs,
            &self.buffer,
            &self.cursor_spans,
            &mut pixels,
            width,
            height,
            writing_mode,
            &overflow_clips,
            &self.nested_scroll,
            false,
        );
        self.draw_first_line_backgrounds(
            &mut pixels,
            width,
            height,
            &overflow_clips,
            &mut background_image_cache,
        );
        super::first_line_boxes::draw_borders(
            &self.runs,
            &self.buffer,
            &self.cursor_spans,
            &mut pixels,
            width,
            height,
            writing_mode,
            &overflow_clips,
            &self.nested_scroll,
        );
        outlines::draw_first_line_outlines(
            &self.runs,
            &self.buffer,
            &self.cursor_spans,
            &mut pixels,
            width,
            height,
            writing_mode,
            &overflow_clips,
            &self.nested_scroll,
        );
        box_shadows::draw_first_line(
            &self.runs,
            &self.buffer,
            &self.cursor_spans,
            &mut pixels,
            width,
            height,
            writing_mode,
            &overflow_clips,
            &self.nested_scroll,
            true,
        );
        self.draw_first_letter_backgrounds(&mut pixels, width, height, &overflow_clips);
        box_shadows::draw_first_letter(
            &self.runs,
            &self.buffer,
            &self.cursor_spans,
            &mut pixels,
            width,
            height,
            writing_mode,
            &overflow_clips,
            &self.nested_scroll,
            true,
        );
        self.draw_selection(&mut pixels, width, height, &overflow_clips);
        let selection = self.selection.map(|(first, second)| {
            if first <= second {
                (first, second)
            } else {
                (second, first)
            }
        });
        let Self {
            runs,
            buffer,
            font_system,
            cache,
            cursor_spans,
            decoration_opacity_scales,
            ..
        } = self;
        font_raster::draw(
            runs,
            font_system,
            buffer,
            &mut pixels,
            width,
            height,
            selection,
            cursor_spans,
            writing_mode,
            &overflow_clips,
            nested_scroll,
        );
        let default_color = Color::rgb(0, 0, 0);
        let mut auto_ink_mask = vec![0; pixels.len() / 4];
        let mut all_ink_mask = vec![0; pixels.len() / 4];
        let vertical_block_origins =
            super::local_writing::vertical_block_origins(runs, buffer, writing_mode);
        let style_ranges = super::local_writing::text_style_ranges(runs);
        let line_starts = super::local_writing::text_line_starts(runs);
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let style_index = super::local_writing::style_index_at(
                    &style_ranges,
                    line_starts.get(run.line_i).copied().unwrap_or(0) + glyph.start,
                );
                let local_transform = style_index
                    .and_then(|index| runs.get(index))
                    .filter(|style_run| {
                        writing_mode == crate::style::WritingMode::HorizontalTb
                            && style_run.style.writing_mode
                                != crate::style::WritingMode::HorizontalTb
                    })
                    .and_then(|style_run| {
                        let block = style_run.first_line_block?;
                        let origin_y = vertical_block_origins.get(&block).copied()?;
                        Some(super::local_writing::LocalVerticalTransform::new(
                            style_run.style.writing_mode,
                            origin_y,
                            width,
                            height,
                        ))
                    });
                let physical_glyph = glyph.physical((0.0, run.line_y), 1.0);
                let mask_color = Color::rgba(255, 255, 255, 255);
                let auto_mask = &mut auto_ink_mask;
                let all_mask = &mut all_ink_mask;
                let has_descender = run.text[glyph.start..glyph.end]
                    .chars()
                    .any(is_underline_descender);
                let glyph_clip =
                    style_index
                        .and_then(|index| runs.get(index))
                        .and_then(|style_run| {
                            super::overflow_clip::clip_for_path(
                                &style_run.element_path,
                                &overflow_clips,
                            )
                        });
                cache.with_pixels(
                    font_system,
                    physical_glyph.cache_key,
                    mask_color,
                    |x, y, pixel_color| {
                        let source_x = physical_glyph.x + x + PAGE_PADDING as i32;
                        let source_y = physical_glyph.y + y + PAGE_PADDING as i32;
                        let (pixel_x, pixel_y) =
                            local_transform.map_or((source_x, source_y), |transform| {
                                let (x, y, _, _) =
                                    transform.map_rect(source_x as f32, source_y as f32, 1.0, 1.0);
                                (x.round() as i32, y.round() as i32)
                            });
                        let (nested_scroll_x, nested_scroll_y) = style_index
                            .and_then(|index| runs.get(index))
                            .map_or((0.0, 0.0), |run| {
                                super::overflow_clip::scroll_translation_for_path(
                                    &run.element_path,
                                    nested_scroll,
                                    writing_mode,
                                )
                            });
                        let pixel_x = (pixel_x as f32 - nested_scroll_x).round() as i32;
                        let pixel_y = (pixel_y as f32 - nested_scroll_y).round() as i32;
                        if pixel_x >= 0
                            && pixel_y >= 0
                            && pixel_x < width as i32
                            && pixel_y < height as i32
                            && glyph_clip.is_none_or(|clip| {
                                (pixel_x as f32) >= clip.left
                                    && (pixel_x as f32) < clip.right
                                    && (pixel_y as f32) >= clip.top
                                    && (pixel_y as f32) < clip.bottom
                            })
                        {
                            let mask_index = pixel_y as usize * width as usize + pixel_x as usize;
                            if pixel_color.a() > 0 {
                                all_mask[mask_index] = all_mask[mask_index].max(pixel_color.a());
                                if has_descender {
                                    auto_mask[mask_index] =
                                        auto_mask[mask_index].max(pixel_color.a());
                                }
                            }
                        }
                    },
                );
            }
        }
        for run in buffer.layout_runs() {
            let (nested_scroll_x, nested_scroll_y) = run
                .glyphs
                .first()
                .and_then(|glyph| {
                    super::local_writing::style_index_at(
                        &style_ranges,
                        line_starts.get(run.line_i).copied().unwrap_or(0) + glyph.start,
                    )
                })
                .and_then(|index| runs.get(index))
                .map_or((0.0, 0.0), |run| {
                    super::overflow_clip::scroll_translation_for_path(
                        &run.element_path,
                        nested_scroll,
                        writing_mode,
                    )
                });
            let mut renderer = PixelRenderer {
                pixels: &mut pixels,
                width,
                height,
                skip_ink_mask: None,
                local_transform: super::local_writing::LocalVerticalTransform::from_run(
                    runs,
                    buffer,
                    run.line_i,
                    writing_mode,
                    width,
                    height,
                    &vertical_block_origins,
                ),
                clip_rect: None,
                translate_y: nested_scroll_y,
                translate_x: nested_scroll_x,
                decoration_opacity_scale: run
                    .glyphs
                    .first()
                    .and_then(|glyph| {
                        super::local_writing::style_index_at(
                            &style_ranges,
                            line_starts.get(run.line_i).copied().unwrap_or(0) + glyph.start,
                        )
                    })
                    .and_then(|index| decoration_opacity_scales.get(index).copied())
                    .unwrap_or(1.0),
            };
            decorations::render_patterned_decorations(
                &mut renderer,
                &run,
                default_color,
                &auto_ink_mask,
                &all_ink_mask,
                &self.cursor_spans,
                &overflow_clips,
            );
        }
        self.draw_pseudo_decorations(
            &mut pixels,
            width,
            height,
            &auto_ink_mask,
            &all_ink_mask,
            &overflow_clips,
        );
        if !scoped {
            self.draw_root_scrollbars(&mut pixels, width, height, writing_mode);
        }
        if apply_root_opacity && self.root_opacity < 1.0 {
            let opacity = self.root_opacity.clamp(0.0, 1.0);
            for pixel in pixels.chunks_exact_mut(4) {
                for channel in &mut pixel[..3] {
                    *channel = (255.0 + (f32::from(*channel) - 255.0) * opacity)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
            }
        }
        pixels
    }
}

fn is_underline_descender(character: char) -> bool {
    matches!(character, 'g' | 'j' | 'p' | 'q' | 'y')
}
