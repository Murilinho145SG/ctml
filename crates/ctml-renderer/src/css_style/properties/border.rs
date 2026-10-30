use crate::style::{BorderLineStyle, Direction, TextStyle, WritingMode};
use ctml_css::parse_nonnegative_font_relative_length_with_viewport;

pub(super) fn apply_border_width(style: &mut TextStyle, value: &str) {
    let Some(values) = expand_four(value, |part| parse_width(style, part)) else {
        return;
    };
    style.border_width = values.map(Some);
    style.physical_border_width_orders = [style.current_declaration_order.saturating_add(1); 4];
}

pub(super) fn apply_border_width_side(style: &mut TextStyle, side: usize, value: &str) {
    if let Some(width) = parse_width(style, value) {
        style.border_width[side] = Some(width);
        style.physical_border_width_orders[side] =
            style.current_declaration_order.saturating_add(1);
    }
}

pub(super) fn apply_logical_border_width(style: &mut TextStyle, side: usize, value: &str) {
    if let Some(width) = parse_width(style, value) {
        style.logical_border_width[side] = Some(width);
        style.logical_border_width_orders[side] = style.current_declaration_order.saturating_add(1);
    }
}

pub(super) fn apply_logical_border_width_shorthand(
    style: &mut TextStyle,
    inline: bool,
    value: &str,
) {
    let Some(values) = expand_two(value, |part| parse_width(style, part)) else {
        return;
    };
    let start = if inline { 0 } else { 2 };
    let order = style.current_declaration_order.saturating_add(1);
    style.logical_border_width[start] = Some(values.0);
    style.logical_border_width[start + 1] = Some(values.1);
    style.logical_border_width_orders[start] = order;
    style.logical_border_width_orders[start + 1] = order;
}

pub(super) fn apply_border_style(style: &mut TextStyle, value: &str) {
    let Some(values) = expand_four(value, parse_border_style) else {
        return;
    };
    style.border_style = values;
    style.physical_border_style_orders = [style.current_declaration_order.saturating_add(1); 4];
}

pub(super) fn apply_border_style_side(style: &mut TextStyle, side: usize, value: &str) {
    if let Some(border_style) = parse_border_style(value) {
        style.border_style[side] = border_style;
        style.physical_border_style_orders[side] =
            style.current_declaration_order.saturating_add(1);
    }
}

pub(super) fn apply_logical_border_style(style: &mut TextStyle, side: usize, value: &str) {
    if let Some(border_style) = parse_border_style(value) {
        style.logical_border_style[side] = Some(border_style);
        style.logical_border_style_orders[side] = style.current_declaration_order.saturating_add(1);
    }
}

pub(super) fn apply_logical_border_style_shorthand(
    style: &mut TextStyle,
    inline: bool,
    value: &str,
) {
    let Some((first, second)) = expand_two(value, parse_border_style) else {
        return;
    };
    let start = if inline { 0 } else { 2 };
    let order = style.current_declaration_order.saturating_add(1);
    style.logical_border_style[start] = Some(first);
    style.logical_border_style[start + 1] = Some(second);
    style.logical_border_style_orders[start] = order;
    style.logical_border_style_orders[start + 1] = order;
}

pub(super) fn apply_border_color(style: &mut TextStyle, value: &str) {
    let Some(values) = expand_four(value, |part| parse_border_color(style, part)) else {
        return;
    };
    style.border_color = values;
    style.physical_border_color_orders = [style.current_declaration_order.saturating_add(1); 4];
}

pub(super) fn apply_border_color_side(style: &mut TextStyle, side: usize, value: &str) {
    if let Some(color) = parse_border_color(style, value) {
        style.border_color[side] = color;
        style.physical_border_color_orders[side] =
            style.current_declaration_order.saturating_add(1);
    }
}

pub(super) fn apply_logical_border_color(style: &mut TextStyle, side: usize, value: &str) {
    if let Some(color) = parse_border_color(style, value) {
        style.logical_border_color[side] = color;
        style.logical_border_color_orders[side] = style.current_declaration_order.saturating_add(1);
    }
}

