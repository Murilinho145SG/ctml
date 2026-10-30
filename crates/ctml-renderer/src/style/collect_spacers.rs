use super::{TextRun, TextStyle, append_text, push_line_break};

pub(super) fn append_block_start_spacers(
    style: &TextStyle,
    block: bool,
    empty_box: bool,
    has_content: bool,
    element_path: &[usize],
    output: &mut Vec<TextRun>,
    last_was_space: &mut bool,
    capitalize_next: &mut bool,
) {
    if !block
        && empty_box
        && !style.content_skipped
        && (!style.background_layers.is_empty()
            || style.padding.iter().flatten().any(|padding| padding != "0")
            || style
                .border_style
                .iter()
                .any(|style| *style != super::BorderLineStyle::None))
    {
        let last_was_space_before = *last_was_space;
        let capitalize_next_before = *capitalize_next;
        append_text(
            "\u{200b}",
            style,
            element_path,
            output,
            last_was_space,
            capitalize_next,
        );
        *last_was_space = last_was_space_before;
        *capitalize_next = capitalize_next_before;
    }
    let margin_start_side = match style.writing_mode {
        super::WritingMode::HorizontalTb => 0,
        super::WritingMode::VerticalRl => 1,
        super::WritingMode::VerticalLr => 3,
    };
    if block
        && style.writing_mode == super::WritingMode::HorizontalTb
        && (style.padding[0]
            .as_deref()
            .is_some_and(|padding| padding != "0")
            || style.border_style[0] != super::BorderLineStyle::None)
        && has_content
    {
        let mut spacer_style = style.clone();
        spacer_style.font_size = 1.0;
        let padding = style.padding[0]
            .as_deref()
            .and_then(|padding| {
                ctml_css::parse_nonnegative_font_relative_length_with_viewport(
                    padding,
                    style.font_size,
                    style.line_height,
                    style.font_relative_units,
                    style.viewport_size,
                )
            })
            .unwrap_or_default();
        let border = if style.border_style[0] == super::BorderLineStyle::None {
            0.0
        } else {
            match style.border_width[0].as_deref().unwrap_or("medium") {
                "thin" => 1.0,
                "medium" => 3.0,
                "thick" => 5.0,
                value => ctml_css::parse_nonnegative_font_relative_length_with_viewport(
                    value,
                    style.font_size,
                    style.line_height,
                    style.font_relative_units,
                    style.viewport_size,
                )
                .unwrap_or_default(),
            }
        };
        spacer_style.line_height = padding + border;
        spacer_style.line_height_normal = false;
        spacer_style.flow_spacer = true;
        spacer_style.padding_start_spacer = true;
        let last_was_space_before = *last_was_space;
        let capitalize_next_before = *capitalize_next;
        append_text(
            "\u{200b}",
            &spacer_style,
            element_path,
            output,
            last_was_space,
            capitalize_next,
        );
        push_line_break(output, &spacer_style, element_path, last_was_space);
        *last_was_space = last_was_space_before;
        *capitalize_next = capitalize_next_before;
    }
    if block
        && style.margin[margin_start_side]
            .as_deref()
            .is_some_and(|margin| {
                margin
                    .strip_suffix('%')
                    .and_then(|percent| percent.trim().parse::<f32>().ok())
                    .is_some_and(|percent| percent > 0.0)
                    || ctml_css::parse_signed_font_relative_length_with_viewport(
                        margin,
                        style.font_size,
                        style.line_height,
                        style.font_relative_units,
                        style.viewport_size,
                    )
                    .is_some_and(|length| length > 0.0)
            })
        && has_content
    {
        let mut spacer_style = style.clone();
        spacer_style.font_size = 1.0;
        spacer_style.line_height = style.margin[margin_start_side]
            .as_deref()
            .and_then(|margin| {
                ctml_css::parse_signed_font_relative_length_with_viewport(
                    margin,
                    style.font_size,
                    style.line_height,
                    style.font_relative_units,
                    style.viewport_size,
                )
            })
            .unwrap_or_default();
        spacer_style.line_height_normal = false;
        spacer_style.flow_spacer = true;
        spacer_style.margin_start_spacer = true;
        let last_was_space_before = *last_was_space;
        let capitalize_next_before = *capitalize_next;
        append_text(
            "\u{200b}",
            &spacer_style,
            element_path,
            output,
            last_was_space,
            capitalize_next,
        );
        push_line_break(output, &spacer_style, element_path, last_was_space);
        *last_was_space = last_was_space_before;
        *capitalize_next = capitalize_next_before;
    }
}
