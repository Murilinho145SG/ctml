use cosmic_text::{Buffer, Color, Cursor, FontSystem, Metrics, SwashCache};
use winit::window::CursorIcon;

use crate::{
    StyleRule, UiNode,
    font_family::resolve_family_list,
    page::pixel::PixelRenderer,
    style::{TextRun, TextStyle, compile_runs_with_font_system, synthesize_font_variant_caps},
};

mod backgrounds;
mod cursor_spans;
mod decorations;
mod focus;
mod font_faces;
mod font_features;
mod font_raster;
mod highlights;
mod interactions;
mod layout;
mod pixel;
mod sizing;
mod viewport;

const PAGE_PADDING: f32 = 8.0;

use cursor_spans::{CursorSpan, cursor_spans};
use font_faces::register_font_faces;
use layout::shape_buffer;
use sizing::{
    block_spacer_heights, cursor_with_flow_spacers, cursor_without_flow_spacers, flow_spacer_lines,
    insert_block_spacers, line_layout_sizes, resolve_width,
};

pub(crate) struct PageRasterizer {
    nodes: Vec<UiNode>,
    stylesheet: Vec<StyleRule>,
    runs: Vec<TextRun>,
    cursor_spans: Vec<CursorSpan>,
    spacer_line_flags: Vec<bool>,
    font_system: FontSystem,
    cache: SwashCache,
    buffer: Buffer,
    root_style: TextStyle,
    selection_background_color: [u8; 4],
    selection: Option<(Cursor, Cursor)>,
    selection_anchor: Option<Cursor>,
    layout_size: Option<(u32, u32)>,
    hovered_path: Vec<usize>,
    active_path: Vec<usize>,
    focused_path: Vec<usize>,
    keyboard_focus: bool,
    focus_manager: focus::FocusManager,
}

impl PageRasterizer {
    #[cfg(test)]
    pub(crate) fn new(nodes: &[UiNode], stylesheet: &[StyleRule]) -> Self {
        Self::new_with_fonts(nodes, stylesheet, &[])
    }

