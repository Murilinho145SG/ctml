use crate::style::TextStyle;
use ctml_css::{
    parse_css_percentage, parse_math_expression,
    parse_nonnegative_font_relative_length_with_viewport,
};

pub(super) fn apply_border_radius(style: &mut TextStyle, value: &str) {
    let Some((horizontal, vertical)) = split_radius_axes(value) else {
        return;
    };
    let Some(horizontal) = expand_four(style, horizontal) else {
        return;
    };
    let Some(vertical) = vertical
        .map(|values| expand_four(style, values))
        .unwrap_or_else(|| Some(horizontal.clone()))
    else {
        return;
    };
    let order = style.current_declaration_order.saturating_add(1);
    for corner in 0..4 {
        style.border_radius[corner] = [
            Some(horizontal[corner].clone()),
            Some(vertical[corner].clone()),
        ];
        style.border_radius_orders[corner] = order;
    }
}

pub(super) fn apply_corner_radius(style: &mut TextStyle, corner: usize, value: &str) {
    let values = split_components(value);
    let parsed = values
        .iter()
        .map(|value| parse_radius_value(style, value))
        .collect::<Option<Vec<_>>>();
    let Some(values) = parsed.filter(|values| (1..=2).contains(&values.len())) else {
        return;
    };
    let vertical = values.get(1).unwrap_or(&values[0]).clone();
    style.border_radius[corner] = [Some(values[0].clone()), Some(vertical)];
    style.border_radius_orders[corner] = style.current_declaration_order.saturating_add(1);
}

pub(super) fn inherit_css_property(
    style: &mut TextStyle,
    inherited: &TextStyle,
    property: &str,
) -> bool {
    if property == "border-radius" {
        style.border_radius.clone_from(&inherited.border_radius);
        style.border_radius_orders = [style.current_declaration_order.saturating_add(1); 4];
        return true;
    }
    let Some(corner) = corner_index(property) else {
        return false;
    };
    style.border_radius[corner].clone_from(&inherited.border_radius[corner]);
    style.border_radius_orders[corner] = style.current_declaration_order.saturating_add(1);
    true
}

pub(super) fn initial_css_property(style: &mut TextStyle, property: &str) -> bool {
    if property == "border-radius" {
        style.border_radius = std::array::from_fn(|_| [None, None]);
        style.border_radius_orders = [style.current_declaration_order.saturating_add(1); 4];
        return true;
    }
    let Some(corner) = corner_index(property) else {
        return false;
    };
    style.border_radius[corner] = [None, None];
    style.border_radius_orders[corner] = style.current_declaration_order.saturating_add(1);
    true
}

fn corner_index(property: &str) -> Option<usize> {
    Some(match property {
        "border-top-left-radius" => 0,
        "border-top-right-radius" => 1,
        "border-bottom-right-radius" => 2,
        "border-bottom-left-radius" => 3,
        _ => return None,
    })
}

fn split_radius_axes(value: &str) -> Option<(&str, Option<&str>)> {
    let mut depth = 0usize;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            '/' if depth == 0 => {
                let (horizontal, vertical) = value.split_at(index);
                let vertical = &vertical[1..];
                return (!horizontal.trim().is_empty() && !vertical.trim().is_empty())
                    .then_some((horizontal.trim(), Some(vertical.trim())));
            }
            _ => {}
        }
    }
    (!value.trim().is_empty()).then_some((value.trim(), None))
}

fn expand_four(style: &TextStyle, values: &str) -> Option<[String; 4]> {
    let values = split_components(values)
        .into_iter()
        .map(|value| parse_radius_value(style, value))
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
        [top_left, top_right, bottom_right, bottom_left] => Some([
            top_left.clone(),
            top_right.clone(),
            bottom_right.clone(),
            bottom_left.clone(),
        ]),
        _ => None,
    }
}

fn parse_radius_value(style: &TextStyle, value: &str) -> Option<String> {
    let value = value.trim().to_ascii_lowercase();
    let metrics = || {
        parse_nonnegative_font_relative_length_with_viewport(
            &value,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
    };
    if parse_css_percentage(&value).is_some_and(|percentage| percentage >= 0.0)
        || metrics().is_some()
        || parse_math_expression(&value, |component| {
            parse_css_percentage(component)
                .filter(|percentage| *percentage >= 0.0)
                .map(|percentage| percentage / 100.0)
                .or_else(|| {
                    parse_nonnegative_font_relative_length_with_viewport(
                        component,
                        style.font_size,
                        style.line_height,
                        style.font_relative_units,
                        style.viewport_size,
                    )
                })
        })
        .is_some()
    {
        Some(value)
    } else {
        None
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
