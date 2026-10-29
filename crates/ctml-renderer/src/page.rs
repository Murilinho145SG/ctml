use cosmic_text::{Buffer, Color, Cursor, FontSystem, Metrics, Shaping, SwashCache};
use winit::window::CursorIcon;

use crate::{
    StyleRule, UiNode,
    font_family::resolve_family_list,
    font_metrics::normal_line_height,
    page::pixel::{PixelRenderer, fill_rect},
    style::{TextRun, TextStyle, compile_runs, synthesize_font_variant_caps},
};

mod backgrounds;
mod decorations;
mod focus;
mod font_features;
mod font_raster;
mod interactions;
mod pixel;

const PAGE_PADDING: f32 = 8.0;
const SELECTION_COLOR: [u8; 3] = [173, 216, 230];

pub(crate) struct PageRasterizer {
    nodes: Vec<UiNode>,
    stylesheet: Vec<StyleRule>,
    runs: Vec<TextRun>,
    cursor_spans: Vec<CursorSpan>,
    font_system: FontSystem,
    cache: SwashCache,
    buffer: Buffer,
    root_style: TextStyle,
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
    pub(crate) fn new(nodes: &[UiNode], stylesheet: &[StyleRule]) -> Self {
        let (compiled_runs, root_style) = compile_runs(nodes, stylesheet, &[], &[], &[], false);
        let focus_manager = focus::FocusManager::new(nodes);
        let mut font_system = FontSystem::new();
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
            Metrics::new(root_style.font_size, root_style.line_height),
        );

        Self {
            nodes: nodes.to_vec(),
            stylesheet: stylesheet.to_vec(),
            runs,
            cursor_spans,
            font_system,
            cache: SwashCache::new(),
            buffer,
            root_style,
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
        self.buffer.set_size(
            Some((width as f32 - PAGE_PADDING * 2.0).max(1.0)),
            Some((height as f32 - PAGE_PADDING * 2.0).max(1.0)),
        );
        let run_families = self
            .runs
            .iter()
            .map(|run| resolve_family_list(&run.style.family, &self.font_system))
            .collect::<Vec<_>>();
        let attributes = self
            .runs
            .iter()
            .zip(&run_families)
            .map(|(run, family)| {
                let mut style = run.style.clone();
                if style.line_height_normal {
                    style.line_height = normal_line_height(&mut self.font_system, &style, family)
                        .unwrap_or(style.font_size * 1.2);
                }
                style.to_attrs(family)
            })
            .collect::<Vec<_>>();
        let root_family = resolve_family_list(&self.root_style.family, &self.font_system);
        let mut root_style = self.root_style.clone();
        if root_style.line_height_normal {
            root_style.line_height =
                normal_line_height(&mut self.font_system, &root_style, &root_family)
                    .unwrap_or(root_style.font_size * 1.2);
        }
        let root_attributes = root_style.to_attrs(&root_family);
        self.buffer.set_rich_text(
            self.runs
                .iter()
                .zip(&attributes)
                .map(|(run, attrs)| (run.text.as_str(), attrs.clone())),
            &root_attributes,
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

    fn rebuild_interaction_styles(&mut self) {
        let (compiled_runs, root_style) = compile_runs(
            &self.nodes,
            &self.stylesheet,
            &self.hovered_path,
            &self.active_path,
            &self.focused_path,
            self.keyboard_focus,
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
        self.draw_selection(&mut pixels, width, height);
        let Self {
            runs,
            buffer,
            font_system,
            cache,
            ..
        } = self;
        font_raster::draw(runs, font_system, buffer, &mut pixels, width, height);
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

struct CursorSpan {
    line: usize,
    start: usize,
    end: usize,
    icon: CursorIcon,
    hidden: bool,
    element_path: Vec<usize>,
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
                element_path: run.element_path.clone(),
            });
            index = end;
        }
    }
    spans
}

#[cfg(test)]
#[path = "page_tests.rs"]
mod tests;
