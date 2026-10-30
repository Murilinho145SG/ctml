use crate::style::{BorderLineStyle, TextStyle};
use ctml_css::parse_signed_font_relative_length_with_viewport;

use super::border::{parse_border_color, parse_border_style, parse_width};

pub(super) fn apply_outline(style: &mut TextStyle, value: &str) {
    let parts = split_components(value);
    if parts.is_empty() {
        return;
    }
    let mut width = Some("medium".to_owned());
    let mut line_style = BorderLineStyle::None;
    let mut color = None;
    let (mut has_width, mut has_style, mut has_color) = (false, false, false);
    for part in parts {
        if let Some(parsed) = parse_width(style, part) {
            if has_width {
                return;
            }
            width = Some(parsed);
            has_width = true;
        } else if let Some(parsed) = if part.eq_ignore_ascii_case("auto") {
            Some(BorderLineStyle::Solid)
        } else {
            parse_border_style(part).filter(|style| *style != BorderLineStyle::Hidden)
        } {
            if has_style {
                return;
            }
            line_style = parsed;
            has_style = true;
        } else if let Some(parsed) = parse_border_color(style, part) {
            if has_color {
                return;
            }
            color = parsed;
            has_color = true;
        } else {
            return;
        }
    }
    style.outline_width = width;
    style.outline_style = line_style;
    style.outline_color = color;
}

pub(super) fn apply_outline_width(style: &mut TextStyle, value: &str) {
    if let Some(width) = parse_width(style, value) {
        style.outline_width = Some(width);
    }
}

pub(super) fn apply_outline_style(style: &mut TextStyle, value: &str) {
    if value.trim().eq_ignore_ascii_case("auto") {
        style.outline_style = BorderLineStyle::Solid;
        return;
    }
    if let Some(line_style) =
        parse_border_style(value).filter(|style| *style != BorderLineStyle::Hidden)
    {
        style.outline_style = line_style;
    }
}

pub(super) fn apply_outline_color(style: &mut TextStyle, value: &str) {
    if let Some(color) = parse_border_color(style, value) {
        style.outline_color = color;
    }
}

pub(super) fn apply_outline_offset(style: &mut TextStyle, value: &str) {
    let value = value.trim().to_ascii_lowercase();
    if value.contains('%') {
        return;
    }
    let parsed = parse_signed_font_relative_length_with_viewport(
        &value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    );
    let parsed_math = ctml_css::parse_math_expression(&value, |component| {
        parse_signed_font_relative_length_with_viewport(
            component,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
    });
    if parsed.is_some() || parsed_math.is_some() {
        style.outline_offset = Some(value);
    }
}

pub(super) fn inherit_css_property(
    style: &mut TextStyle,
    inherited: &TextStyle,
    property: &str,
) -> bool {
    match property {
        "outline" => {
            style.outline_width.clone_from(&inherited.outline_width);
            style.outline_style = inherited.outline_style;
            style.outline_color = inherited.outline_color;
            true
        }
        "outline-width" => {
            style.outline_width.clone_from(&inherited.outline_width);
            true
        }
        "outline-style" => {
            style.outline_style = inherited.outline_style;
            true
        }
        "outline-color" => {
            style.outline_color = inherited.outline_color;
            true
        }
        "outline-offset" => {
            style.outline_offset.clone_from(&inherited.outline_offset);
            true
        }
        _ => false,
    }
}

pub(super) fn initial_css_property(style: &mut TextStyle, property: &str) -> bool {
    match property {
        "outline" => {
            style.outline_width = Some("medium".to_owned());
            style.outline_style = BorderLineStyle::None;
            style.outline_color = None;
            true
        }
        "outline-width" => {
            style.outline_width = Some("medium".to_owned());
            true
        }
        "outline-style" => {
            style.outline_style = BorderLineStyle::None;
            true
        }
        "outline-color" => {
            style.outline_color = None;
            true
        }
        "outline-offset" => {
            style.outline_offset = None;
            true
        }
        _ => false,
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
