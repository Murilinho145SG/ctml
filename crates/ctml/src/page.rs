use cosmic_text::{
    Affinity, Buffer, Color, Cursor, FontSystem, Metrics, Renderer, Shaping, SwashCache,
};
use unicode_segmentation::UnicodeSegmentation;
use winit::window::CursorIcon;

use crate::{
    StyleRule, UiNode,
    style::{TextRun, TextStyle, compile_runs},
};

const PAGE_PADDING: f32 = 8.0;
const SELECTION_COLOR: [u8; 3] = [173, 216, 230];

pub(crate) struct PageRasterizer {
    runs: Vec<TextRun>,
    cursor_spans: Vec<CursorSpan>,
    font_system: FontSystem,
    cache: SwashCache,
    buffer: Buffer,
    root_style: TextStyle,
    selection: Option<(Cursor, Cursor)>,
    selection_anchor: Option<Cursor>,
    layout_size: Option<(u32, u32)>,
}

impl PageRasterizer {
    pub(crate) fn new(nodes: &[UiNode], stylesheet: &[StyleRule]) -> Self {
        let (runs, root_style) = compile_runs(nodes, stylesheet);
        let cursor_spans = cursor_spans(&runs);
        let mut font_system = FontSystem::new();
        #[cfg(target_os = "linux")]
        // Match Chromium's Arial-compatible face for the generic sans-serif family.
        font_system
            .db_mut()
            .set_sans_serif_family("Liberation Sans");
        let buffer = Buffer::new(
            &mut font_system,
            Metrics::new(root_style.font_size, root_style.line_height),
        );

        Self {
            runs,
            cursor_spans,
            font_system,
            cache: SwashCache::new(),
            buffer,
            root_style,
            selection: None,
            selection_anchor: None,
            layout_size: None,
        }
    }

    pub(crate) fn resize(
        &mut self,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
    ) {
        self.prepare_layout(width, height);
        self.upload_pixels(queue, texture, width, height);
    }

    #[cfg(test)]
    pub(crate) fn snapshot_pixels(&mut self, width: u32, height: u32) -> Vec<u8> {
        self.prepare_layout(width, height);
        self.render_pixels(width, height)
    }

    fn prepare_layout(&mut self, width: u32, height: u32) {
        self.layout_size = Some((width, height));
        self.buffer.set_size(
            Some((width as f32 - PAGE_PADDING * 2.0).max(1.0)),
            Some((height as f32 - PAGE_PADDING * 2.0).max(1.0)),
        );
        let attributes = self
            .runs
            .iter()
            .map(|run| run.style.to_attrs())
            .collect::<Vec<_>>();
        self.buffer.set_rich_text(
            self.runs
                .iter()
                .zip(&attributes)
                .map(|(run, attrs)| (run.text.as_str(), attrs.clone())),
            &self.root_style.to_attrs(),
            Shaping::Advanced,
            Some(self.root_style.text_align),
        );
        for (line, align) in self
            .buffer
            .lines
            .iter_mut()
            .zip(line_alignments(&self.runs))
        {
            line.set_align(Some(align));
        }
        self.buffer.shape_until_scroll(&mut self.font_system, false);
    }

    pub(crate) fn redraw(&mut self, queue: &wgpu::Queue, texture: &wgpu::Texture) {
        let Some((width, height)) = self.layout_size else {
            return;
        };
        self.upload_pixels(queue, texture, width, height);
    }

    pub(crate) fn cursor_at(&self, x: f64, y: f64) -> (CursorIcon, bool) {
        let fallback = if self.root_style.cursor == CursorIcon::Text {
            CursorIcon::Default
        } else {
            self.root_style.cursor
        };
        let fallback_visible = !self.root_style.cursor_hidden;
        if x < PAGE_PADDING as f64 || y < PAGE_PADDING as f64 {
            return (fallback, fallback_visible);
        }
        let Some(cursor) = self.hit(x, y) else {
            return (fallback, fallback_visible);
        };
        self.cursor_spans
            .iter()
            .find(|span| {
                span.line == cursor.line && cursor.index >= span.start && cursor.index < span.end
            })
            .or_else(|| {
                self.cursor_spans
                    .iter()
                    .rev()
                    .find(|span| span.line == cursor.line && cursor.index == span.end)
            })
            .map_or((fallback, fallback_visible), |span| {
                (span.icon, !span.hidden)
            })
    }

