use super::*;

pub(super) fn apply_declaration(
    style: &mut TextStyle,
    property: &str,
    value: &str,
    parent_line_height: f32,
    parent_font_units: ctml_css::FontRelativeUnits,
    parent_font_size: f32,
) {
    let value = value.trim();
    match property {
        "opacity" => {
            let opacity = ctml_css::parse_math_number(value)
                .or_else(|| ctml_css::parse_css_number(value))
                .or_else(|| ctml_css::parse_css_percentage(value).map(|value| value / 100.0));
            if let Some(opacity) = opacity.filter(|opacity| opacity.is_finite()) {
                style.opacity = opacity.clamp(0.0, 1.0);
            }
        }
        "visibility" => match value.to_ascii_lowercase().as_str() {
            "visible" => style.visibility_hidden = false,
            "hidden" | "collapse" => style.visibility_hidden = true,
            _ => {}
        },
        "content-visibility" => match value.to_ascii_lowercase().as_str() {
            "hidden" => {
                style.content_visibility_hidden = true;
                style.content_visibility_auto = false;
            }
            "auto" => {
                style.content_visibility_hidden = false;
                style.content_visibility_auto = true;
            }
            "visible" => {
                style.content_visibility_hidden = false;
                style.content_visibility_auto = false;
            }
            _ => {}
        },
        "contain" => apply_contain(style, value),
        "contain-intrinsic-size" => apply_contain_intrinsic_size(style, value),
        "contain-intrinsic-width" => apply_contain_intrinsic_width(style, value),
        "contain-intrinsic-height" => apply_contain_intrinsic_height(style, value),
        "overflow" => {
            if let Some((overflow_x, overflow_y)) = parse_overflow_value(value) {
                style.overflow_x = overflow_x;
                style.overflow_y = overflow_y;
            }
        }
        "overflow-x" => {
            if let Some(overflow) = parse_overflow_keyword(value) {
                style.overflow_x = overflow;
            }
        }
        "overflow-y" => {
            if let Some(overflow) = parse_overflow_keyword(value) {
                style.overflow_y = overflow;
            }
        }
        "pointer-events" => match value.to_ascii_lowercase().as_str() {
            "auto" => style.pointer_events_none = false,
            "none" => style.pointer_events_none = true,
            _ => {}
        },
        "user-select" => match value.to_ascii_lowercase().as_str() {
            "auto" => style.user_select = UserSelect::Auto,
            "text" => {
                style.user_select = UserSelect::Text;
                style.user_select_none = false;
            }
            "none" => {
                style.user_select = UserSelect::None;
                style.user_select_none = true;
            }
            "all" => {
                style.user_select = UserSelect::All;
                style.user_select_none = false;
            }
            _ => {}
        },
        "font" => apply_font_shorthand(style, value, parent_line_height, parent_font_units),
        "display" => apply_display(style, value),
        "width" => apply_width(style, value),
        "inline-size" => apply_inline_size(style, value),
        "box-sizing" => apply_box_sizing(style, value),
        "border-radius" => apply_border_radius(style, value),
        "border-top-left-radius" => apply_corner_radius(style, 0, value),
        "border-top-right-radius" => apply_corner_radius(style, 1, value),
        "border-bottom-right-radius" => apply_corner_radius(style, 2, value),
        "border-bottom-left-radius" => apply_corner_radius(style, 3, value),
        "outline" => apply_outline(style, value),
        "outline-width" => apply_outline_width(style, value),
        "outline-style" => apply_outline_style(style, value),
        "outline-color" => apply_outline_color(style, value),
        "outline-offset" => apply_outline_offset(style, value),
        "aspect-ratio" => apply_aspect_ratio(style, value),
        "box-shadow" => apply_box_shadow(style, value),
        "box-decoration-break" => apply_box_decoration_break(style, value),
        "border" => apply_border_shorthand(style, value),
        "border-top" => apply_border_side(style, 0, value),
        "border-right" => apply_border_side(style, 1, value),
        "border-bottom" => apply_border_side(style, 2, value),
        "border-left" => apply_border_side(style, 3, value),
        "border-inline" => apply_logical_border_axis_shorthand(style, true, value),
        "border-block" => apply_logical_border_axis_shorthand(style, false, value),
        "border-inline-start" => apply_logical_border_shorthand(style, 0, value),
        "border-inline-end" => apply_logical_border_shorthand(style, 1, value),
        "border-block-start" => apply_logical_border_shorthand(style, 2, value),
        "border-block-end" => apply_logical_border_shorthand(style, 3, value),
        "border-width" => apply_border_width(style, value),
        "border-top-width" => apply_border_width_side(style, 0, value),
        "border-right-width" => apply_border_width_side(style, 1, value),
        "border-bottom-width" => apply_border_width_side(style, 2, value),
        "border-left-width" => apply_border_width_side(style, 3, value),
        "border-inline-width" => apply_logical_border_width_shorthand(style, true, value),
        "border-inline-start-width" => apply_logical_border_width(style, 0, value),
        "border-inline-end-width" => apply_logical_border_width(style, 1, value),
        "border-block-width" => apply_logical_border_width_shorthand(style, false, value),
        "border-block-start-width" => apply_logical_border_width(style, 2, value),
        "border-block-end-width" => apply_logical_border_width(style, 3, value),
        "border-style" => apply_border_style(style, value),
        "border-top-style" => apply_border_style_side(style, 0, value),
        "border-right-style" => apply_border_style_side(style, 1, value),
        "border-bottom-style" => apply_border_style_side(style, 2, value),
        "border-left-style" => apply_border_style_side(style, 3, value),
        "border-inline-style" => apply_logical_border_style_shorthand(style, true, value),
        "border-inline-start-style" => apply_logical_border_style(style, 0, value),
        "border-inline-end-style" => apply_logical_border_style(style, 1, value),
        "border-block-style" => apply_logical_border_style_shorthand(style, false, value),
        "border-block-start-style" => apply_logical_border_style(style, 2, value),
        "border-block-end-style" => apply_logical_border_style(style, 3, value),
        "border-color" => apply_border_color(style, value),
        "border-top-color" => apply_border_color_side(style, 0, value),
        "border-right-color" => apply_border_color_side(style, 1, value),
        "border-bottom-color" => apply_border_color_side(style, 2, value),
        "border-left-color" => apply_border_color_side(style, 3, value),
        "border-inline-color" => apply_logical_border_color_shorthand(style, true, value),
        "border-inline-start-color" => apply_logical_border_color(style, 0, value),
        "border-inline-end-color" => apply_logical_border_color(style, 1, value),
        "border-block-color" => apply_logical_border_color_shorthand(style, false, value),
        "border-block-start-color" => apply_logical_border_color(style, 2, value),
        "border-block-end-color" => apply_logical_border_color(style, 3, value),
        "padding" => apply_padding(style, value),
        "padding-top" => apply_padding_side(style, 0, value),
        "padding-right" => apply_padding_side(style, 1, value),
        "padding-bottom" => apply_padding_side(style, 2, value),
        "padding-left" => apply_padding_side(style, 3, value),
        "padding-inline" => apply_logical_padding_shorthand(style, true, value),
        "padding-inline-start" => apply_logical_padding(style, 0, value),
        "padding-inline-end" => apply_logical_padding(style, 1, value),
        "padding-block" => apply_logical_padding_shorthand(style, false, value),
        "padding-block-start" => apply_logical_padding(style, 2, value),
        "padding-block-end" => apply_logical_padding(style, 3, value),
        "margin" => apply_margin(style, value),
        "margin-top" => apply_margin_side(style, 0, value),
        "margin-right" => apply_margin_side(style, 1, value),
        "margin-bottom" => apply_margin_side(style, 2, value),
        "margin-left" => apply_margin_side(style, 3, value),
        "margin-inline" => apply_logical_margin_shorthand(style, true, value),
        "margin-inline-start" => apply_logical_margin(style, 0, value),
        "margin-inline-end" => apply_logical_margin(style, 1, value),
        "margin-block" => apply_logical_margin_shorthand(style, false, value),
        "margin-block-start" => apply_logical_margin(style, 2, value),
        "margin-block-end" => apply_logical_margin(style, 3, value),
        "min-width" => apply_min_width(style, value),
        "min-inline-size" => apply_min_inline_size(style, value),
        "max-width" => apply_max_width(style, value),
        "max-inline-size" => apply_max_inline_size(style, value),
        "height" => apply_height(style, value),
        "block-size" => apply_block_size(style, value),
        "min-height" => apply_min_height(style, value),
        "min-block-size" => apply_min_block_size(style, value),
        "max-height" => apply_max_height(style, value),
        "max-block-size" => apply_max_block_size(style, value),
        "list-style-type" => apply_list_style_type(style, value),
        "list-style" => apply_list_style_shorthand(style, value),
        "background" => apply_background_shorthand(style, value),
        "background-color" => apply_background_color(style, value),
        "background-image" => apply_background_image(style, value),
        "font-size" => {
            if let Some(size) = parse_font_size_with_font_units(
                value,
                style.font_size,
                parent_line_height,
                parent_font_units,
                style.viewport_size,
            ) {
                style.font_size = size;
                if let Some(multiplier) = style.line_height_multiplier {
                    style.line_height = size * multiplier;
                }
            }
        }
        "line-height" => {
            if value.eq_ignore_ascii_case("normal") {
                style.line_height_normal = true;
                style.line_height_multiplier = None;
            } else if let Some(multiplier) =
                ctml_css::parse_math_number(value).or_else(|| ctml_css::parse_css_number(value))
            {
                style.line_height = style.font_size * multiplier;
                style.line_height_multiplier = Some(multiplier);
                style.line_height_normal = false;
            } else if let Some(height) = parse_nonnegative_font_relative_length_with_viewport(
                value,
                style.font_size,
                parent_line_height,
                style.font_relative_units,
                style.viewport_size,
            ) {
                style.line_height = height;
                style.line_height_multiplier = None;
                style.line_height_normal = false;
            }
        }
        "vertical-align" => {
            style.vertical_align_offset = match value.to_ascii_lowercase().as_str() {
                "baseline" => 0.0,
                "sub" => -parent_font_size * 0.2,
                "super" => parent_font_size / 3.0,
                "middle" => parent_font_units.ex * parent_font_size / 2.0,
                "text-top" => {
                    parent_font_units.cap * parent_font_size
                        - style.font_relative_units.cap * style.font_size
                }
                "text-bottom" => (style.font_size - parent_font_size) * 0.2,
                "top" => (parent_line_height - style.line_height) / 2.0,
                "bottom" => (style.line_height - parent_line_height) / 2.0,
                _ if value.ends_with('%') => ctml_css::parse_css_percentage(value)
                    .filter(|percent| percent.is_finite())
                    .map_or(style.vertical_align_offset, |percent| {
                        style.line_height * percent / 100.0
                    }),
                _ => parse_signed_font_relative_length_with_viewport(
                    value,
                    style.font_size,
                    style.line_height,
                    style.font_relative_units,
                    style.viewport_size,
                )
                .unwrap_or(style.vertical_align_offset),
            };
        }
        "font-family" => {
            style.family = value.to_owned();
        }
        "font-weight" => {
            style.weight = match value.to_ascii_lowercase().as_str() {
                "normal" => Weight::NORMAL,
                "bold" => Weight::BOLD,
                "bolder" => relative_font_weight(style.weight, true),
                "lighter" => relative_font_weight(style.weight, false),
                _ => ctml_css::parse_math_number(value)
                    .or_else(|| ctml_css::parse_css_number(value))
                    .filter(|number| (1.0..=1000.0).contains(number))
                    .filter(|number| number.fract() == 0.0)
                    .map(|number| number as u16)
                    .filter(|weight| (1..=1000).contains(weight))
                    .map_or(style.weight, Weight),
            };
        }
        "font-style" => apply_font_style(style, value),
        "font-stretch" => {
            if let Some(stretch) = parse_font_stretch(value) {
                style.stretch = stretch;
            }
        }
        "font-kerning" => {
            style.kerning = match value.to_ascii_lowercase().as_str() {
                "auto" | "normal" => Some(true),
                "none" => Some(false),
                _ => style.kerning,
            };
        }
        "font-variant-ligatures" => apply_font_variant_ligatures(style, value),
        "font-variant" => apply_font_variant(style, value),
        "font-variant-caps" => apply_font_variant_caps(style, value),
        "font-variant-numeric" => apply_font_variant_numeric(style, value),
        "font-variant-east-asian" => apply_font_variant_east_asian(style, value),
        "font-feature-settings" => apply_font_feature_settings(style, value),
        "font-optical-sizing" => match value.to_ascii_lowercase().as_str() {
            "auto" => style.font_optical_sizing = true,
            "none" => style.font_optical_sizing = false,
            _ => {}
        },
        "font-variation-settings" => apply_font_variation_settings(style, value),
        "text-shadow" => apply_text_shadow(style, value),
        "text-emphasis"
        | "text-emphasis-style"
        | "text-emphasis-color"
        | "text-emphasis-position" => apply_text_emphasis_property(style, property, value),
        "font-variant-position" => {
            style.font_variant_position = match value.to_ascii_lowercase().as_str() {
                "normal" => FontVariantPosition::Normal,
                "sub" => FontVariantPosition::Sub,
                "super" => FontVariantPosition::Super,
                _ => style.font_variant_position,
            };
        }
        "letter-spacing" => apply_letter_spacing(style, value),
        "word-spacing" => apply_word_spacing(style, value),
        "text-transform" => {
            style.text_transform = match value.to_ascii_lowercase().as_str() {
                "uppercase" => TextTransform::Uppercase,
                "lowercase" => TextTransform::Lowercase,
                "capitalize" => TextTransform::Capitalize,
                "none" => TextTransform::None,
                _ => style.text_transform,
            };
        }
        "text-align" => {
            style.text_align = match value.to_ascii_lowercase().as_str() {
                "left" => cosmic_text::Align::Left,
                "start" => cosmic_text::Align::Start,
                "right" => cosmic_text::Align::Right,
                "end" => cosmic_text::Align::End,
                "center" => cosmic_text::Align::Center,
                "justify" => cosmic_text::Align::Justified,
                _ => style.text_align,
            };
        }
        "text-align-last" => {
            style.text_align_last = match value.to_ascii_lowercase().as_str() {
                "auto" => None,
                "left" => Some(Align::Left),
                "start" => Some(Align::Start),
                "right" => Some(Align::Right),
                "end" => Some(Align::End),
                "center" => Some(Align::Center),
                "justify" => Some(Align::JustifiedLast),
                _ => style.text_align_last,
            };
        }
        "text-justify" => {
            style.text_justify = match value.to_ascii_lowercase().as_str() {
                "auto" => cosmic_text::TextJustify::Auto,
                "none" => cosmic_text::TextJustify::None,
                "inter-word" => cosmic_text::TextJustify::InterWord,
                "inter-character" | "distribute" => cosmic_text::TextJustify::InterCharacter,
                _ => style.text_justify,
            };
        }
        "white-space" => {
            style.white_space = match value.to_ascii_lowercase().as_str() {
                "normal" => {
                    style.text_wrap_mode = TextWrapMode::Wrap;
                    WhiteSpace::Normal
                }
                "nowrap" => {
                    style.text_wrap_mode = TextWrapMode::NoWrap;
                    WhiteSpace::NoWrap
                }
                "pre" => {
                    style.text_wrap_mode = TextWrapMode::NoWrap;
                    WhiteSpace::Pre
                }
                "pre-wrap" => {
                    style.text_wrap_mode = TextWrapMode::Wrap;
                    WhiteSpace::PreWrap
                }
                "pre-line" => {
                    style.text_wrap_mode = TextWrapMode::Wrap;
                    WhiteSpace::PreLine
                }
                "break-spaces" => {
                    style.text_wrap_mode = TextWrapMode::Wrap;
                    WhiteSpace::BreakSpaces
                }
                _ => style.white_space,
            };
        }
        "text-wrap" | "text-wrap-mode" => {
            style.text_wrap_mode = match value.trim().to_ascii_lowercase().as_str() {
                "wrap" => TextWrapMode::Wrap,
                "nowrap" => TextWrapMode::NoWrap,
                _ => style.text_wrap_mode,
            };
        }
        "overflow-wrap" => {
            style.overflow_wrap = match value.to_ascii_lowercase().as_str() {
                "normal" => OverflowWrap::Normal,
                "break-word" => OverflowWrap::BreakWord,
                "anywhere" => OverflowWrap::Anywhere,
                _ => style.overflow_wrap,
            };
        }
        "word-break" => {
            style.word_break = match value.to_ascii_lowercase().as_str() {
                "normal" => WordBreak::Normal,
                "break-all" => WordBreak::BreakAll,
                "keep-all" => WordBreak::KeepAll,
                "break-word" => WordBreak::BreakWord,
                _ => style.word_break,
            };
        }
        "line-break" => {
            style.line_break = match value.to_ascii_lowercase().as_str() {
                "auto" => LineBreak::Auto,
                "loose" => LineBreak::Loose,
                "normal" => LineBreak::Normal,
                "strict" => LineBreak::Strict,
                "anywhere" => LineBreak::Anywhere,
                _ => style.line_break,
            };
        }
        "hyphens" => {
            style.hyphens = match value.to_ascii_lowercase().as_str() {
                "none" => Hyphens::None,
                "manual" => Hyphens::Manual,
                "auto" => Hyphens::Auto,
                _ => style.hyphens,
            };
        }
        "text-rendering" => {
            style.text_rendering = match value.to_ascii_lowercase().as_str() {
                "auto" => TextRendering::Auto,
                "optimizespeed" => TextRendering::OptimizeSpeed,
                "optimizelegibility" => TextRendering::OptimizeLegibility,
                "geometricprecision" => TextRendering::GeometricPrecision,
                _ => style.text_rendering,
            };
        }
        "text-orientation" => {
            style.text_orientation = match value.to_ascii_lowercase().as_str() {
                "mixed" => TextOrientation::Mixed,
                "upright" => TextOrientation::Upright,
                "sideways" => TextOrientation::Sideways,
                _ => style.text_orientation,
            };
        }
        "writing-mode" => {
            style.writing_mode = match value.to_ascii_lowercase().as_str() {
                "horizontal-tb" => WritingMode::HorizontalTb,
                "vertical-rl" => WritingMode::VerticalRl,
                "vertical-lr" => WritingMode::VerticalLr,
                _ => style.writing_mode,
            }
        }
        "direction" => {
            style.direction = match value.to_ascii_lowercase().as_str() {
                "ltr" => Direction::Ltr,
                "rtl" => Direction::Rtl,
                _ => style.direction,
            };
        }
        "unicode-bidi" => {
            style.unicode_bidi = match value.to_ascii_lowercase().as_str() {
                "normal" => UnicodeBidi::Normal,
                "isolate" => UnicodeBidi::Isolate,
                _ => style.unicode_bidi,
            };
        }
        "quotes" => apply_quotes(style, value),
        "counter-reset" => apply_counter_property(style, CounterProperty::Reset, value),
        "counter-increment" => apply_counter_property(style, CounterProperty::Increment, value),
        "counter-set" => apply_counter_property(style, CounterProperty::Set, value),
        "text-overflow" => {
            style.text_overflow = match value.to_ascii_lowercase().as_str() {
                "clip" => TextOverflow::Clip,
                "ellipsis" => TextOverflow::Ellipsis,
                _ => style.text_overflow,
            };
        }
        "line-clamp" => {
            style.line_clamp = if value.eq_ignore_ascii_case("none") {
                None
            } else {
                value
                    .parse::<usize>()
                    .ok()
                    .filter(|lines| *lines > 0)
                    .or(style.line_clamp)
            };
        }
        "text-indent" => apply_text_indent(style, value),
        "text-decoration"
        | "text-decoration-line"
        | "text-decoration-style"
        | "text-decoration-skip-ink"
        | "text-decoration-thickness"
        | "text-underline-offset"
        | "text-decoration-color" => apply_text_decoration_property(style, property, value),
        "cursor" => {
            if value.eq_ignore_ascii_case("none") {
                style.cursor = CursorIcon::Default;
                style.cursor_hidden = true;
            } else if let Some(cursor) = parse_cursor(value) {
                style.cursor = cursor;
                style.cursor_hidden = false;
            }
        }
        _ => {}
    }
}
