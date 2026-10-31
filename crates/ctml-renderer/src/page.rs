use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use cosmic_text::{Buffer, Cursor, FontSystem, LayoutRun, Metrics, Scroll, SwashCache};
use winit::window::CursorIcon;

use crate::{
    StyleRule, UiNode,
    font_family::resolve_family_list,
    page::pixel::PixelRenderer,
    style::{
        BackgroundBlendMode, TextRun, TextStyle, compile_runs_with_opacity_map,
        synthesize_font_variant_caps,
    },
};

mod animations;
mod background_blend;
mod background_images;
mod backgrounds;
mod block_sizing;
mod border_images;
mod border_radius;
mod borders;
mod box_shadows;
mod content_visibility;
mod cursor_spans;
mod decorations;
mod first_line_boxes;
mod focus;
mod font_faces;
mod font_features;
mod font_raster;
mod highlight_paint;
mod highlights;
mod hyphenation;
mod inline_boxes;
mod interactions;
mod layout;
mod local_writing;
mod opacity_groups;
mod outlines;
mod overflow_clip;
mod pixel;
mod render;
mod scrollbars;
mod scrolling;
mod selection_backgrounds;
mod sizing;
mod sizing_height;
mod sizing_intrinsic;
mod sizing_padding;
mod vertical;
mod viewport;

const PAGE_PADDING: f32 = 8.0;

use block_sizing::{block_spacer_heights, insert_block_spacers};
use cursor_spans::{CursorSpan, cursor_spans};
use font_faces::register_font_faces;
use layout::shape_buffer;
use sizing::{
    cursor_with_flow_spacers, cursor_without_flow_spacers, flow_spacer_lines, line_layout_sizes,
    resolve_constrained_size, select_axis_size,
};
use sizing_intrinsic::{
    measure_root_intrinsic_widths, resolve_constrained_size as resolve_intrinsic_size,
};

pub(crate) struct PageRasterizer {
    nodes: Vec<UiNode>,
    stylesheet: Vec<StyleRule>,
    color_profiles: Vec<ctml_css::CssColorProfile>,
    keyframes: Vec<ctml_css::CssKeyframes>,
    animations_enabled: bool,
    runs: Vec<TextRun>,
    cursor_spans: Vec<CursorSpan>,
    overflow_clips: HashMap<usize, overflow_clip::ClipRect>,
    overflow_geometry: HashMap<usize, overflow_clip::BlockGeometry>,
    flex_wrap_line_offsets: HashMap<(usize, usize), f32>,
    root_scroll: Scroll,
    nested_scroll: HashMap<usize, (f32, f32)>,
    spacer_line_flags: Vec<bool>,
    font_system: FontSystem,
    cache: SwashCache,
    buffer: Buffer,
    root_style: TextStyle,
    root_opacity: f32,
    root_opacity_transition: Option<animations::OpacityTransition>,
    opacity_groups: HashMap<usize, f32>,
    opacity_transitions: HashMap<usize, animations::OpacityTransition>,
    element_colors: HashMap<usize, [u8; 4]>,
    color_transitions: HashMap<usize, animations::ColorTransition>,
    element_background_colors: HashMap<usize, [u8; 4]>,
    background_color_transitions: HashMap<usize, animations::ColorTransition>,
    element_outline_colors: HashMap<usize, [u8; 4]>,
    outline_color_transitions: HashMap<usize, animations::ColorTransition>,
    element_border_colors: HashMap<usize, [[u8; 4]; 4]>,
    border_color_transitions: HashMap<usize, [animations::ColorTransition; 4]>,
    element_visibility: HashMap<usize, bool>,
    visibility_transitions: HashMap<usize, animations::VisibilityTransition>,
    keyframe_animations:
        HashMap<usize, Vec<(usize, animations::keyframes::KeyframeAnimationState)>>,
    mix_blend_modes: HashMap<usize, BackgroundBlendMode>,
    isolation_groups: HashSet<usize>,
    decoration_opacity_scales: Vec<f32>,
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
    pub(super) fn layout_runs(&self) -> impl Iterator<Item = LayoutRun<'_>> + '_ {
        adjusted_layout_runs(&self.buffer, &self.flex_wrap_line_offsets)
    }

    #[cfg(test)]
    pub(crate) fn new(nodes: &[UiNode], stylesheet: &[StyleRule]) -> Self {
        Self::new_with_fonts(nodes, stylesheet, &[])
    }

