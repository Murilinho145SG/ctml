use crate::style::TextStyle;
use ctml_css::{
    parse_color_with_profiles, parse_nonnegative_font_relative_length_with_viewport,
    parse_signed_font_relative_length_with_viewport,
};

pub(super) fn apply_text_decoration_shorthand(style: &mut TextStyle, value: &str) {
    let components = split_css_whitespace(value);
    if components.is_empty() {
        return;
    }
    let mut lines = Vec::new();
    let mut decoration_style = None;
    let mut thickness = None;
    let mut color = None;
    for component in components {
        let lower = component.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "none" | "underline" | "overline" | "line-through"
        ) {
            if lines
                .iter()
                .any(|line: &&str| line.eq_ignore_ascii_case(component))
                || (lower == "none" && !lines.is_empty())
                || (lower != "none" && lines.iter().any(|line| line.eq_ignore_ascii_case("none")))
            {
                return;
            }
            lines.push(component);
        } else if let Some(parsed_style) = parse_text_decoration_style(component) {
            if decoration_style.replace(parsed_style).is_some() {
                return;
            }
        } else if matches!(lower.as_str(), "auto" | "from-font") {
            if thickness.replace(None).is_some() {
                return;
            }
        } else if let Some(parsed_thickness) = parse_nonnegative_font_relative_length_with_viewport(
            component,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        ) {
            if thickness.replace(Some(parsed_thickness)).is_some() {
                return;
            }
        } else if lower == "currentcolor" {
            if color.replace(None).is_some() {
                return;
            }
        } else if let Some(parsed_color) =
            parse_color_with_profiles(component, &style.color_profiles)
        {
            if color.replace(Some(parsed_color)).is_some() {
                return;
            }
        } else {
            return;
        }
    }

    reset_text_decoration(style);
    if !lines.is_empty() {
        apply_text_decoration_line(style, &lines.join(" "));
    }
    if let Some(parsed_style) = decoration_style {
        style.decoration_style = parsed_style;
    }
    if let Some(parsed_thickness) = thickness {
        style.decoration_thickness = parsed_thickness;
    }
    if let Some(parsed_color) = color {
        style.text_decoration_color = parsed_color;
    }
}

pub(super) fn apply_text_decoration_line(style: &mut TextStyle, value: &str) {
    let components = split_css_whitespace(value);
    if components.is_empty() {
        return;
    }
    if components.len() == 1 && components[0].eq_ignore_ascii_case("none") {
        style.underline = false;
        style.overline = false;
        style.strikethrough = false;
        return;
    }
    let mut underline = false;
    let mut overline = false;
    let mut strikethrough = false;
    for component in components {
        match component.to_ascii_lowercase().as_str() {
            "underline" if !underline => underline = true,
            "overline" if !overline => overline = true,
            "line-through" if !strikethrough => strikethrough = true,
            _ => return,
        }
    }
    style.underline = underline;
    style.overline = overline;
    style.strikethrough = strikethrough;
}

fn parse_text_decoration_style(value: &str) -> Option<crate::style::TextDecorationStyle> {
    Some(match value.to_ascii_lowercase().as_str() {
        "solid" => crate::style::TextDecorationStyle::Solid,
        "double" => crate::style::TextDecorationStyle::Double,
        "dotted" => crate::style::TextDecorationStyle::Dotted,
        "dashed" => crate::style::TextDecorationStyle::Dashed,
        "wavy" => crate::style::TextDecorationStyle::Wavy,
        _ => return None,
    })
}

pub(super) fn reset_text_decoration(style: &mut TextStyle) {
    style.underline = false;
    style.overline = false;
    style.strikethrough = false;
    style.decoration_style = crate::style::TextDecorationStyle::Solid;
    style.decoration_thickness = None;
    style.text_decoration_color = None;
}

pub(super) fn inherit_text_decoration(style: &mut TextStyle, inherited: &TextStyle) {
    copy_text_decoration_lines(style, inherited);
    style.decoration_style = inherited.decoration_style;
    style.decoration_thickness = inherited.decoration_thickness;
    style.text_decoration_color = inherited.text_decoration_color;
}

pub(super) fn copy_text_decoration_lines(style: &mut TextStyle, inherited: &TextStyle) {
    style.underline = inherited.underline;
    style.overline = inherited.overline;
    style.strikethrough = inherited.strikethrough;
}

pub(super) fn apply_text_decoration_property(style: &mut TextStyle, property: &str, value: &str) {
    match property {
        "text-decoration" => apply_text_decoration_shorthand(style, value),
        "text-decoration-line" => apply_text_decoration_line(style, value),
        "text-decoration-style" => {
            if let Some(decoration_style) = parse_text_decoration_style(value) {
                style.decoration_style = decoration_style;
            }
        }
        "text-decoration-skip-ink" => {
            style.decoration_skip_ink = match value.to_ascii_lowercase().as_str() {
                "auto" => crate::style::TextDecorationSkipInk::Auto,
                "none" => crate::style::TextDecorationSkipInk::None,
                "all" => crate::style::TextDecorationSkipInk::All,
                _ => style.decoration_skip_ink,
            };
        }
        "text-decoration-thickness" => {
            if matches!(value.to_ascii_lowercase().as_str(), "auto" | "from-font") {
                style.decoration_thickness = None;
            } else if let Some(thickness) = parse_nonnegative_font_relative_length_with_viewport(
                value,
                style.font_size,
                style.line_height,
                style.font_relative_units,
                style.viewport_size,
            ) {
                style.decoration_thickness = Some(thickness);
            }
        }
        "text-underline-offset" => {
            if value.eq_ignore_ascii_case("auto") {
                style.underline_offset = None;
            } else if let Some(offset) = parse_signed_font_relative_length_with_viewport(
                value,
                style.font_size,
                style.line_height,
                style.font_relative_units,
                style.viewport_size,
            ) {
                style.underline_offset = Some(offset);
            }
        }
        "text-decoration-color" => {
            style.text_decoration_color = if value.eq_ignore_ascii_case("currentcolor") {
                None
            } else {
                parse_color_with_profiles(value, &style.color_profiles)
            };
        }
        _ => {}
    }
}

pub(super) fn split_css_whitespace(value: &str) -> Vec<&str> {
    let mut components = Vec::new();
    let mut start = None;
    let mut depth: usize = 0;
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if start.is_none() && !character.is_whitespace() {
            start = Some(index);
        }
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == active_quote {
                quote = None;
            }
        } else {
            match character {
                '\'' | '"' => quote = Some(character),
                '(' => depth += 1,
                ')' => depth = depth.saturating_sub(1),
                character if character.is_whitespace() && depth == 0 => {
                    if let Some(component_start) = start.take() {
                        components.push(&value[component_start..index]);
                    }
                }
                _ => {}
            }
        }
    }
    if let Some(component_start) = start {
        components.push(&value[component_start..]);
    }
    components
}