    pub(crate) fn begin_selection(&mut self, x: f64, y: f64) -> bool {
        let Some(cursor) = self.hit(x, y) else {
            let changed = self.selection.take().is_some();
            self.selection_anchor = None;
            return changed;
        };
        self.selection_anchor = Some(cursor);
        self.selection = Some((cursor, cursor));
        true
    }

    pub(crate) fn extend_selection(&mut self, x: f64, y: f64) -> bool {
        let Some(anchor) = self.selection_anchor else {
            return false;
        };
        let Some(cursor) = self.hit(x, y) else {
            return false;
        };
        let selection = (anchor, cursor);
        if self.selection == Some(selection) {
            return false;
        }
        self.selection = Some(selection);
        true
    }

    pub(crate) fn select_word_at(&mut self, x: f64, y: f64) -> bool {
        let Some(cursor) = self.hit(x, y) else {
            let changed = self.selection.take().is_some();
            self.selection_anchor = None;
            return changed;
        };
        let Some(text) = self
            .buffer
            .layout_runs()
            .find(|run| run.line_i == cursor.line)
            .map(|run| run.text.to_owned())
        else {
            return false;
        };
        let target = match cursor.affinity {
            Affinity::After => cursor.index,
            Affinity::Before => text[..cursor.index]
                .char_indices()
                .last()
                .map_or(cursor.index, |(index, _)| index),
        };
        let Some((start, segment)) = text
            .split_word_bound_indices()
            .find(|(start, segment)| target >= *start && target < *start + segment.len())
        else {
            return false;
        };
        let start = Cursor::new_with_affinity(cursor.line, start, Affinity::After);
        let end =
            Cursor::new_with_affinity(cursor.line, start.index + segment.len(), Affinity::Before);
        self.selection = Some((start, end));
        self.selection_anchor = None;
        true
    }

    pub(crate) fn end_selection(&mut self) {
        self.selection_anchor = None;
    }

    fn hit(&self, x: f64, y: f64) -> Option<Cursor> {
        let x = x as f32 - PAGE_PADDING;
        let y = y as f32 - PAGE_PADDING;
        let inside_text_line = self.buffer.layout_runs().any(|run| {
            run.line_w > 0.0
                && x >= 0.0
                && x <= run.line_w
                && y >= run.line_top
                && y < run.line_top + run.line_height
        });
        inside_text_line.then(|| self.buffer.hit(x, y)).flatten()
    }

    fn upload_pixels(
        &mut self,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
    ) {
        let pixels = self.render_pixels(width, height);
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }

    fn render_pixels(&mut self, width: u32, height: u32) -> Vec<u8> {
        let mut pixels = vec![0_u8; width as usize * height as usize * 4];
        for pixel in pixels.chunks_exact_mut(4) {
            pixel[..3].copy_from_slice(&self.root_style.background);
            pixel[3] = 255;
        }
        self.draw_selection(&mut pixels, width, height);
        let Self {
            buffer,
            font_system,
            cache,
            ..
        } = self;
        let mut renderer = PixelRenderer {
            pixels: &mut pixels,
            width,
            height,
        };
        let default_color = Color::rgb(0, 0, 0);
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let physical_glyph = glyph.physical((0.0, run.line_y), 1.0);
                let color = glyph.color_opt.unwrap_or(default_color);
                let base_alpha = color.a() as u16;
                let pixels = &mut *renderer.pixels;
                let width = renderer.width;
                let height = renderer.height;
                cache.with_pixels(
                    font_system,
                    physical_glyph.cache_key,
                    color,
                    |x, y, pixel_color| {
                        let alpha = (pixel_color.a() as u16 * base_alpha / 255) as u8;
                        blend_pixel(
                            pixels,
                            width,
                            height,
                            physical_glyph.x + x + PAGE_PADDING as i32,
                            physical_glyph.y + y + PAGE_PADDING as i32,
                            pixel_color,
                            alpha,
                        );
                    },
                );
            }
            cosmic_text::render_decoration(&mut renderer, &run, default_color);
        }
        pixels
    }

    fn draw_selection(&self, pixels: &mut [u8], width: u32, height: u32) {
        let Some((first, second)) = self.selection else {
            return;
        };
        let (start, end) = if first <= second {
            (first, second)
        } else {
            (second, first)
        };
        for run in self.buffer.layout_runs() {
            if run.line_i < start.line || run.line_i > end.line {
                continue;
            }
            for (left, span_width) in run.highlight(start, end) {
                fill_rect(
                    pixels,
                    width,
                    height,
                    left + PAGE_PADDING,
                    run.line_top + PAGE_PADDING,
                    span_width,
                    run.line_height,
                    SELECTION_COLOR,
                );
            }
        }
    }
}