    pub(crate) fn new_with_fonts(
        nodes: &[UiNode],
        stylesheet: &[StyleRule],
        fonts: &[crate::FontFace],
    ) -> Self {
        let mut font_system = FontSystem::new();
        register_font_faces(&mut font_system, fonts);
        #[cfg(target_os = "linux")]
        if let Some(family) = crate::fontconfig::matched_family("Arial") {
            font_system.db_mut().set_sans_serif_family(family);
        }
        #[cfg(target_os = "linux")]
        if let Some(family) = crate::fontconfig::matched_family("Times New Roman") {
            font_system.db_mut().set_serif_family(family);
        }
        #[cfg(target_os = "linux")]
        if let Some(family) = crate::fontconfig::matched_family("Courier New") {
            font_system.db_mut().set_monospace_family(family);
        }
        let (compiled_runs, root_style) = compile_runs_with_font_system(
            nodes,
            stylesheet,
            &[],
            &[],
            &[],
            false,
            Some(&font_system),
            ctml_css::ViewportSize::default(),
        );
        let focus_manager = focus::FocusManager::new(nodes);
        let selection_background_color = crate::css_style::selection_background_color(stylesheet);
        let runs = compiled_runs
            .into_iter()
            .flat_map(|run| {
                let family = resolve_family_list(&run.style.family, &font_system);
                if font_features::supports_caps_features(&font_system, &family, &run.style) {
                    vec![run]
                } else {
                    synthesize_font_variant_caps(run)
                }
            })
            .collect::<Vec<_>>();
        let cursor_spans = cursor_spans(&runs);
        let buffer = Buffer::new(
            &mut font_system,
            Metrics::new(
                root_style.font_size.max(1.0),
                root_style.line_height.max(1.0),
            ),
        );

        Self {
            nodes: nodes.to_vec(),
            stylesheet: stylesheet.to_vec(),
            runs,
            cursor_spans,
            spacer_line_flags: Vec::new(),
            font_system,
            cache: SwashCache::new(),
            buffer,
            root_style,
            selection_background_color,
            selection: None,
            selection_anchor: None,
            layout_size: None,
            hovered_path: Vec::new(),
            active_path: Vec::new(),
            focused_path: Vec::new(),
            keyboard_focus: false,
            focus_manager,
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
        self.runs.retain(|run| !run.style.flow_spacer);
        let selection = self.selection.map(|(start, end)| {
            (
                cursor_without_flow_spacers(start, &self.spacer_line_flags),
                cursor_without_flow_spacers(end, &self.spacer_line_flags),
            )
        });
        let selection_anchor = self
            .selection_anchor
            .map(|cursor| cursor_without_flow_spacers(cursor, &self.spacer_line_flags));
        let viewport_size = viewport::resize_styles(self, width, height);
        let available_width = (width as f32 - PAGE_PADDING * 2.0).max(1.0);
        let font_size = self.root_style.font_size;
        let min_width = self.root_style.min_width.as_deref().and_then(|value| {
            resolve_width(
                value,
                available_width,
                font_size,
                self.root_style.line_height,
                self.root_style.font_relative_units,
                viewport_size,
            )
        });
        let max_width = self.root_style.max_width.as_deref().and_then(|value| {
            resolve_width(
                value,
                available_width,
                font_size,
                self.root_style.line_height,
                self.root_style.font_relative_units,
                viewport_size,
            )
        });
        let mut content_width = self
            .root_style
            .width
            .as_deref()
            .and_then(|value| {
                resolve_width(
                    value,
                    available_width,
                    font_size,
                    self.root_style.line_height,
                    self.root_style.font_relative_units,
                    viewport_size,
                )
            })
            .unwrap_or(available_width);
        match (min_width, max_width) {
            (Some(min), Some(max)) if min > max => content_width = content_width.max(min),
            (Some(min), Some(max)) => content_width = content_width.max(min).min(max),
            (Some(min), None) => content_width = content_width.max(min),
            (None, Some(max)) => content_width = content_width.min(max),
            (None, None) => {}
        }
        let content_width = content_width.max(1.0);
        let content_height = (height as f32 - PAGE_PADDING * 2.0).max(1.0);
        self.cursor_spans = cursor_spans(&self.runs);
        shape_buffer(self, content_width, content_height);
        let metrics = self.buffer.metrics();
        let line_sizes = line_layout_sizes(&self.runs, self.buffer.lines.len());
        let spacer_heights = block_spacer_heights(
            &self.runs,
            &line_sizes,
            &self.buffer,
            content_height,
            metrics.font_size,
            metrics.line_height,
        );
        if let Some(runs) = insert_block_spacers(&self.runs, &spacer_heights) {
            self.runs = runs;
            self.cursor_spans = cursor_spans(&self.runs);
            shape_buffer(self, content_width, content_height);
        }
        if layout::apply_first_line_word_spacing(&self.buffer, &mut self.runs, &self.cursor_spans) {
            self.cursor_spans = cursor_spans(&self.runs);
            shape_buffer(self, content_width, content_height);
        }
        if layout::apply_first_line_text_transform(&self.buffer, &mut self.runs, &self.cursor_spans)
        {
            self.cursor_spans = cursor_spans(&self.runs);
            shape_buffer(self, content_width, content_height);
        }
        if layout::apply_first_line_line_height(&self.buffer, &mut self.runs, &self.cursor_spans) {
            self.cursor_spans = cursor_spans(&self.runs);
            shape_buffer(self, content_width, content_height);
        }
        self.spacer_line_flags = flow_spacer_lines(&self.runs, self.buffer.lines.len());
        self.selection = selection.map(|(start, end)| {
            (
                cursor_with_flow_spacers(start, &self.spacer_line_flags),
                cursor_with_flow_spacers(end, &self.spacer_line_flags),
            )
        });
        self.selection_anchor = selection_anchor
            .map(|cursor| cursor_with_flow_spacers(cursor, &self.spacer_line_flags));
    }

    pub(crate) fn redraw(&mut self, queue: &wgpu::Queue, texture: &wgpu::Texture) {
        let Some((width, height)) = self.layout_size else {
            return;
        };
        self.upload_pixels(queue, texture, width, height);
    }

    fn rebuild_interaction_styles(&mut self) {
        let (compiled_runs, root_style) = compile_runs_with_font_system(
            &self.nodes,
            &self.stylesheet,
            &self.hovered_path,
            &self.active_path,
            &self.focused_path,
            self.keyboard_focus,
            Some(&self.font_system),
            self.layout_size
                .map_or_else(ctml_css::ViewportSize::default, |(width, height)| {
                    ctml_css::ViewportSize {
                        width: width as f32,
                        height: height as f32,
                    }
                }),
        );
        self.runs = compiled_runs
            .into_iter()
            .flat_map(|run| {
                let family = resolve_family_list(&run.style.family, &self.font_system);
                if font_features::supports_caps_features(&self.font_system, &family, &run.style) {
                    vec![run]
                } else {
                    synthesize_font_variant_caps(run)
                }
            })
            .collect();
        self.cursor_spans = cursor_spans(&self.runs);
        self.root_style = root_style;
        if let Some((width, height)) = self.layout_size {
            self.prepare_layout(width, height);
        }
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
        backgrounds::draw(
            &self.runs,
            &self.buffer,
            &mut pixels,
            width,
            height,
            self.root_style.background_color.is_some(),
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
            let use_custom_decorations = !run.decorations.is_empty();
            let mut renderer = PixelRenderer {
                pixels: &mut pixels,
                width,
                height,
                skip_ink_mask: None,
            };
            if use_custom_decorations {
                decorations::render_patterned_decorations(
                    &mut renderer,
                    &run,
                    default_color,
                    &auto_ink_mask,
                    &all_ink_mask,
                );
            } else {
                cosmic_text::render_decoration(&mut renderer, &run, default_color);
            }
        }
        self.draw_pseudo_decorations(&mut pixels, width, height, &auto_ink_mask, &all_ink_mask);
        pixels
    }
}

fn is_underline_descender(character: char) -> bool {
    matches!(character, 'g' | 'j' | 'p' | 'q' | 'y')
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

#[cfg(test)]
#[path = "page_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "page_tests/font_face.rs"]
mod font_face_tests;
