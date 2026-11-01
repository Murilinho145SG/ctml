use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use cosmic_text::{Buffer, Cursor, FontSystem, Metrics, Scroll, SwashCache};
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
mod layout_runs;
mod local_writing;
mod opacity_groups;
mod outlines;
mod overflow_clip;
mod pixel;
mod prepare;
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
pub(super) use layout_runs::{FlexLayoutOffsetKey, adjusted_layout_runs};
use layout_runs::{FlexLayoutOffsets, PageLayoutRun};

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
    flex_layout_offsets: FlexLayoutOffsets,
    flex_item_box_x: HashMap<usize, f32>,
    flex_column_extents: HashMap<usize, (f32, f32)>,
    root_scroll: Scroll,
    nested_scroll: HashMap<usize, (f32, f32)>,
    spacer_line_flags: Vec<bool>,
    font_system: FontSystem,
    cache: SwashCache,
    frame_pixels: Vec<u8>,
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
    has_interaction_rules: bool,
}

impl PageRasterizer {
    pub(super) fn layout_runs(&self) -> impl Iterator<Item = PageLayoutRun<'_>> + '_ {
        adjusted_layout_runs(&self.buffer, &self.flex_layout_offsets)
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
        let has_interaction_rules = stylesheet.iter().any(|rule| {
            rule.selectors.iter().any(|selector| {
                [
                    ":hover",
                    ":active",
                    ":focus",
                    ":focus-visible",
                    ":focus-within",
                ]
                .iter()
                .any(|pseudo| selector.contains(pseudo))
            })
        });
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
            flex_layout_offsets: HashMap::new(),
            flex_item_box_x: HashMap::new(),
            flex_column_extents: HashMap::new(),
            root_scroll: Scroll::default(),
            nested_scroll: HashMap::new(),
            spacer_line_flags: Vec::new(),
            font_system,
            cache: SwashCache::new(),
            frame_pixels: Vec::new(),
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
            has_interaction_rules,
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

    fn writing_mode(&self) -> crate::style::WritingMode {
        self.root_style.writing_mode
    }
}

#[cfg(test)]
#[path = "page_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "page_tests/font_face.rs"]
mod font_face_tests;
