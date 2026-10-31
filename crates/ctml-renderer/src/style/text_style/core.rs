use cosmic_text::{Align, Stretch, Style, Weight};
use ctml_css::{FontRelativeUnits, ViewportSize};
use std::{collections::HashMap, sync::Arc};
use winit::window::CursorIcon;

use super::{
    BackgroundAttachment, BackgroundBlendMode, BackgroundBox, BackgroundImage, BackgroundPosition,
    BackgroundRepeat, BackgroundSize, BorderImageOutset, BorderImageRepeat, BorderImageSlice,
    BorderImageWidth, BorderLineStyle, BoxDecorationBreak, BoxShadow, Direction, FontVariantCaps,
    FontVariantPosition, Hyphens, LayoutHeight, LayoutSize, LineBreak, Overflow, OverflowWrap,
    TextDecorationSkipInk, TextDecorationStyle, TextEmphasisPosition, TextEmphasisSide,
    TextEmphasisStyle, TextOrientation, TextOverflow, TextRendering, TextShadow, TextTransform,
    TextWrapMode, UnicodeBidi, UserSelect, WhiteSpace, WordBreak, WritingMode,
};

pub(super) const DEFAULT_FONT_SIZE: f32 = 16.0;
pub(super) const DEFAULT_LINE_HEIGHT: f32 = 20.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum FlexAlignment {
    #[default]
    FlexStart,
    Start,
    Center,
    Stretch,
    FlexEnd,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TransitionEasing {
    Linear,
    #[default]
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier([u32; 4]),
    Steps {
        count: u32,
        position: StepPosition,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StepPosition {
    JumpStart,
    JumpEnd,
    JumpNone,
    JumpBoth,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum AnimationDirection {
    #[default]
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum AnimationFillMode {
    #[default]
    None,
    Forwards,
    Backwards,
    Both,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum AnimationPlayState {
    #[default]
    Running,
    Paused,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AnimationSpec {
    pub(crate) name: String,
    pub(crate) duration_ms: f32,
    pub(crate) delay_ms: f32,
    pub(crate) iterations: f32,
    pub(crate) direction: AnimationDirection,
    pub(crate) fill_mode: AnimationFillMode,
    pub(crate) play_state: AnimationPlayState,
    pub(crate) easing: TransitionEasing,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FlexBasis {
    Auto,
    Content,
    MinContent,
    MaxContent,
    FitContent,
    Length(f32),
    Percentage(f32),
    Calculation(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TransitionTiming {
    pub(crate) duration_ms: f32,
    pub(crate) delay_ms: f32,
    pub(crate) easing: TransitionEasing,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ElementTransitionStyle {
    pub(crate) opacity: Option<TransitionTiming>,
    pub(crate) color: Option<TransitionTiming>,
    pub(crate) color_value: [u8; 4],
    pub(crate) background_color: Option<TransitionTiming>,
    pub(crate) background_color_value: [u8; 4],
    pub(crate) outline_color: Option<TransitionTiming>,
    pub(crate) outline_color_value: [u8; 4],
    pub(crate) border_color: Option<TransitionTiming>,
    pub(crate) border_color_value: [[u8; 4]; 4],
    pub(crate) visibility: Option<TransitionTiming>,
    pub(crate) visibility_hidden: bool,
    pub(crate) animation: Vec<Option<AnimationSpec>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextStyle {
    pub(crate) font_relative_units: FontRelativeUnits,
    pub(crate) viewport_size: ViewportSize,
    pub(crate) color_profiles: Arc<[ctml_css::CssColorProfile]>,
    pub(crate) dark_color_scheme: bool,
    pub(crate) font_size: f32,
    pub(crate) font_scale: f32,
    pub(crate) line_height: f32,
    pub(crate) line_height_multiplier: Option<f32>,
    pub(crate) line_height_normal: bool,
    pub(crate) vertical_align_offset: f32,
    pub(crate) color: [u8; 4],
    pub(crate) color_opacity: f32,
    pub(crate) color_owner: Option<usize>,
    pub(crate) color_is_explicit: bool,
    pub(crate) color_transition_owner: Option<usize>,
    pub(crate) visibility_owner: Option<usize>,
    pub(crate) visibility_is_explicit: bool,
    pub(crate) opacity: f32,
    pub(crate) transition_opacity: bool,
    pub(crate) transition_duration_ms: f32,
    pub(crate) transition_delay_ms: f32,
    pub(crate) transition_easing: TransitionEasing,
    pub(crate) transition_color: bool,
    pub(crate) transition_color_duration_ms: f32,
    pub(crate) transition_color_delay_ms: f32,
    pub(crate) transition_color_easing: TransitionEasing,
    pub(crate) transition_background_color: bool,
    pub(crate) transition_background_color_duration_ms: f32,
    pub(crate) transition_background_color_delay_ms: f32,
    pub(crate) transition_background_color_easing: TransitionEasing,
    pub(crate) transition_outline_color: bool,
    pub(crate) transition_outline_color_duration_ms: f32,
    pub(crate) transition_outline_color_delay_ms: f32,
    pub(crate) transition_outline_color_easing: TransitionEasing,
    pub(crate) transition_border_color: bool,
    pub(crate) transition_border_color_duration_ms: f32,
    pub(crate) transition_border_color_delay_ms: f32,
    pub(crate) transition_border_color_easing: TransitionEasing,
    pub(crate) transition_visibility: Option<TransitionTiming>,
    pub(crate) animation_name: Vec<Option<String>>,
    pub(crate) animation_duration_ms: Vec<f32>,
    pub(crate) animation_delay_ms: Vec<f32>,
    pub(crate) animation_iterations: Vec<f32>,
    pub(crate) animation_direction: Vec<AnimationDirection>,
    pub(crate) animation_fill_mode: Vec<AnimationFillMode>,
    pub(crate) animation_play_state: Vec<AnimationPlayState>,
    pub(crate) animation_easing: Vec<TransitionEasing>,
    pub(crate) transition_behaviors: Vec<bool>,
    pub(crate) transition_properties: Vec<String>,
    pub(crate) transition_durations_ms: Vec<f32>,
    pub(crate) transition_delays_ms: Vec<f32>,
    pub(crate) transition_easings: Vec<TransitionEasing>,
    pub(crate) background: [u8; 3],
    pub(crate) background_color: Option<[u8; 4]>,
    pub(crate) background_layers: Vec<[u8; 4]>,
    pub(crate) background_images: Vec<BackgroundImage>,
    pub(crate) background_blend_modes: Vec<BackgroundBlendMode>,
    pub(crate) mix_blend_mode: BackgroundBlendMode,
    pub(crate) isolation: bool,
    pub(crate) background_repeats: Vec<BackgroundRepeat>,
    pub(crate) background_positions: Vec<BackgroundPosition>,
    pub(crate) background_sizes: Vec<BackgroundSize>,
    pub(crate) background_attachments: Vec<BackgroundAttachment>,
    pub(crate) background_origins: Vec<BackgroundBox>,
    pub(crate) background_clips: Vec<BackgroundBox>,
    pub(crate) display_none: bool,
    pub(crate) visibility_hidden: bool,
    pub(crate) content_visibility_hidden: bool,
    pub(crate) content_visibility_auto: bool,
    pub(crate) content_skipped: bool,
    pub(crate) content_skipped_by_ancestor: bool,
    pub(crate) paint_contained: bool,
    pub(crate) layout_contained: bool,
    pub(crate) size_contained: bool,
    pub(crate) inline_size_contained: bool,
    pub(crate) style_contained: bool,
    pub(crate) contain_intrinsic_width: Option<String>,
    pub(crate) contain_intrinsic_height: Option<String>,
    pub(crate) overflow_x: Overflow,
    pub(crate) overflow_y: Overflow,
    pub(crate) display_block: Option<bool>,
    pub(crate) display_list_item: Option<bool>,
    pub(crate) display_run_in: bool,
    pub(crate) display_flex: bool,
    pub(crate) flex_direction_column: bool,
    pub(crate) flex_direction_reverse: bool,
    pub(crate) flex_wrap: bool,
    pub(crate) flex_wrap_reverse: bool,
    pub(crate) flex_item_no_wrap: bool,
    pub(crate) flex_order: i32,
    pub(crate) flex_grow: f32,
    pub(crate) flex_shrink: f32,
    pub(crate) flex_basis: Option<FlexBasis>,
    pub(crate) flex_min_main_size: Option<String>,
    pub(crate) flex_max_main_size: Option<String>,
    pub(crate) justify_content: FlexAlignment,
    pub(crate) align_content: Option<FlexAlignment>,
    pub(crate) align_items: Option<FlexAlignment>,
    pub(crate) align_self: Option<FlexAlignment>,
    pub(crate) flex_item_alignment: Option<FlexAlignment>,
    pub(crate) flex_item_cross_alignment: Option<FlexAlignment>,
    pub(crate) flex_item_cross_id: Option<usize>,
    pub(crate) flex_distributed_gap: bool,
    pub(crate) flex_distributed_gap_leading: bool,
    pub(crate) flex_distributed_gap_applied: bool,
    pub(crate) flex_grow_weight: f32,
    pub(crate) flex_shrink_weight: f32,
    pub(crate) flex_justify_distributed: bool,
    pub(crate) flex_wrap_break: bool,
    pub(crate) flex_wrap_leading_spacer: bool,
    pub(crate) flex_wrap_gap_px: f32,
    pub(crate) flex_wrap_gap_applied: bool,
    pub(crate) flex_wrap_cross_size_px: Option<f32>,
    pub(crate) row_gap: Option<String>,
    pub(crate) column_gap: Option<String>,
    pub(crate) row_gap_order: usize,
    pub(crate) column_gap_order: usize,
    pub(crate) flex_items_row: bool,
    pub(crate) flex_items_column: bool,
    pub(crate) list_style_type: Option<String>,
    pub(crate) width: Option<String>,
    pub(crate) min_width: Option<String>,
    pub(crate) max_width: Option<String>,
    pub(crate) inline_size: Option<String>,
    pub(crate) min_inline_size: Option<String>,
    pub(crate) max_inline_size: Option<String>,
    pub(crate) border_box: bool,
    pub(crate) border_image_source: Option<Arc<[u8]>>,
    pub(crate) border_image_slice: [BorderImageSlice; 4],
    pub(crate) border_image_fill: bool,
    pub(crate) border_image_repeat: [BorderImageRepeat; 2],
    pub(crate) border_image_width: [Option<BorderImageWidth>; 4],
    pub(crate) border_image_outset: [Option<BorderImageOutset>; 4],
    pub(crate) border_width: [Option<String>; 4],
    pub(crate) logical_border_width: [Option<String>; 4],
    pub(crate) physical_border_width_orders: [usize; 4],
    pub(crate) logical_border_width_orders: [usize; 4],
    pub(crate) border_style: [BorderLineStyle; 4],
    pub(crate) logical_border_style: [Option<BorderLineStyle>; 4],
    pub(crate) physical_border_style_orders: [usize; 4],
    pub(crate) logical_border_style_orders: [usize; 4],
    pub(crate) border_color: [Option<[u8; 4]>; 4],
    pub(crate) logical_border_color: [Option<[u8; 4]>; 4],
    pub(crate) physical_border_color_orders: [usize; 4],
    pub(crate) logical_border_color_orders: [usize; 4],
    pub(crate) border_radius: [[Option<String>; 2]; 4],
    pub(crate) border_radius_orders: [usize; 4],
    pub(crate) outline_width: Option<String>,
    pub(crate) outline_style: BorderLineStyle,
    pub(crate) outline_color: Option<[u8; 4]>,
    pub(crate) outline_offset: Option<String>,
    pub(crate) aspect_ratio: Option<f32>,
    pub(crate) box_shadows: Vec<BoxShadow>,
    pub(crate) box_decoration_break: BoxDecorationBreak,
    pub(crate) inline_box: bool,
    pub(crate) padding: [Option<String>; 4],
    pub(crate) logical_padding: [Option<String>; 4],
    pub(crate) physical_padding_orders: [usize; 4],
    pub(crate) logical_padding_orders: [usize; 4],
    pub(crate) margin: [Option<String>; 4],
    pub(crate) logical_margin: [Option<String>; 4],
    pub(crate) physical_margin_orders: [usize; 4],
    pub(crate) logical_margin_orders: [usize; 4],
    pub(crate) layout_sizes: Vec<LayoutSize>,
    pub(crate) height: Option<String>,
    pub(crate) min_height: Option<String>,
    pub(crate) max_height: Option<String>,
    pub(crate) block_size: Option<String>,
    pub(crate) min_block_size: Option<String>,
    pub(crate) max_block_size: Option<String>,
    pub(crate) size_property_orders: HashMap<String, usize>,
    pub(crate) current_declaration_order: usize,
    pub(crate) empty_block_height: Option<LayoutHeight>,
    pub(crate) layout_end_sizes: Vec<LayoutSize>,
    pub(crate) flow_spacer: bool,
    pub(crate) padding_start_spacer: bool,
    pub(crate) margin_start_spacer: bool,
    pub(crate) family: String,
    pub(crate) weight: Weight,
    pub(crate) stretch: Stretch,
    pub(crate) kerning: Option<bool>,
    pub(crate) common_ligatures: Option<bool>,
    pub(crate) discretionary_ligatures: Option<bool>,
    pub(crate) historical_ligatures: Option<bool>,
    pub(crate) contextual_alternates: Option<bool>,
    pub(crate) font_style: Style,
    pub(crate) font_oblique_angle: Option<f32>,
    pub(crate) font_variant_caps: FontVariantCaps,
    pub(crate) font_variant_position: FontVariantPosition,
    pub(crate) font_variant_numeric: Vec<String>,
    pub(crate) font_variant_east_asian: Vec<String>,
    pub(crate) font_feature_settings: Vec<([u8; 4], u32)>,
    pub(crate) font_optical_sizing: bool,
    pub(crate) font_variation_settings: Vec<([u8; 4], f32)>,
    pub(crate) cursor: CursorIcon,
    pub(crate) cursor_hidden: bool,
    pub(crate) pointer_events_none: bool,
    pub(crate) user_select: UserSelect,
    pub(crate) user_select_none: bool,
    pub(crate) user_select_all_owner: Option<usize>,
    pub(crate) letter_spacing: f32,
    pub(crate) letter_spacing_px: bool,
    pub(crate) word_spacing_px: f32,
    pub(crate) word_spacing_percent: Option<f32>,
    pub(crate) word_spacing_space: bool,
    pub(crate) text_transform: TextTransform,
    pub(crate) text_transform_full_width: bool,
    pub(crate) text_transform_full_size_kana: bool,
    pub(crate) first_line_letter_spacing_adjustment: f32,
    pub(crate) text_align: Align,
    pub(crate) text_align_last: Option<Align>,
    pub(crate) text_justify: cosmic_text::TextJustify,
    pub(crate) white_space: WhiteSpace,
    pub(crate) text_wrap_mode: TextWrapMode,
    pub(crate) overflow_wrap: OverflowWrap,
    pub(crate) word_break: WordBreak,
    pub(crate) line_break: LineBreak,
    pub(crate) hyphens: Hyphens,
    pub(crate) language: Option<String>,
    pub(crate) text_overflow: TextOverflow,
    pub(crate) line_clamp: Option<usize>,
    pub(crate) text_rendering: TextRendering,
    pub(crate) text_orientation: TextOrientation,
    pub(crate) writing_mode: WritingMode,
    pub(crate) direction: Direction,
    pub(crate) unicode_bidi: UnicodeBidi,
    pub(crate) bidi_isolate_direction: Option<Direction>,
    pub(crate) quotes: Vec<(String, String)>,
    pub(crate) counter_reset: Vec<(String, i32)>,
    pub(crate) counter_increment: Vec<(String, i32)>,
    pub(crate) counter_set: Vec<(String, i32)>,
    pub(crate) text_indent: f32,
    pub(crate) text_indent_percent: f32,
    pub(crate) text_indent_hanging: bool,
    pub(crate) text_indent_each_line: bool,
    pub(crate) underline: bool,
    pub(crate) decoration_style: TextDecorationStyle,
    pub(crate) decoration_skip_ink: TextDecorationSkipInk,
    pub(crate) decoration_thickness: Option<f32>,
    pub(crate) underline_offset: Option<f32>,
    pub(crate) overline: bool,
    pub(crate) strikethrough: bool,
    pub(crate) text_decoration_color: Option<[u8; 4]>,
    pub(crate) text_shadows: Vec<TextShadow>,
    pub(crate) text_emphasis_style: TextEmphasisStyle,
    pub(crate) text_emphasis_color: Option<[u8; 4]>,
    pub(crate) text_emphasis_position: TextEmphasisPosition,
    pub(crate) text_emphasis_side: Option<TextEmphasisSide>,
    pub(crate) custom_properties: HashMap<String, String>,
}

impl TextStyle {
    pub(crate) fn animation_specs(&self) -> Vec<Option<AnimationSpec>> {
        let count = self.animation_name.len();
        (0..count)
            .map(|index| {
                self.animation_name[index]
                    .clone()
                    .map(|name| AnimationSpec {
                        name,
                        duration_ms: cycle_value(&self.animation_duration_ms, index, 0.0),
                        delay_ms: cycle_value(&self.animation_delay_ms, index, 0.0),
                        iterations: cycle_value(&self.animation_iterations, index, 1.0),
                        direction: cycle_value(
                            &self.animation_direction,
                            index,
                            AnimationDirection::Normal,
                        ),
                        fill_mode: cycle_value(
                            &self.animation_fill_mode,
                            index,
                            AnimationFillMode::None,
                        ),
                        play_state: cycle_value(
                            &self.animation_play_state,
                            index,
                            AnimationPlayState::Running,
                        ),
                        easing: cycle_value(&self.animation_easing, index, TransitionEasing::Ease),
                    })
            })
            .collect()
    }

    pub(crate) fn parse_color(&self, value: &str) -> Option<([u8; 3], f32)> {
        ctml_css::parse_color_with_opacity_and_profiles_for_scheme(
            value,
            &self.color_profiles,
            self.dark_color_scheme,
        )
    }

    pub(crate) fn parse_color_rgba(&self, value: &str) -> Option<[u8; 4]> {
        let (channels, alpha) = self.parse_color(value)?;
        Some([
            channels[0],
            channels[1],
            channels[2],
            (alpha * 255.0).round() as u8,
        ])
    }

    pub(crate) fn record_size_property_order(&mut self, property: &str) {
        self.size_property_orders
            .insert(property.to_owned(), self.current_declaration_order);
    }
}

fn cycle_value<T: Copy>(values: &[T], index: usize, default: T) -> T {
    values
        .get(index % values.len().max(1))
        .copied()
        .unwrap_or(default)
}