pub(super) fn apply_logical_border_color_shorthand(
    style: &mut TextStyle,
    inline: bool,
    value: &str,
) {
    let Some((first, second)) = expand_two(value, |part| parse_border_color(style, part)) else {
        return;
    };
    let start = if inline { 0 } else { 2 };
    let order = style.current_declaration_order.saturating_add(1);
    style.logical_border_color[start] = first;
    style.logical_border_color[start + 1] = second;
    style.logical_border_color_orders[start] = order;
    style.logical_border_color_orders[start + 1] = order;
}

pub(super) fn apply_border_shorthand(style: &mut TextStyle, value: &str) {
    let Some(border) = parse_border(style, value) else {
        return;
    };
    for side in 0..4 {
        style.border_width[side] = border.width.clone();
        style.border_style[side] = border.style;
        style.border_color[side] = border.color;
        let order = style.current_declaration_order.saturating_add(1);
        style.physical_border_width_orders[side] = order;
        style.physical_border_style_orders[side] = order;
        style.physical_border_color_orders[side] = order;
    }
}

pub(super) fn apply_border_side(style: &mut TextStyle, side: usize, value: &str) {
    let Some(border) = parse_border(style, value) else {
        return;
    };
    style.border_width[side] = border.width;
    style.border_style[side] = border.style;
    style.border_color[side] = border.color;
    let order = style.current_declaration_order.saturating_add(1);
    style.physical_border_width_orders[side] = order;
    style.physical_border_style_orders[side] = order;
    style.physical_border_color_orders[side] = order;
}

pub(super) fn apply_logical_border_shorthand(style: &mut TextStyle, side: usize, value: &str) {
    let Some(border) = parse_border(style, value) else {
        return;
    };
    style.logical_border_width[side] = border.width;
    style.logical_border_style[side] = Some(border.style);
    style.logical_border_color[side] = border.color;
    let order = style.current_declaration_order.saturating_add(1);
    style.logical_border_width_orders[side] = order;
    style.logical_border_style_orders[side] = order;
    style.logical_border_color_orders[side] = order;
}

pub(super) fn apply_logical_border_axis_shorthand(
    style: &mut TextStyle,
    inline: bool,
    value: &str,
) {
    let Some(border) = parse_border(style, value) else {
        return;
    };
    let start = if inline { 0 } else { 2 };
    let order = style.current_declaration_order.saturating_add(1);
    for side in start..start + 2 {
        style.logical_border_width[side] = border.width.clone();
        style.logical_border_style[side] = Some(border.style);
        style.logical_border_color[side] = border.color;
        style.logical_border_width_orders[side] = order;
        style.logical_border_style_orders[side] = order;
        style.logical_border_color_orders[side] = order;
    }
}

struct BorderComponents {
    width: Option<String>,
    style: BorderLineStyle,
    color: Option<[u8; 4]>,
}

fn parse_border(style: &TextStyle, value: &str) -> Option<BorderComponents> {
    if split_components(value).is_empty() {
        return None;
    }
    let mut border = BorderComponents {
        width: None,
        style: BorderLineStyle::None,
        color: None,
    };
    let mut has_width = false;
    let mut has_style = false;
    let mut has_color = false;
    for component in split_components(value) {
        if let Some(width) = parse_width(style, component) {
            if has_width {
                return None;
            }
            border.width = Some(width);
            has_width = true;
        } else if let Some(border_style) = parse_border_style(component) {
            if has_style {
                return None;
            }
            border.style = border_style;
            has_style = true;
        } else if let Some(color) = parse_border_color(style, component) {
            if has_color {
                return None;
            }
            border.color = color;
            has_color = true;
        } else {
            return None;
        }
    }
    Some(border)
}

fn parse_border_color(style: &TextStyle, value: &str) -> Option<Option<[u8; 4]>> {
    if value.eq_ignore_ascii_case("currentcolor") {
        Some(None)
    } else {
        ctml_css::parse_color_with_profiles(value, &style.color_profiles).map(Some)
    }
}

