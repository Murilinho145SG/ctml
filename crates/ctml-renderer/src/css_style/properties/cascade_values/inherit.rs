use super::super::*;

pub(in crate::css_style::properties) fn inherit_property(
    style: &mut TextStyle,
    inherited: &TextStyle,
    property: &str,
) {
    match property {
        "color-scheme" => style.dark_color_scheme = inherited.dark_color_scheme,
        "opacity" => style.opacity = inherited.opacity,
        "visibility" => style.visibility_hidden = inherited.visibility_hidden,
        "content-visibility" => {
            style.content_visibility_hidden = inherited.content_visibility_hidden;
            style.content_visibility_auto = inherited.content_visibility_auto;
        }
        "contain" => {
            style.paint_contained = inherited.paint_contained;
            style.layout_contained = inherited.layout_contained;
            style.size_contained = inherited.size_contained;
            style.inline_size_contained = inherited.inline_size_contained;
            style.style_contained = inherited.style_contained;
        }
        "contain-intrinsic-size" => {
            style.contain_intrinsic_width = inherited.contain_intrinsic_width.clone();
            style.contain_intrinsic_height = inherited.contain_intrinsic_height.clone();
        }
        "contain-intrinsic-width" => {
            style
                .contain_intrinsic_width
                .clone_from(&inherited.contain_intrinsic_width);
        }
        "contain-intrinsic-height" => {
            style
                .contain_intrinsic_height
                .clone_from(&inherited.contain_intrinsic_height);
        }
        "overflow" => {
            style.overflow_x = inherited.overflow_x;
            style.overflow_y = inherited.overflow_y;
        }
        "overflow-x" => style.overflow_x = inherited.overflow_x,
        "overflow-y" => style.overflow_y = inherited.overflow_y,
        "pointer-events" => style.pointer_events_none = inherited.pointer_events_none,
        "user-select" => {
            style.user_select = inherited.user_select;
            style.user_select_none = inherited.user_select_none;
            style.user_select_all_owner = inherited.user_select_all_owner;
        }
        "background-color" => {
            style.background = inherited.background;
            style.background_color = inherited.background_color;
        }
        "background-image" => {
            style
                .background_images
                .clone_from(&inherited.background_images);
        }
        "background-blend-mode" => style
            .background_blend_modes
            .clone_from(&inherited.background_blend_modes),
        "mix-blend-mode" => style.mix_blend_mode = inherited.mix_blend_mode,
        "isolation" => style.isolation = inherited.isolation,
        "border-image-source" => {
            style
                .border_image_source
                .clone_from(&inherited.border_image_source);
        }
        "border-image" => {
            style
                .border_image_source
                .clone_from(&inherited.border_image_source);
            style.border_image_slice = inherited.border_image_slice;
            style.border_image_fill = inherited.border_image_fill;
            style.border_image_repeat = inherited.border_image_repeat;
            style.border_image_width = inherited.border_image_width;
            style.border_image_outset = inherited.border_image_outset;
        }
        "border-image-slice" => {
            style.border_image_slice = inherited.border_image_slice;
            style.border_image_fill = inherited.border_image_fill;
        }
        "border-image-repeat" => style.border_image_repeat = inherited.border_image_repeat,
        "border-image-width" => style.border_image_width = inherited.border_image_width,
        "border-image-outset" => style.border_image_outset = inherited.border_image_outset,
        "background-repeat" => {
            style
                .background_repeats
                .clone_from(&inherited.background_repeats);
        }
        "background-position" => {
            style
                .background_positions
                .clone_from(&inherited.background_positions);
        }
        "background-size" => {
            style
                .background_sizes
                .clone_from(&inherited.background_sizes);
        }
        "background-attachment" => {
            style
                .background_attachments
                .clone_from(&inherited.background_attachments);
        }
        "background-origin" => {
            style
                .background_origins
                .clone_from(&inherited.background_origins);
        }
        "background-clip" => {
            style
                .background_clips
                .clone_from(&inherited.background_clips);
        }
        "display" => inherit_display(style, inherited),
        "flex-direction" => {
            style.flex_direction_column = inherited.flex_direction_column;
        }
        "justify-content" => style.justify_content = inherited.justify_content,
        "align-items" => style.align_items = inherited.align_items,
        "align-self" => style.align_self = inherited.align_self,
        "order" => style.flex_order = inherited.flex_order,
        "gap" | "row-gap" | "column-gap" => {
            super::super::flex_gap::inherit_flex_gap(style, inherited, property);
        }
        "width" => inherit_width(style, inherited),
        "inline-size" => inherit_inline_size(style, inherited),
        "box-sizing" => style.border_box = inherited.border_box,
        "padding" => {
            style.padding.clone_from(&inherited.padding);
            style.physical_padding_orders = [style.current_declaration_order.saturating_add(1); 4];
        }
        "padding-top" => {
            style.padding[0].clone_from(&inherited.padding[0]);
            style.physical_padding_orders[0] = style.current_declaration_order.saturating_add(1);
        }
        "padding-right" => {
            style.padding[1].clone_from(&inherited.padding[1]);
            style.physical_padding_orders[1] = style.current_declaration_order.saturating_add(1);
        }
        "padding-bottom" => {
            style.padding[2].clone_from(&inherited.padding[2]);
            style.physical_padding_orders[2] = style.current_declaration_order.saturating_add(1);
        }
        "padding-left" => {
            style.padding[3].clone_from(&inherited.padding[3]);
            style.physical_padding_orders[3] = style.current_declaration_order.saturating_add(1);
        }
        "padding-inline" | "padding-block" => {
            let start = if property == "padding-inline" { 0 } else { 2 };
            style.logical_padding[start].clone_from(&inherited.logical_padding[start]);
            style.logical_padding[start + 1].clone_from(&inherited.logical_padding[start + 1]);
            style.logical_padding_orders[start] = style.current_declaration_order.saturating_add(1);
            style.logical_padding_orders[start + 1] =
                style.current_declaration_order.saturating_add(1);
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
            style.logical_padding[side].clone_from(&inherited.logical_padding[side]);
            style.logical_padding_orders[side] = style.current_declaration_order.saturating_add(1);
        }
        "min-width" => style.min_width.clone_from(&inherited.min_width),
        "min-inline-size" => style.min_inline_size.clone_from(&inherited.min_inline_size),
        "max-width" => style.max_width.clone_from(&inherited.max_width),
        "max-inline-size" => style.max_inline_size.clone_from(&inherited.max_inline_size),
        "height" => style.height.clone_from(&inherited.height),
        "block-size" => style.block_size.clone_from(&inherited.block_size),
        "min-height" => style.min_height.clone_from(&inherited.min_height),
        "min-block-size" => style.min_block_size.clone_from(&inherited.min_block_size),
        "max-height" => style.max_height.clone_from(&inherited.max_height),
        "max-block-size" => style.max_block_size.clone_from(&inherited.max_block_size),
        property if is_margin_property(property) => {
            inherit_margin_property(style, inherited, property)
        }
        "list-style" | "list-style-type" => {
            style.list_style_type.clone_from(&inherited.list_style_type);
        }
        "background" => {
            style.background = inherited.background;
            style.background_color = inherited.background_color;
        }
        "font" => inherit_font(style, inherited),
        "font-family" => style.family.clone_from(&inherited.family),
        "font-size" => {
            style.font_size = inherited.font_size;
            style.font_scale = inherited.font_scale;
            style.line_height = inherited.line_height;
            style.line_height_multiplier = inherited.line_height_multiplier;
            style.line_height_normal = inherited.line_height_normal;
        }
        "font-weight" => style.weight = inherited.weight,
        "font-style" => {
            style.font_style = inherited.font_style;
            style.font_oblique_angle = inherited.font_oblique_angle;
        }
        "font-stretch" => style.stretch = inherited.stretch,
        "font-kerning" => style.kerning = inherited.kerning,
        "font-variant" => {
            style.font_variant_caps = inherited.font_variant_caps;
            style.font_variant_position = inherited.font_variant_position;
            style.common_ligatures = inherited.common_ligatures;
            style.discretionary_ligatures = inherited.discretionary_ligatures;
            style.historical_ligatures = inherited.historical_ligatures;
            style.contextual_alternates = inherited.contextual_alternates;
            style
                .font_variant_numeric
                .clone_from(&inherited.font_variant_numeric);
            style
                .font_variant_east_asian
                .clone_from(&inherited.font_variant_east_asian);
        }
        "font-variant-caps" => style.font_variant_caps = inherited.font_variant_caps,
        "font-variant-ligatures" => {
            style.common_ligatures = inherited.common_ligatures;
            style.discretionary_ligatures = inherited.discretionary_ligatures;
            style.historical_ligatures = inherited.historical_ligatures;
            style.contextual_alternates = inherited.contextual_alternates;
        }
        "font-variant-position" => style.font_variant_position = inherited.font_variant_position,
        "font-variant-numeric" => style
            .font_variant_numeric
            .clone_from(&inherited.font_variant_numeric),
        "font-variant-east-asian" => style
            .font_variant_east_asian
            .clone_from(&inherited.font_variant_east_asian),
        "font-feature-settings" => style
            .font_feature_settings
            .clone_from(&inherited.font_feature_settings),
        "font-optical-sizing" => style.font_optical_sizing = inherited.font_optical_sizing,
        "font-variation-settings" => style
            .font_variation_settings
            .clone_from(&inherited.font_variation_settings),
        "text-shadow" => style.text_shadows.clone_from(&inherited.text_shadows),
        "text-emphasis"
        | "text-emphasis-style"
        | "text-emphasis-color"
        | "text-emphasis-position" => inherit_text_emphasis(style, inherited, property),
        "line-height" => {
            style.line_height = inherited.line_height;
            style.line_height_multiplier = inherited.line_height_multiplier;
            style.line_height_normal = inherited.line_height_normal;
        }
        "vertical-align" => style.vertical_align_offset = inherited.vertical_align_offset,
        "letter-spacing" => {
            style.letter_spacing = inherited.letter_spacing;
            style.letter_spacing_px = inherited.letter_spacing_px;
        }
        "word-spacing" => {
            style.word_spacing_px = inherited.word_spacing_px;
            style.word_spacing_percent = inherited.word_spacing_percent;
        }
        "text-transform" => {
            style.text_transform = inherited.text_transform;
            style.text_transform_full_width = inherited.text_transform_full_width;
            style.text_transform_full_size_kana = inherited.text_transform_full_size_kana;
        }
        "text-align" => style.text_align = inherited.text_align,
        "text-align-last" => style.text_align_last = inherited.text_align_last,
        "text-justify" => style.text_justify = inherited.text_justify,
        "white-space" => {
            style.white_space = inherited.white_space;
            style.text_wrap_mode = inherited.text_wrap_mode;
        }
        "text-wrap" | "text-wrap-mode" => style.text_wrap_mode = inherited.text_wrap_mode,
        "overflow-wrap" => style.overflow_wrap = inherited.overflow_wrap,
        "word-break" => style.word_break = inherited.word_break,
        "line-break" => style.line_break = inherited.line_break,
        "hyphens" => style.hyphens = inherited.hyphens,
        "text-rendering" => style.text_rendering = inherited.text_rendering,
        "text-orientation" => style.text_orientation = inherited.text_orientation,
        "writing-mode" => style.writing_mode = inherited.writing_mode,
        "direction" => style.direction = inherited.direction,
        "unicode-bidi" => style.unicode_bidi = inherited.unicode_bidi,
        "quotes" => style.quotes = inherited.quotes.clone(),
        "text-indent" => {
            style.text_indent = inherited.text_indent;
            style.text_indent_percent = inherited.text_indent_percent;
            style.text_indent_hanging = inherited.text_indent_hanging;
            style.text_indent_each_line = inherited.text_indent_each_line;
        }
        "text-decoration" => {
            inherit_text_decoration(style, inherited);
        }
        "text-decoration-line" => copy_text_decoration_lines(style, inherited),
        "text-decoration-color" => style.text_decoration_color = inherited.text_decoration_color,
        "text-decoration-skip-ink" => style.decoration_skip_ink = inherited.decoration_skip_ink,
        "text-decoration-thickness" => style.decoration_thickness = inherited.decoration_thickness,
        "text-underline-offset" => style.underline_offset = inherited.underline_offset,
        "cursor" => {
            style.cursor = inherited.cursor;
            style.cursor_hidden = inherited.cursor_hidden;
        }
        _ => {
            if !super::super::border_cascade::inherit_css_property(style, inherited, property) {
                if !super::super::border_radius::inherit_css_property(style, inherited, property) {
                    if !super::super::outline::inherit_css_property(style, inherited, property) {
                        if !super::super::aspect_ratio::inherit_css_property(
                            style, inherited, property,
                        ) {
                            if !super::super::box_shadow::inherit_css_property(
                                style, inherited, property,
                            ) {
                                super::super::box_decoration::inherit_css_property(
                                    style, inherited, property,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
