use crate::style::TextStyle;
use ctml_css::{
    parse_css_dimension, parse_css_percentage,
    parse_nonnegative_font_relative_length_with_viewport,
    parse_signed_font_relative_length_with_viewport,
};

pub(super) fn apply_padding(style: &mut TextStyle, value: &str) {
    let values = value
        .split_whitespace()
        .map(|part| parse_padding_value(style, part))
        .collect::<Option<Vec<_>>>();
    let Some(values) = values.filter(|values| (1..=4).contains(&values.len())) else {
        return;
    };
    let [top, right, bottom, left] = match values.as_slice() {
        [all] => [all.clone(), all.clone(), all.clone(), all.clone()],
        [vertical, horizontal] => [
            vertical.clone(),
            horizontal.clone(),
            vertical.clone(),
            horizontal.clone(),
        ],
        [top, horizontal, bottom] => [
            top.clone(),
            horizontal.clone(),
            bottom.clone(),
            horizontal.clone(),
        ],
        [top, right, bottom, left] => [top.clone(), right.clone(), bottom.clone(), left.clone()],
        _ => return,
    };
    style.padding = [Some(top), Some(right), Some(bottom), Some(left)];
    style.physical_padding_orders = [style.current_declaration_order.saturating_add(1); 4];
}

pub(super) fn apply_padding_side(style: &mut TextStyle, side: usize, value: &str) {
    if let Some(padding) = parse_padding_value(style, value) {
        style.padding[side] = Some(padding);
        style.physical_padding_orders[side] = style.current_declaration_order.saturating_add(1);
    }
}

pub(super) fn apply_margin(style: &mut TextStyle, value: &str) {
    let values = value
        .split_whitespace()
        .map(|part| parse_margin_value(style, part))
        .collect::<Option<Vec<_>>>();
    let Some(values) = values.filter(|values| (1..=4).contains(&values.len())) else {
        return;
    };
    let [top, right, bottom, left] = match values.as_slice() {
        [all] => [all.clone(), all.clone(), all.clone(), all.clone()],
        [vertical, horizontal] => [
            vertical.clone(),
            horizontal.clone(),
            vertical.clone(),
            horizontal.clone(),
        ],
        [top, horizontal, bottom] => [
            top.clone(),
            horizontal.clone(),
            bottom.clone(),
            horizontal.clone(),
        ],
        [top, right, bottom, left] => [top.clone(), right.clone(), bottom.clone(), left.clone()],
        _ => return,
    };
    style.margin = [Some(top), Some(right), Some(bottom), Some(left)];
    style.physical_margin_orders = [style.current_declaration_order.saturating_add(1); 4];
}

pub(super) fn apply_margin_side(style: &mut TextStyle, side: usize, value: &str) {
    if let Some(margin) = parse_margin_value(style, value) {
        style.margin[side] = Some(margin);
        style.physical_margin_orders[side] = style.current_declaration_order.saturating_add(1);
    }
}

pub(super) fn apply_logical_margin(style: &mut TextStyle, side: usize, value: &str) {
    if let Some(margin) = parse_margin_value(style, value) {
        style.logical_margin[side] = Some(margin);
        style.logical_margin_orders[side] = style.current_declaration_order.saturating_add(1);
    }
}

pub(super) fn apply_logical_margin_shorthand(style: &mut TextStyle, inline: bool, value: &str) {
    let values = value
        .split_whitespace()
        .map(|part| parse_margin_value(style, part))
        .collect::<Option<Vec<_>>>();
    let Some(values) = values.filter(|values| (1..=2).contains(&values.len())) else {
        return;
    };
    let (first, second) = match values.as_slice() {
        [value] => (value.clone(), value.clone()),
        [first, second] => (first.clone(), second.clone()),
        _ => return,
    };
    let start = if inline { 0 } else { 2 };
    let order = style.current_declaration_order.saturating_add(1);
    style.logical_margin[start] = Some(first);
    style.logical_margin[start + 1] = Some(second);
    style.logical_margin_orders[start] = order;
    style.logical_margin_orders[start + 1] = order;
}

