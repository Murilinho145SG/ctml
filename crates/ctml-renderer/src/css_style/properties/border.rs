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
    }
}

pub(super) fn inherit_css_property(
    style: &mut TextStyle,
    inherited: &TextStyle,
    property: &str,
) -> bool {
    match property {
        "border-width" => {
            style.border_width.clone_from(&inherited.border_width);
            style.physical_border_width_orders =
                [style.current_declaration_order.saturating_add(1); 4];
        }
        "border-style" => {
            style.border_style = inherited.border_style;
            style.physical_border_style_orders =
                [style.current_declaration_order.saturating_add(1); 4];
        }
        "border-inline-width" | "border-block-width" => {
            let start = usize::from(property == "border-block-width") * 2;
            style.logical_border_width[start..start + 2]
                .clone_from_slice(&inherited.logical_border_width[start..start + 2]);
            style.logical_border_width_orders[start..start + 2]
                .fill(style.current_declaration_order.saturating_add(1));
        }
        "border-inline-style" | "border-block-style" => {
            let start = usize::from(property == "border-block-style") * 2;
            style.logical_border_style[start..start + 2]
                .clone_from_slice(&inherited.logical_border_style[start..start + 2]);
            style.logical_border_style_orders[start..start + 2]
                .fill(style.current_declaration_order.saturating_add(1));
        }
        _ => {
            let Some((logical, side, width)) = border_side_property(property) else {
                return false;
            };
            if logical {
                if width {
                    style.logical_border_width[side] = inherited.logical_border_width[side].clone();
                    style.logical_border_width_orders[side] =
                        style.current_declaration_order.saturating_add(1);
                } else {
                    style.logical_border_style[side] = inherited.logical_border_style[side];
                    style.logical_border_style_orders[side] =
                        style.current_declaration_order.saturating_add(1);
                }
            } else if width {
                style.border_width[side] = inherited.border_width[side].clone();
                style.physical_border_width_orders[side] =
                    style.current_declaration_order.saturating_add(1);
            } else {
                style.border_style[side] = inherited.border_style[side];
                style.physical_border_style_orders[side] =
                    style.current_declaration_order.saturating_add(1);
            }
        }
    }
    true
}

pub(super) fn initial_css_property(style: &mut TextStyle, property: &str) -> bool {
    match property {
        "border-width" => {
            style.border_width = std::array::from_fn(|_| None);
            style.physical_border_width_orders =
                [style.current_declaration_order.saturating_add(1); 4];
        }
        "border-style" => {
            style.border_style = [BorderLineStyle::None; 4];
            style.physical_border_style_orders =
                [style.current_declaration_order.saturating_add(1); 4];
        }
        "border-inline-width" | "border-block-width" => {
            let start = usize::from(property == "border-block-width") * 2;
            style.logical_border_width[start..start + 2].fill(Some("medium".to_owned()));
            style.logical_border_width_orders[start..start + 2]
                .fill(style.current_declaration_order.saturating_add(1));
        }
        "border-inline-style" | "border-block-style" => {
            let start = usize::from(property == "border-block-style") * 2;
            style.logical_border_style[start..start + 2].fill(Some(BorderLineStyle::None));
            style.logical_border_style_orders[start..start + 2]
                .fill(style.current_declaration_order.saturating_add(1));
        }
        _ => {
            let Some((logical, side, width)) = border_side_property(property) else {
                return false;
            };
            if logical {
                if width {
                    style.logical_border_width[side] = Some("medium".to_owned());
                    style.logical_border_width_orders[side] =
                        style.current_declaration_order.saturating_add(1);
                } else {
                    style.logical_border_style[side] = Some(BorderLineStyle::None);
                    style.logical_border_style_orders[side] =
                        style.current_declaration_order.saturating_add(1);
                }
            } else if width {
                style.border_width[side] = None;
                style.physical_border_width_orders[side] =
                    style.current_declaration_order.saturating_add(1);
            } else {
                style.border_style[side] = BorderLineStyle::None;
                style.physical_border_style_orders[side] =
                    style.current_declaration_order.saturating_add(1);
            }
        }
    }
    true
}

fn border_side_property(property: &str) -> Option<(bool, usize, bool)> {
    let (side_name, width) = if let Some(side) = property.strip_suffix("-width") {
        (side, true)
    } else if let Some(side) = property.strip_suffix("-style") {
        (side, false)
    } else {
        return None;
    };
    let side_name = side_name.strip_prefix("border-")?;
    let (logical, side) = match side_name {
        "top" => (false, 0),
        "right" => (false, 1),
        "bottom" => (false, 2),
        "left" => (false, 3),
        "inline-start" => (true, 0),
        "inline-end" => (true, 1),
        "block-start" => (true, 2),
        "block-end" => (true, 3),
        _ => return None,
    };
    Some((logical, side, width))
}

fn expand_four<T: Clone>(value: &str, parse: impl Fn(&str) -> Option<T>) -> Option<[T; 4]> {
    let values = value
        .split_whitespace()
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
    let values = value
        .split_whitespace()
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