fn split_components(value: &str) -> Vec<&str> {
    let mut components = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (index, character) in value.char_indices() {
        if character.is_whitespace() && depth == 0 {
            if let Some(component_start) = start.take() {
                components.push(&value[component_start..index]);
            }
            continue;
        }
        start.get_or_insert(index);
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    if let Some(component_start) = start {
        components.push(&value[component_start..]);
    }
    components
}

pub(super) fn resolve_logical_borders(style: &mut TextStyle) {
    let (inline_start, inline_end, block_start, block_end) = match style.writing_mode {
        WritingMode::HorizontalTb if style.direction == Direction::Ltr => (3, 1, 0, 2),
        WritingMode::HorizontalTb => (1, 3, 0, 2),
        WritingMode::VerticalRl if style.direction == Direction::Ltr => (0, 2, 1, 3),
        WritingMode::VerticalRl => (2, 0, 1, 3),
        WritingMode::VerticalLr if style.direction == Direction::Ltr => (0, 2, 3, 1),
        WritingMode::VerticalLr => (2, 0, 3, 1),
    };
    for (logical, physical) in [inline_start, inline_end, block_start, block_end]
        .into_iter()
        .enumerate()
    {
        let width_order = style.logical_border_width_orders[logical];
        if width_order > style.physical_border_width_orders[physical]
            && let Some(width) = &style.logical_border_width[logical]
        {
            style.border_width[physical] = Some(width.clone());
            style.physical_border_width_orders[physical] = width_order;
        }
        let style_order = style.logical_border_style_orders[logical];
        if style_order > style.physical_border_style_orders[physical]
            && let Some(border_style) = style.logical_border_style[logical]
        {
            style.border_style[physical] = border_style;
            style.physical_border_style_orders[physical] = style_order;
        }
        let color_order = style.logical_border_color_orders[logical];
        if color_order > style.physical_border_color_orders[physical] {
            style.border_color[physical] = style.logical_border_color[logical];
            style.physical_border_color_orders[physical] = color_order;
        }
    }
}

fn expand_four<T: Clone>(value: &str, parse: impl Fn(&str) -> Option<T>) -> Option<[T; 4]> {
    let values = split_components(value)
        .into_iter()
        .map(parse)
        .collect::<Option<Vec<_>>>()?;
    match values.as_slice() {
        [all] => Some([all.clone(), all.clone(), all.clone(), all.clone()]),
        [vertical, horizontal] => Some([
            vertical.clone(),
            horizontal.clone(),
            vertical.clone(),
            horizontal.clone(),
        ]),
        [top, horizontal, bottom] => Some([
            top.clone(),
            horizontal.clone(),
            bottom.clone(),
            horizontal.clone(),
        ]),
        [top, right, bottom, left] => {
            Some([top.clone(), right.clone(), bottom.clone(), left.clone()])
        }
        _ => None,
    }
}

fn expand_two<T: Clone>(value: &str, parse: impl Fn(&str) -> Option<T>) -> Option<(T, T)> {
    let values = split_components(value)
        .into_iter()
        .map(parse)
        .collect::<Option<Vec<_>>>()?;
    match values.as_slice() {
        [all] => Some((all.clone(), all.clone())),
        [first, second] => Some((first.clone(), second.clone())),
        _ => None,
    }
}

fn parse_width(style: &TextStyle, value: &str) -> Option<String> {
    let value = value.trim().to_ascii_lowercase();
    if matches!(value.as_str(), "thin" | "medium" | "thick" | "0")
        || ctml_css::parse_math_expression(&value, |component| {
            parse_nonnegative_font_relative_length_with_viewport(
                component,
                style.font_size,
                style.line_height,
                style.font_relative_units,
                style.viewport_size,
            )
        })
        .is_some()
        || parse_nonnegative_font_relative_length_with_viewport(
            &value,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
        .is_some()
    {
        Some(value)
    } else {
        None
    }
}

fn parse_border_style(value: &str) -> Option<BorderLineStyle> {
    Some(match value.trim().to_ascii_lowercase().as_str() {
        "none" => BorderLineStyle::None,
        "hidden" => BorderLineStyle::Hidden,
        "solid" => BorderLineStyle::Solid,
        "dotted" => BorderLineStyle::Dotted,
        "dashed" => BorderLineStyle::Dashed,
        "double" => BorderLineStyle::Double,
        "groove" => BorderLineStyle::Groove,
        "ridge" => BorderLineStyle::Ridge,
        "inset" => BorderLineStyle::Inset,
        "outset" => BorderLineStyle::Outset,
        _ => return None,
    })
}