    #[cfg(test)]
    pub(crate) fn new_with_fonts(
        nodes: &[UiNode],
        stylesheet: &[StyleRule],
        fonts: &[crate::FontFace],
    ) -> Self {
        Self::new_with_fonts_color_profiles_and_keyframes(nodes, stylesheet, &[], true, fonts, &[])
    }

    #[cfg(test)]
    pub(crate) fn new_with_keyframes(
        nodes: &[UiNode],
        stylesheet: &[StyleRule],
        keyframes: &[ctml_css::CssKeyframes],
    ) -> Self {
        Self::new_with_fonts_color_profiles_and_keyframes(
            nodes,
            stylesheet,
            keyframes,
            true,
            &[],
            &[],
        )
    }

    pub(crate) fn new_with_fonts_color_profiles_and_keyframes(
        nodes: &[UiNode],
        stylesheet: &[StyleRule],
        keyframes: &[ctml_css::CssKeyframes],
        animations_enabled: bool,
        fonts: &[crate::FontFace],
        color_profiles: &[ctml_css::CssColorProfile],
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
        let (
            compiled_runs,
            root_style,
            opacity_groups,
            mix_blend_modes,
            isolation_groups,
            transition_styles,
        ) = compile_runs_with_opacity_map(
            nodes,
            stylesheet,
            &[],
            &[],
            &[],
            false,
            Some(&font_system),
            color_profiles,
            ctml_css::ViewportSize::default(),
        );
        let focus_manager = focus::FocusManager::new(nodes);
        let selection_background_color =
            crate::css_style::selection_background_color(stylesheet, color_profiles);
        let mut runs = compiled_runs
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
        hyphenation::apply_auto_hyphenation(&mut runs);
        let cursor_spans = cursor_spans(&runs);
        let keyframes = keyframes.to_vec();
        let keyframe_animations = if animations_enabled {
            animations::keyframes::build_states(
                &transition_styles,
                &keyframes,
                &opacity_groups,
                root_style.opacity,
                Instant::now(),
                &HashMap::new(),
            )
        } else {
            HashMap::new()
        };
        let root_opacity = root_style.opacity;
        let mut root_style = root_style;
        root_style.opacity = 1.0;
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
            color_profiles: color_profiles.to_vec(),
            keyframes,
            animations_enabled,
            runs,
            cursor_spans,
            overflow_clips: HashMap::new(),
            overflow_geometry: HashMap::new(),
            flex_wrap_line_offsets: HashMap::new(),
            root_scroll: Scroll::default(),
            nested_scroll: HashMap::new(),
            spacer_line_flags: Vec::new(),
            font_system,
            cache: SwashCache::new(),
            buffer,
            root_style,
            root_opacity,
            root_opacity_transition: None,
            opacity_groups,
            opacity_transitions: HashMap::new(),
            element_colors: transition_styles
                .iter()
                .map(|(node_id, style)| (*node_id, style.color_value))
                .collect(),
            color_transitions: HashMap::new(),
            element_background_colors: transition_styles
                .iter()
                .map(|(node_id, style)| (*node_id, style.background_color_value))
                .collect(),
            background_color_transitions: HashMap::new(),
            element_outline_colors: transition_styles
                .iter()
                .map(|(node_id, style)| (*node_id, style.outline_color_value))
                .collect(),
            outline_color_transitions: HashMap::new(),
            element_border_colors: transition_styles
                .iter()
                .map(|(node_id, style)| (*node_id, style.border_color_value))
                .collect(),
            border_color_transitions: HashMap::new(),
            element_visibility: transition_styles
                .iter()
                .map(|(node_id, style)| (*node_id, style.visibility_hidden))
                .collect(),
            visibility_transitions: HashMap::new(),
            keyframe_animations,
            mix_blend_modes,
            isolation_groups,
            decoration_opacity_scales: Vec::new(),
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
        for run in &mut self.runs {
            if run.style.flex_wrap_break {
                run.style.line_height = 1.0;
                run.style.flex_wrap_gap_applied = false;
            }
        }
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
        let (layout_width, layout_height) =
            vertical::layout_dimensions(self.writing_mode(), width, height);
        let available_width = (layout_width as f32 - PAGE_PADDING * 2.0).max(1.0);
        let available_height = (layout_height as f32 - PAGE_PADDING * 2.0).max(1.0);
        let font_size = self.root_style.font_size;
        let root_intrinsic_widths = measure_root_intrinsic_widths(self);
        let vertical = self.writing_mode() != crate::style::WritingMode::HorizontalTb;
        let (
            inline_physical,
            inline_min_physical,
            inline_max_physical,
            inline_name,
            inline_min_name,
            inline_max_name,
        ) = if vertical {
            (
                &self.root_style.height,
                &self.root_style.min_height,
                &self.root_style.max_height,
                "height",
                "min-height",
                "max-height",
            )
        } else {
            (
                &self.root_style.width,
                &self.root_style.min_width,
                &self.root_style.max_width,
                "width",
                "min-width",
                "max-width",
            )
        };
        let inline_size = select_axis_size(
            self.root_style.inline_size.as_ref(),
            inline_physical.as_ref(),
            &self.root_style.size_property_orders,
            "inline-size",
            inline_name,
        )
        .flatten();
        let min_inline_size = select_axis_size(
            self.root_style.min_inline_size.as_ref(),
            inline_min_physical.as_ref(),
            &self.root_style.size_property_orders,
            "min-inline-size",
            inline_min_name,
        )
        .flatten();
        let max_inline_size = select_axis_size(
            self.root_style.max_inline_size.as_ref(),
            inline_max_physical.as_ref(),
            &self.root_style.size_property_orders,
            "max-inline-size",
            inline_max_name,
        )
        .flatten();
        let (
            block_physical,
            block_min_physical,
            block_max_physical,
            block_name,
            block_min_name,
            block_max_name,
        ) = if vertical {
            (
                &self.root_style.width,
                &self.root_style.min_width,
                &self.root_style.max_width,
                "width",
                "min-width",
                "max-width",
            )
        } else {
            (
                &self.root_style.height,
                &self.root_style.min_height,
                &self.root_style.max_height,
                "height",
                "min-height",
                "max-height",
            )
        };
        let block_size = select_axis_size(
            self.root_style.block_size.as_ref(),
            block_physical.as_ref(),
            &self.root_style.size_property_orders,
            "block-size",
            block_name,
        )
        .flatten();
        let min_block_size = select_axis_size(
            self.root_style.min_block_size.as_ref(),
            block_min_physical.as_ref(),
            &self.root_style.size_property_orders,
            "min-block-size",
            block_min_name,
        )
        .flatten();
        let max_block_size = select_axis_size(
            self.root_style.max_block_size.as_ref(),
            block_max_physical.as_ref(),
            &self.root_style.size_property_orders,
            "max-block-size",
            block_max_name,
        )
        .flatten();
        let content_width = resolve_intrinsic_size(
            inline_size,
            min_inline_size,
            max_inline_size,
            available_width,
            font_size,
            self.root_style.line_height,
            self.root_style.font_relative_units,
            viewport_size,
            root_intrinsic_widths,
        );
        let content_height = resolve_constrained_size(
            block_size,
            min_block_size,
            max_block_size,
            available_height,
            font_size,
            self.root_style.line_height,
            self.root_style.font_relative_units,
            viewport_size,
        );
        let page_writing_mode = self.writing_mode();
        layout::insert_first_letter_spacing(&mut self.runs, content_width, page_writing_mode);
        self.cursor_spans = cursor_spans(&self.runs);
        shape_buffer(self, content_width, content_height);
        let skipped_auto_nodes = if self
            .runs
            .iter()
            .any(|run| run.style.content_visibility_auto)
        {
            let mut auto_visibility_buffer = self.buffer.clone();
            auto_visibility_buffer.set_size(Some(content_width.max(content_height)), None);
            auto_visibility_buffer.set_scroll(Scroll::default());
            auto_visibility_buffer.shape_until_scroll(&mut self.font_system, false);
            let auto_visibility_geometries = overflow_clip::block_geometries(
                &self.runs,
                &mut auto_visibility_buffer,
                &mut self.font_system,
                layout_width,
                layout_height,
                page_writing_mode,
                &self.flex_wrap_line_offsets,
            );
            content_visibility::offscreen_auto_nodes(
                &self.runs,
                &auto_visibility_geometries,
                layout_width,
                layout_height,
                content_visibility::vertical_scroll_offset(&self.buffer),
                self.selection.is_some(),
                &self.focused_path,
            )
        } else {
            Default::default()
        };
        if !skipped_auto_nodes.is_empty() {
            content_visibility::skip_auto_contents(&mut self.runs, &skipped_auto_nodes);
            self.cursor_spans = cursor_spans(&self.runs);
        }
        let metrics = self.buffer.metrics();
        let line_sizes = line_layout_sizes(&self.runs, self.buffer.lines.len());
        let spacer_heights = block_spacer_heights(
            &self.runs,
            &line_sizes,
            &self.buffer,
            content_height,
            content_width,
            metrics.font_size,
            metrics.line_height,
            self.writing_mode(),
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
        for _ in 0..8 {
            if !layout::apply_first_line_letter_spacing(
                &self.buffer,
                &mut self.runs,
                &self.cursor_spans,
            ) {
                break;
            }
            self.cursor_spans = cursor_spans(&self.runs);
            shape_buffer(self, content_width, content_height);
        }
        if layout::insert_first_line_spacing(
            &self.buffer,
            &mut self.runs,
            &self.cursor_spans,
            content_width,
            page_writing_mode,
        ) {
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
        if !scrolling::has_user_scrollable_axis(
            self.root_style.overflow_x,
            self.root_style.overflow_y,
        ) {
            self.root_scroll = Scroll::default();
        }
        self.buffer.set_scroll(self.root_scroll);
        self.buffer.shape_until_scroll(&mut self.font_system, false);
        self.root_scroll = self.buffer.scroll();
        self.refresh_overflow_geometry(width, height);
    }

    fn refresh_overflow_geometry(&mut self, width: u32, height: u32) {
        let (layout_width, layout_height) =
            vertical::layout_dimensions(self.writing_mode(), width, height);
        let writing_mode = self.writing_mode();
        let overflow_geometry = overflow_clip::block_geometries(
            &self.runs,
            &mut self.buffer,
            &mut self.font_system,
            layout_width,
            layout_height,
            writing_mode,
            &self.flex_wrap_line_offsets,
        );
        self.overflow_geometry = overflow_geometry;
        self.overflow_clips = self
            .overflow_geometry
            .iter()
            .filter(|(node_id, _)| {
                self.runs.iter().any(|run| {
                    run.style.layout_sizes.iter().any(|size| {
                        size.node_id == **node_id
                            && (size.overflow_x != crate::style::Overflow::Visible
                                || size.overflow_y != crate::style::Overflow::Visible
                                || size.paint_contained)
                    })
                })
            })
            .map(|(node_id, geometry)| (*node_id, geometry.clip))
            .collect();
        self.nested_scroll.retain(|node_id, _| {
            self.overflow_geometry.contains_key(node_id)
                && self.runs.iter().any(|run| {
                    run.style.layout_sizes.iter().any(|size| {
                        let (x_scrollable, y_scrollable) =
                            scrolling::user_scrollable_axes(size.overflow_x, size.overflow_y);
                        size.node_id == *node_id && (x_scrollable || y_scrollable)
                    })
                })
        });
        for (node_id, (offset_x, offset_y)) in &mut self.nested_scroll {
            if let Some(geometry) = self.overflow_geometry.get(node_id) {
                let (x_scrollable, y_scrollable) = self
                    .runs
                    .iter()
                    .flat_map(|run| &run.style.layout_sizes)
                    .find(|size| size.node_id == *node_id)
                    .map_or((false, false), |size| {
                        scrolling::user_scrollable_axes(size.overflow_x, size.overflow_y)
                    });
                let (max_x, max_y) = match writing_mode {
                    crate::style::WritingMode::HorizontalTb => (
                        (geometry.content_inline_extent
                            - (geometry.clip.right - geometry.clip.left))
                            .max(0.0),
                        (geometry.content_block_extent
                            - (geometry.clip.bottom - geometry.clip.top))
                            .max(0.0),
                    ),
                    crate::style::WritingMode::VerticalRl
                    | crate::style::WritingMode::VerticalLr => (
                        (geometry.content_block_extent
                            - (geometry.clip.bottom - geometry.clip.top))
                            .max(0.0),
                        (geometry.content_inline_extent
                            - (geometry.clip.right - geometry.clip.left))
                            .max(0.0),
                    ),
                };
                *offset_x = if x_scrollable {
                    offset_x.clamp(0.0, max_x)
                } else {
                    0.0
                };
                *offset_y = offset_y.clamp(0.0, if y_scrollable { max_y } else { 0.0 });
            }
        }
    }

    pub(crate) fn redraw(&mut self, queue: &wgpu::Queue, texture: &wgpu::Texture) {
        let Some((width, height)) = self.layout_size else {
            return;
        };
        self.upload_pixels(queue, texture, width, height);
    }

    pub(crate) fn advance_animations(&mut self, now: Instant) -> Option<bool> {
        if !self.animations_enabled {
            return None;
        }
        animations::advance(self, now)
    }

    #[cfg(test)]
    pub(crate) fn keyframe_animation_start(&self, node_id: usize) -> Option<Instant> {
        self.keyframe_animations
            .get(&node_id)
            .and_then(|states| states.first())
            .map(|(_, state)| state.starts_at)
    }

    fn rebuild_interaction_styles(&mut self) {
        self.rebuild_interaction_styles_at(Instant::now());
    }

    pub(crate) fn rebuild_interaction_styles_at(&mut self, now: Instant) {
        let previous_root_opacity = self.root_opacity;
        let previous_opacity_groups = self.opacity_groups.clone();
        let previous_element_colors = self.element_colors.clone();
        let previous_element_background_colors = self.element_background_colors.clone();
        let previous_element_outline_colors = self.element_outline_colors.clone();
        let previous_element_border_colors = self.element_border_colors.clone();
        let previous_element_visibility = self.element_visibility.clone();
        let (
            compiled_runs,
            root_style,
            opacity_groups,
            mix_blend_modes,
            isolation_groups,
            transition_styles,
        ) = compile_runs_with_opacity_map(
            &self.nodes,
            &self.stylesheet,
            &self.hovered_path,
            &self.active_path,
            &self.focused_path,
            self.keyboard_focus,
            Some(&self.font_system),
            &self.color_profiles,
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
        self.opacity_groups = opacity_groups;
        self.keyframe_animations = if self.animations_enabled {
            animations::keyframes::build_states(
                &transition_styles,
                &self.keyframes,
                &self.opacity_groups,
                root_style.opacity,
                now,
                &self.keyframe_animations,
            )
        } else {
            HashMap::new()
        };
        self.element_colors = transition_styles
            .iter()
            .map(|(node_id, style)| (*node_id, style.color_value))
            .collect();
        self.element_background_colors = transition_styles
            .iter()
            .map(|(node_id, style)| (*node_id, style.background_color_value))
            .collect();
        self.element_outline_colors = transition_styles
            .iter()
            .map(|(node_id, style)| (*node_id, style.outline_color_value))
            .collect();
        self.element_border_colors = transition_styles
            .iter()
            .map(|(node_id, style)| (*node_id, style.border_color_value))
            .collect();
        self.element_visibility = transition_styles
            .iter()
            .map(|(node_id, style)| (*node_id, style.visibility_hidden))
            .collect();
        self.mix_blend_modes = mix_blend_modes;
        self.isolation_groups = isolation_groups;
        self.decoration_opacity_scales.clear();
        animations::begin_transitions(
            self,
            previous_root_opacity,
            &previous_opacity_groups,
            &root_style,
            &transition_styles,
            &previous_element_colors,
            &previous_element_background_colors,
            &previous_element_outline_colors,
            &previous_element_border_colors,
            &previous_element_visibility,
        );
        self.root_style = root_style;
        self.root_style.opacity = 1.0;
        self.cursor_spans = cursor_spans(&self.runs);
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
        let mode = self.writing_mode();
        let (layout_width, layout_height) = vertical::layout_dimensions(mode, width, height);
        let pixels = self.render_layout_pixels(layout_width, layout_height, mode);
        vertical::orient_pixels(mode, pixels, width, height)
    }

    fn writing_mode(&self) -> crate::style::WritingMode {
        self.root_style.writing_mode
    }
}

pub(super) fn adjusted_layout_runs<'a>(
    buffer: &'a Buffer,
    offsets: &'a HashMap<(usize, usize), f32>,
) -> impl Iterator<Item = LayoutRun<'a>> + 'a {
    buffer.layout_runs().map(move |mut run| {
        let offset = run
            .glyphs
            .first()
            .and_then(|glyph| offsets.get(&(run.line_i, glyph.start)))
            .copied()
            .unwrap_or_default();
        run.line_top += offset;
        run.line_y += offset;
        run
    })
}

#[cfg(test)]
#[path = "page_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "page_tests/font_face.rs"]
mod font_face_tests;
