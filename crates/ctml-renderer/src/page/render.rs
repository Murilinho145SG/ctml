use cosmic_text::Color;

use super::{PAGE_PADDING, PageRasterizer, PixelRenderer, backgrounds, decorations, font_raster};

impl PageRasterizer {
    pub(super) fn render_layout_pixels(
        &mut self,
        width: u32,
        height: u32,
        writing_mode: crate::style::WritingMode,
    ) -> Vec<u8> {
        let mut pixels = vec![0_u8; width as usize * height as usize * 4];
        for pixel in pixels.chunks_exact_mut(4) {
            pixel[..3].copy_from_slice(&self.root_style.background);
            pixel[3] = 255;
        }
        backgrounds::draw(
            &self.runs,
            &self.buffer,
            &mut pixels,
            width,
            height,
            self.root_style.background_color.is_some(),
            writing_mode,
        );
        self.draw_first_line_backgrounds(&mut pixels, width, height);
        self.draw_first_letter_backgrounds(&mut pixels, width, height);
        self.draw_selection(&mut pixels, width, height);
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
        );
        let default_color = Color::rgb(0, 0, 0);
        let mut auto_ink_mask = vec![0; pixels.len() / 4];
        let mut all_ink_mask = vec![0; pixels.len() / 4];
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let physical_glyph = glyph.physical((0.0, run.line_y), 1.0);
                let mask_color = Color::rgba(255, 255, 255, 255);
                let auto_mask = &mut auto_ink_mask;
                let all_mask = &mut all_ink_mask;
                let has_descender = run.text[glyph.start..glyph.end]
                    .chars()
                    .any(is_underline_descender);
                cache.with_pixels(
                    font_system,
                    physical_glyph.cache_key,
                    mask_color,
                    |x, y, pixel_color| {
                        let pixel_x = physical_glyph.x + x + PAGE_PADDING as i32;
                        let pixel_y = physical_glyph.y + y + PAGE_PADDING as i32;
                        if pixel_x >= 0
                            && pixel_y >= 0
                            && pixel_x < width as i32
                            && pixel_y < height as i32
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
            let mut renderer = PixelRenderer {
                pixels: &mut pixels,
                width,
                height,
                skip_ink_mask: None,
            };
            decorations::render_patterned_decorations(
                &mut renderer,
                &run,
                default_color,
                &auto_ink_mask,
                &all_ink_mask,
                &self.cursor_spans,
            );
        }
        self.draw_pseudo_decorations(&mut pixels, width, height, &auto_ink_mask, &all_ink_mask);
        pixels
    }
}

fn is_underline_descender(character: char) -> bool {
    matches!(character, 'g' | 'j' | 'p' | 'q' | 'y')
}