pub(super) fn resolve_logical_margin(style: &mut TextStyle) {
    use crate::style::{Direction, WritingMode};

    let (inline_start, inline_end, block_start, block_end) = match style.writing_mode {
        WritingMode::HorizontalTb if style.direction == Direction::Ltr => (3, 1, 0, 2),
        WritingMode::HorizontalTb => (1, 3, 0, 2),
        WritingMode::VerticalRl if style.direction == Direction::Ltr => (0, 2, 1, 3),
        WritingMode::VerticalRl => (2, 0, 1, 3),
        WritingMode::VerticalLr if style.direction == Direction::Ltr => (0, 2, 3, 1),
        WritingMode::VerticalLr => (2, 0, 3, 1),
    };
    for (logical_side, physical_side) in [inline_start, inline_end, block_start, block_end]
        .into_iter()
        .enumerate()
    {
        let logical_order = style.logical_margin_orders[logical_side];
        if logical_order > style.physical_margin_orders[physical_side]
            && let Some(value) = &style.logical_margin[logical_side]
        {
            style.margin[physical_side] = Some(value.clone());
            style.physical_margin_orders[physical_side] = logical_order;
        }
    }
}

fn parse_margin_value(style: &TextStyle, value: &str) -> Option<String> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        Some("auto".to_owned())
    } else if value == "0" {
        Some("0".to_owned())
    } else if parse_css_percentage(value).is_some() {
        Some(value.to_ascii_lowercase())
    } else if parse_signed_font_relative_length_with_viewport(
        value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    )
    .is_some()
    {
        Some(value.to_ascii_lowercase())
    } else {
        None
    }
}

pub(super) fn is_margin_property(property: &str) -> bool {
    matches!(
        property,
        "margin"
            | "margin-top"
            | "margin-right"
            | "margin-bottom"
            | "margin-left"
            | "margin-inline"
            | "margin-inline-start"
            | "margin-inline-end"
            | "margin-block"
            | "margin-block-start"
            | "margin-block-end"
    )
}

pub(super) fn inherit_margin_property(
    style: &mut TextStyle,
    inherited: &TextStyle,
    property: &str,
) {
    let order = style.current_declaration_order.saturating_add(1);
    match property {
        "margin" => {
            style.margin.clone_from(&inherited.margin);
            style.physical_margin_orders = [order; 4];
        }
        "margin-top" | "margin-right" | "margin-bottom" | "margin-left" => {
            let side = margin_physical_side(property);
            style.margin[side].clone_from(&inherited.margin[side]);
            style.physical_margin_orders[side] = order;
        }
        "margin-inline" | "margin-block" => {
            let start = if property == "margin-inline" { 0 } else { 2 };
            for side in start..start + 2 {
                style.logical_margin[side].clone_from(&inherited.logical_margin[side]);
                style.logical_margin_orders[side] = order;
            }
        }
        _ => {
            let side = margin_logical_side(property);
            style.logical_margin[side].clone_from(&inherited.logical_margin[side]);
            style.logical_margin_orders[side] = order;
        }
    }
}

pub(super) fn reset_margin_property(style: &mut TextStyle, property: &str) {
    let order = style.current_declaration_order.saturating_add(1);
    match property {
        "margin" => {
            style.margin = std::array::from_fn(|_| Some("0".to_owned()));
            style.physical_margin_orders = [order; 4];
        }
        "margin-top" | "margin-right" | "margin-bottom" | "margin-left" => {
            let side = margin_physical_side(property);
            style.margin[side] = Some("0".to_owned());
            style.physical_margin_orders[side] = order;
        }
        "margin-inline" | "margin-block" => {
            let start = if property == "margin-inline" { 0 } else { 2 };
            for side in start..start + 2 {
                style.logical_margin[side] = Some("0".to_owned());
                style.logical_margin_orders[side] = order;
            }
        }
        _ => {
            let side = margin_logical_side(property);
            style.logical_margin[side] = Some("0".to_owned());
            style.logical_margin_orders[side] = order;
        }
    }
}

fn margin_physical_side(property: &str) -> usize {
    match property {
        "margin-top" => 0,
        "margin-right" => 1,
        "margin-bottom" => 2,
        _ => 3,
    }
}