struct PixelRenderer<'a> {
    pixels: &'a mut [u8],
    width: u32,
    height: u32,
}

impl Renderer for PixelRenderer<'_> {
    fn rectangle(&mut self, x: i32, y: i32, width: u32, height: u32, color: Color) {
        for offset_y in 0..height as i32 {
            for offset_x in 0..width as i32 {
                self.blend(
                    x + offset_x + PAGE_PADDING as i32,
                    y + offset_y + PAGE_PADDING as i32,
                    color,
                    color.a(),
                );
            }
        }
    }

    fn glyph(&mut self, _glyph: cosmic_text::PhysicalGlyph, _color: Color) {
        unreachable!("glyphs are rasterized through the alpha-aware page renderer")
    }
}

impl PixelRenderer<'_> {
    fn blend(&mut self, x: i32, y: i32, color: Color, alpha: u8) {
        blend_pixel(self.pixels, self.width, self.height, x, y, color, alpha);
    }
}

fn blend_pixel(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    color: Color,
    alpha: u8,
) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 || alpha == 0 {
        return;
    }
    let offset = (y as usize * width as usize + x as usize) * 4;
    let rgba = color.as_rgba_tuple();
    let alpha = alpha as u16;
    for channel in 0..3 {
        pixels[offset + channel] = ((pixels[offset + channel] as u16 * (255 - alpha)
            + rgba_channel(rgba, channel) as u16 * alpha)
            / 255) as u8;
    }
    pixels[offset + 3] = 255;
}

fn line_alignments(runs: &[TextRun]) -> Vec<cosmic_text::Align> {
    let mut alignments = vec![cosmic_text::Align::Left];
    let mut line_has_text = false;
    for run in runs {
        for character in run.text.chars() {
            if character == '\n' {
                alignments.push(run.style.text_align);
                line_has_text = false;
            } else if !line_has_text {
                *alignments.last_mut().expect("at least one text line") = run.style.text_align;
                line_has_text = true;
            }
        }
    }
    alignments
}

struct CursorSpan {
    line: usize,
    start: usize,
    end: usize,
    icon: CursorIcon,
    hidden: bool,
}

fn cursor_spans(runs: &[TextRun]) -> Vec<CursorSpan> {
    let mut spans = Vec::new();
    let mut line = 0;
    let mut index = 0;
    for run in runs {
        for character in run.text.chars() {
            if character == '\n' {
                line += 1;
                index = 0;
                continue;
            }
            let end = index + character.len_utf8();
            spans.push(CursorSpan {
                line,
                start: index,
                end,
                icon: run.style.cursor,
                hidden: run.style.cursor_hidden,
            });
            index = end;
        }
    }
    spans
}

fn fill_rect(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: f32,
    y: f32,
    rect_width: f32,
    rect_height: f32,
    color: [u8; 3],
) {
    let left = (x.floor() as i32).clamp(0, width as i32) as u32;
    let top = (y.floor() as i32).clamp(0, height as i32) as u32;
    let right = ((x + rect_width).ceil() as i32).clamp(0, width as i32) as u32;
    let bottom = ((y + rect_height).ceil() as i32).clamp(0, height as i32) as u32;
    for row in top..bottom {
        for column in left..right {
            let offset = (row as usize * width as usize + column as usize) * 4;
            pixels[offset..offset + 3].copy_from_slice(&color);
        }
    }
}

fn rgba_channel(rgba: (u8, u8, u8, u8), channel: usize) -> u8 {
    match channel {
        0 => rgba.0,
        1 => rgba.1,
        _ => rgba.2,
    }
}

#[cfg(test)]
#[path = "page_tests.rs"]
mod tests;
