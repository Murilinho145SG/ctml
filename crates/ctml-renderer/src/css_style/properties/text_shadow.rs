use crate::style::{TextShadow, TextStyle};
use ctml_css::{
    parse_color_with_opacity, parse_nonnegative_length, parse_signed_length_percentage,
};

pub(super) fn apply_text_shadow(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("none") {
        style.text_shadows.clear();
        return;
    }
    let Some(shadows) = parse_text_shadows(value, style.font_size, style.color) else {
        return;
    };
    style.text_shadows = shadows;
}

fn parse_text_shadows(
    value: &str,
    font_size: f32,
    current_color: [u8; 4],
) -> Option<Vec<TextShadow>> {
    let mut shadows = Vec::new();
    for shadow in split_top_level_commas(value) {
        let components = split_components(shadow);
        if components.is_empty() {
            return None;
        }
        let mut offsets = Vec::new();
        let mut blur = None;
        let mut color = None;
        for component in components {
            if component.eq_ignore_ascii_case("currentcolor") {
                if color.replace(current_color).is_some() {
                    return None;
                }
            } else if let Some((channels, opacity)) = parse_color_with_opacity(component) {
                if color
                    .replace([
                        channels[0],
                        channels[1],
                        channels[2],
                        (opacity * 255.0).round() as u8,
                    ])
                    .is_some()
                {
                    return None;
                }
            } else if offsets.len() < 2 {
                let length = parse_shadow_length(component, font_size)?;
                offsets.push(length);
            } else if blur.is_none() {
                if component.contains('%') {
                    return None;
                }
                blur = Some(parse_nonnegative_length(component, font_size)?);
            } else {
                return None;
            }
        }
        if offsets.len() != 2 {
            return None;
        }
        shadows.push(TextShadow {
            offset_x: offsets[0],
            offset_y: offsets[1],
            blur: blur.unwrap_or(0.0),
            color: color.unwrap_or(current_color),
        });
    }
    (!shadows.is_empty()).then_some(shadows)
}

fn parse_shadow_length(value: &str, font_size: f32) -> Option<f32> {
    if value.contains('%') {
        return None;
    }
    parse_signed_length_percentage(value, font_size)
}

fn split_top_level_commas(value: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(value[start..index].trim());
                start = index + character.len_utf8();
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
        if start.is_none() && !character.is_whitespace() {
            start = Some(index);
        }
        match character {
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
    if let Some(component_start) = start {
        components.push(&value[component_start..]);
    }
    components
}

#[cfg(test)]
mod tests {
    use super::parse_text_shadows;

    #[test]
    fn parses_multiple_shadows_with_optional_blur_and_color() {
        assert_eq!(
            parse_text_shadows("1px -2px 3px rgb(255 0 0 / 50%), 0 1em blue", 16.0, [0; 4]),
            Some(vec![
                crate::style::TextShadow {
                    offset_x: 1.0,
                    offset_y: -2.0,
                    blur: 3.0,
                    color: [255, 0, 0, 128],
                },
                crate::style::TextShadow {
                    offset_x: 0.0,
                    offset_y: 16.0,
                    blur: 0.0,
                    color: [0, 0, 255, 255],
                },
            ])
        );
    }

    #[test]
    fn rejects_invalid_shadow_lists_atomically() {
        for value in [
            "1px",
            "1px 2px -1px",
            "1px 2px 3px 4px",
            "1% 2px red",
            "1px 2px red,",
        ] {
            assert_eq!(parse_text_shadows(value, 16.0, [0; 4]), None, "{value}");
        }
    }
}
