use super::super::*;

pub(in crate::css_style::properties) fn initial_property(style: &mut TextStyle, property: &str) {
    match property {
        "color-scheme" => style.dark_color_scheme = false,
        "opacity" => style.opacity = 1.0,
        "transition"
        | "transition-property"
        | "transition-duration"
        | "transition-delay"
        | "transition-timing-function" => {
            super::super::transition::reset_transition(style, property)
        }
        "visibility" => style.visibility_hidden = false,
        "content-visibility" => {
            style.content_visibility_hidden = false;
            style.content_visibility_auto = false;
        }
        "contain" => {
            style.paint_contained = false;
            style.layout_contained = false;
            style.size_contained = false;
            style.inline_size_contained = false;
            style.style_contained = false;
        }
        "contain-intrinsic-size" => {
            style.contain_intrinsic_width = None;
            style.contain_intrinsic_height = None;
        }
        "contain-intrinsic-width" => style.contain_intrinsic_width = None,
        "contain-intrinsic-height" => style.contain_intrinsic_height = None,
        "overflow" => {
            style.overflow_x = Overflow::Visible;
            style.overflow_y = Overflow::Visible;
        }
        "overflow-x" => style.overflow_x = Overflow::Visible,
        "overflow-y" => style.overflow_y = Overflow::Visible,
        "pointer-events" => style.pointer_events_none = false,
        "user-select" => {
            style.user_select = UserSelect::Auto;
            style.user_select_none = false;
            style.user_select_all_owner = None;
        }
        "background-color" => {
            style.background = [255, 255, 255];
            style.background_color = None;
        }
        "background-image" => style.background_images.clear(),
        "background-blend-mode" => style.background_blend_modes.clear(),
        "mix-blend-mode" => {
            style.mix_blend_mode = crate::style::BackgroundBlendMode::Normal;
        }
        "isolation" => style.isolation = false,
        "border-image-source" => style.border_image_source = None,
        "border-image" => {
            style.border_image_source = None;
            style.border_image_slice = [crate::style::BorderImageSlice::default(); 4];
            style.border_image_fill = false;
            style.border_image_repeat = [crate::style::BorderImageRepeat::Stretch; 2];
            style.border_image_width = [None; 4];
            style.border_image_outset = [None; 4];
        }
        "border-image-slice" => {
            style.border_image_slice = [crate::style::BorderImageSlice::default(); 4];
            style.border_image_fill = false;
        }
        "border-image-repeat" => {
            style.border_image_repeat = [crate::style::BorderImageRepeat::Stretch; 2];
        }
        "border-image-width" => style.border_image_width = [None; 4],
        "border-image-outset" => style.border_image_outset = [None; 4],
        "background-repeat" => style.background_repeats.clear(),
        "background-position" => style.background_positions.clear(),
        "background-size" => style.background_sizes.clear(),
        "background-attachment" => style.background_attachments.clear(),
        "background-origin" => style.background_origins.clear(),
        "background-clip" => style.background_clips.clear(),
        "display" => reset_display(style),
        "flex-direction" => {
            style.flex_direction_column = false;
            style.flex_direction_reverse = false;
        }
        "flex-wrap" => style.flex_wrap = false,
        "flex-flow" => {
            style.flex_direction_column = false;
            style.flex_direction_reverse = false;
            style.flex_wrap = false;
        }
        "justify-content" => {
            style.justify_content = crate::style::FlexAlignment::default();
        }
        "align-items" => style.align_items = None,
        "align-self" => style.align_self = None,
        "order" => style.flex_order = 0,
        "flex-grow" => style.flex_grow = 0.0,
        "flex-shrink" => style.flex_shrink = 1.0,
        "flex-basis" => style.flex_basis = None,
        "gap" | "row-gap" | "column-gap" => {
            super::super::flex_gap::initial_flex_gap(style, property);
        }
        "width" => style.width = None,
        "inline-size" => style.inline_size = None,
        "box-sizing" => style.border_box = false,
        "padding" => {
            style.padding = std::array::from_fn(|_| Some("0".to_owned()));
            style.physical_padding_orders = [style.current_declaration_order.saturating_add(1); 4];
        }
        "padding-top" | "padding-right" | "padding-bottom" | "padding-left" => {
            let side = match property {
                "padding-top" => 0,
                "padding-right" => 1,
                "padding-bottom" => 2,
                _ => 3,
            };
            style.padding[side] = Some("0".to_owned());
            style.physical_padding_orders[side] = style.current_declaration_order.saturating_add(1);
        }
        "padding-inline" => {
            style.logical_padding[0] = Some("0".to_owned());
            style.logical_padding[1] = Some("0".to_owned());
            style.logical_padding_orders[0] = style.current_declaration_order.saturating_add(1);
            style.logical_padding_orders[1] = style.current_declaration_order.saturating_add(1);
        }
        "padding-inline-start"
        | "padding-inline-end"
        | "padding-block-start"
        | "padding-block-end" => {
            let side = match property {
                "padding-inline-start" => 0,
                "padding-inline-end" => 1,
                "padding-block-start" => 2,
                _ => 3,
            };
            style.logical_padding[side] = Some("0".to_owned());
            style.logical_padding_orders[side] = style.current_declaration_order.saturating_add(1);
        }
        "padding-block" => {
            style.logical_padding[2] = Some("0".to_owned());
            style.logical_padding[3] = Some("0".to_owned());
            style.logical_padding_orders[2] = style.current_declaration_order.saturating_add(1);
            style.logical_padding_orders[3] = style.current_declaration_order.saturating_add(1);
        }
        "min-width" => style.min_width = None,
        "min-inline-size" => style.min_inline_size = None,
        "max-width" => style.max_width = None,
        "max-inline-size" => style.max_inline_size = None,
        "height" => style.height = None,
        "block-size" => style.block_size = None,
        "min-height" => style.min_height = None,
        "min-block-size" => style.min_block_size = None,
        "max-height" => style.max_height = None,
        "max-block-size" => style.max_block_size = None,
        property if is_margin_property(property) => reset_margin_property(style, property),
        "list-style" | "list-style-type" => reset_list_style_type(style),
        "background" => {
            style.background = [255, 255, 255];
            style.background_color = None;
            style.background_images.clear();
            style.background_repeats.clear();
            style.background_positions.clear();
            style.background_sizes.clear();
            style.background_attachments.clear();
            reset_background_box_properties(style);
        }
        "font" => reset_font(style),
        "font-family" => style.family = "serif".to_owned(),
        "font-size" => {
            style.font_size = 16.0;
            if let Some(multiplier) = style.line_height_multiplier {
                style.line_height = style.font_size * multiplier;
            }
        }
        "font-weight" => style.weight = Weight::NORMAL,
        "font-style" => {
            style.font_style = Style::Normal;
            style.font_oblique_angle = None;
        }
        "font-stretch" => style.stretch = Stretch::Normal,
        "font-kerning" => style.kerning = None,
        "font-variant" => {
            style.font_variant_caps = crate::style::FontVariantCaps::Normal;
            style.font_variant_position = FontVariantPosition::Normal;
            style.common_ligatures = None;
            style.discretionary_ligatures = None;
            style.historical_ligatures = None;
            style.contextual_alternates = None;
            style.font_variant_numeric.clear();
            style.font_variant_east_asian.clear();
        }
        "font-variant-caps" => style.font_variant_caps = crate::style::FontVariantCaps::Normal,
        "font-variant-ligatures" => {
            style.common_ligatures = None;
            style.discretionary_ligatures = None;
            style.historical_ligatures = None;
            style.contextual_alternates = None;
        }
        "font-variant-position" => style.font_variant_position = FontVariantPosition::Normal,
        "font-variant-numeric" => style.font_variant_numeric.clear(),
        "font-variant-east-asian" => style.font_variant_east_asian.clear(),
        "font-feature-settings" => style.font_feature_settings.clear(),
        "font-optical-sizing" => style.font_optical_sizing = true,
        "font-variation-settings" => style.font_variation_settings.clear(),
        "text-shadow" => style.text_shadows.clear(),
        "text-emphasis"
        | "text-emphasis-style"
        | "text-emphasis-color"
        | "text-emphasis-position" => reset_text_emphasis(style, property),
        "line-height" => {
            style.line_height_normal = true;
            style.line_height_multiplier = None;
        }
        "vertical-align" => style.vertical_align_offset = 0.0,
        "letter-spacing" => {
            style.letter_spacing = 0.0;
            style.letter_spacing_px = false;
        }
        "word-spacing" => {
            style.word_spacing_px = 0.0;
            style.word_spacing_percent = None;
        }
        "text-transform" => {
            style.text_transform = TextTransform::None;
            style.text_transform_full_width = false;
            style.text_transform_full_size_kana = false;
        }
        "text-align" => style.text_align = cosmic_text::Align::Left,
        "text-align-last" => style.text_align_last = None,
        "text-justify" => style.text_justify = cosmic_text::TextJustify::Auto,
        "white-space" => {
            style.white_space = WhiteSpace::Normal;
            style.text_wrap_mode = TextWrapMode::Wrap;
        }
        "text-wrap" | "text-wrap-mode" => style.text_wrap_mode = TextWrapMode::Wrap,
        "overflow-wrap" => style.overflow_wrap = OverflowWrap::Normal,
        "word-break" => style.word_break = WordBreak::Normal,
        "line-break" => style.line_break = LineBreak::Auto,
        "hyphens" => style.hyphens = Hyphens::Manual,
        "text-rendering" => style.text_rendering = TextRendering::Auto,
        "text-orientation" => style.text_orientation = TextOrientation::Mixed,
        "writing-mode" => style.writing_mode = WritingMode::HorizontalTb,
        "direction" => style.direction = Direction::Ltr,
        "unicode-bidi" => style.unicode_bidi = UnicodeBidi::Normal,
        "quotes" => style.quotes = quotes::default_quotes(),
        "counter-reset" => style.counter_reset.clear(),
        "counter-increment" => style.counter_increment.clear(),
        "counter-set" => style.counter_set.clear(),
        "text-overflow" => style.text_overflow = TextOverflow::Clip,
        "line-clamp" => style.line_clamp = None,
        "text-indent" => {
            style.text_indent = 0.0;
            style.text_indent_percent = 0.0;
            style.text_indent_hanging = false;
            style.text_indent_each_line = false;
        }
        "text-decoration" | "text-decoration-line" => {
            if property == "text-decoration" {
                reset_text_decoration(style);
            } else {
                apply_text_decoration_line(style, "none");
            }
        }
        "text-decoration-style" => {
            style.decoration_style = crate::style::TextDecorationStyle::Solid
        }
        "text-decoration-color" => style.text_decoration_color = None,
        "text-decoration-skip-ink" => {
            style.decoration_skip_ink = crate::style::TextDecorationSkipInk::Auto;
        }
        "text-decoration-thickness" => style.decoration_thickness = None,
        "text-underline-offset" => style.underline_offset = None,
        "cursor" => {
            style.cursor = CursorIcon::Text;
            style.cursor_hidden = false;
        }
        _ => {
            if !super::super::border_cascade::initial_css_property(style, property) {
                if !super::super::border_radius::initial_css_property(style, property) {
                    if !super::super::outline::initial_css_property(style, property) {
                        if !super::super::aspect_ratio::initial_css_property(style, property) {
                            if !super::super::box_shadow::initial_css_property(style, property) {
                                super::super::box_decoration::initial_css_property(style, property);
                            }
                        }
                    }
                }
            }
        }
    }
}