fn margin_logical_side(property: &str) -> usize {
    match property {
        "margin-inline-start" => 0,
        "margin-inline-end" => 1,
        "margin-block-start" => 2,
        _ => 3,
    }
}

pub(super) fn apply_logical_padding(style: &mut TextStyle, side: usize, value: &str) {
    if let Some(padding) = parse_padding_value(style, value) {
        style.logical_padding[side] = Some(padding);
        style.logical_padding_orders[side] = style.current_declaration_order.saturating_add(1);
    }
}

pub(super) fn apply_logical_padding_shorthand(style: &mut TextStyle, inline: bool, value: &str) {
    let values = value
        .split_whitespace()
        .map(|part| parse_padding_value(style, part))
        .collect::<Option<Vec<_>>>();
    let Some(values) = values.filter(|values| (1..=2).contains(&values.len())) else {
        return;
    };
    let (first, second) = match values.as_slice() {
        [value] => (value.clone(), value.clone()),
        [first, second] => (first.clone(), second.clone()),
        _ => return,
    };
    let start = if inline { 0 } else { 2 };
    let order = style.current_declaration_order.saturating_add(1);
    style.logical_padding[start] = Some(first);
    style.logical_padding[start + 1] = Some(second);
    style.logical_padding_orders[start] = order;
    style.logical_padding_orders[start + 1] = order;
}

pub(super) fn resolve_logical_padding(style: &mut TextStyle) {
    use crate::style::{Direction, WritingMode};

    let (inline_start, inline_end, block_start, block_end) = match style.writing_mode {
        WritingMode::HorizontalTb if style.direction == Direction::Ltr => (3, 1, 0, 2),
        WritingMode::HorizontalTb => (1, 3, 0, 2),
        WritingMode::VerticalRl if style.direction == Direction::Ltr => (0, 2, 1, 3),
        WritingMode::VerticalRl => (2, 0, 1, 3),
        WritingMode::VerticalLr if style.direction == Direction::Ltr => (0, 2, 3, 1),
        WritingMode::VerticalLr => (2, 0, 3, 1),
    };
    let physical_sides = [inline_start, inline_end, block_start, block_end];
    for (logical_side, physical_side) in physical_sides.into_iter().enumerate() {
        let logical_order = style.logical_padding_orders[logical_side];
        if logical_order > style.physical_padding_orders[physical_side]
            && let Some(value) = &style.logical_padding[logical_side]
        {
            style.padding[physical_side] = Some(value.clone());
            style.physical_padding_orders[physical_side] = logical_order;
        }
    }
}

fn parse_padding_value(style: &TextStyle, value: &str) -> Option<String> {
    let value = value.trim();
    if value == "0" {
        Some("0".to_owned())
    } else if parse_css_percentage(value).is_some_and(|percentage| percentage >= 0.0) {
        Some(value.to_ascii_lowercase())
    } else if parse_nonnegative_font_relative_length_with_viewport(
        value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    )
    .is_some()
    {
        Some(value.to_ascii_lowercase())
    } else {
        None
    }
}

pub(super) fn apply_letter_spacing(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("normal") {
        style.letter_spacing = 0.0;
        style.letter_spacing_px = false;
    } else if let Some((px, unit)) = parse_css_dimension(value)
        && unit == "px"
    {
        style.letter_spacing = px;
        style.letter_spacing_px = true;
    } else if let Some((em, unit)) = parse_css_dimension(value)
        && unit == "em"
    {
        style.letter_spacing = em;
        style.letter_spacing_px = false;
    } else if !value.contains('%')
        && let Some(length) = parse_signed_font_relative_length_with_viewport(
            value,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
    {
        style.letter_spacing = length;
        style.letter_spacing_px = true;
    }
}

pub(super) fn apply_word_spacing(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("normal") {
        style.word_spacing_px = 0.0;
        style.word_spacing_percent = None;
    } else if let Some(percent) = parse_css_percentage(value) {
        style.word_spacing_px = 0.0;
        style.word_spacing_percent = Some(percent);
    } else if let Some(length) = parse_signed_font_relative_length_with_viewport(
        value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    ) {
        style.word_spacing_px = length;
        style.word_spacing_percent = None;
    }
}
