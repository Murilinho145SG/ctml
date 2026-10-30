use crate::style::{BoxShadow, TextStyle};
use ctml_css::parse_signed_font_relative_length_with_viewport;

pub(super) fn apply_box_shadow(style: &mut TextStyle, value: &str) {
    let value = value.trim();
    if value.eq_ignore_ascii_case("none") {
        style.box_shadows.clear();
        return;
    }
    let Some(shadows) = split_list(value)
        .into_iter()
        .map(|value| parse_shadow(style, value))
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };
    style.box_shadows = shadows;
}

fn parse_shadow(style: &TextStyle, value: &str) -> Option<BoxShadow> {
    let mut lengths = Vec::new();
    let mut color = None;
    let mut has_color = false;
    let mut inset = false;
    for component in split_components(value) {
        if component.eq_ignore_ascii_case("inset") {
            if inset {
                return None;
            }
            inset = true;
        } else if let Some(parsed) =
            ctml_css::parse_color_with_profiles(component, &style.color_profiles)
        {
            if has_color {
                return None;
            }
            color = Some(parsed);
            has_color = true;
        } else if component.eq_ignore_ascii_case("currentcolor") {
            if has_color {
                return None;
            }
            color = None;
            has_color = true;
        } else {
            let length = ctml_css::parse_math_expression(component, |part| {
                parse_signed_font_relative_length_with_viewport(
                    part,
                    style.font_size,
                    style.line_height,
                    style.font_relative_units,
                    style.viewport_size,
                )
            })
            .or_else(|| {
                parse_signed_font_relative_length_with_viewport(
                    component,
                    style.font_size,
                    style.line_height,
                    style.font_relative_units,
                    style.viewport_size,
                )
            })?;
            lengths.push(length);
        }
    }
    if !(2..=4).contains(&lengths.len()) || (lengths.len() >= 3 && lengths[2] < 0.0) {
        return None;
    }
    Some(BoxShadow {
        offset_x: lengths[0],
        offset_y: lengths[1],
        blur: lengths.get(2).copied().unwrap_or_default(),
        spread: lengths.get(3).copied().unwrap_or_default(),
        color,
        inset,
    })
}

fn split_list(value: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(value[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(value[start..].trim());
    parts
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

pub(super) fn inherit_css_property(
    style: &mut TextStyle,
    inherited: &TextStyle,
    property: &str,
) -> bool {
    if property == "box-shadow" {
        style.box_shadows.clone_from(&inherited.box_shadows);
        true
    } else {
        false
    }
}

pub(super) fn initial_css_property(style: &mut TextStyle, property: &str) -> bool {
    if property == "box-shadow" {
        style.box_shadows.clear();
        true
    } else {
        false
    }
}
