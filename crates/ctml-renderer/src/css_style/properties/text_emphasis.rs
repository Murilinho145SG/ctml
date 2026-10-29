use crate::style::{TextEmphasisPosition, TextEmphasisStyle, TextStyle};
use ctml_css::parse_color_with_opacity;

pub(super) fn apply_text_emphasis_shorthand(style: &mut TextStyle, value: &str) {
    let components = split_components(value);
    if components.is_empty() {
        return;
    }
    if components.len() == 1 && components[0].eq_ignore_ascii_case("none") {
        style.text_emphasis_style = TextEmphasisStyle::None;
        style.text_emphasis_color = None;
        return;
    }

    let mut style_components = Vec::new();
    let mut color = None;
    for component in components {
        if component.eq_ignore_ascii_case("currentcolor") {
            if color.replace(None).is_some() {
                return;
            }
        } else if let Some((channels, opacity)) = parse_color_with_opacity(component) {
            if color
                .replace(Some([
                    channels[0],
                    channels[1],
                    channels[2],
                    (opacity * 255.0).round() as u8,
                ]))
                .is_some()
            {
                return;
            }
        } else {
            style_components.push(component);
        }
    }
    let emphasis_style = if style_components.is_empty() {
        TextEmphasisStyle::None
    } else if let Some(emphasis_style) = parse_emphasis_style(&style_components.join(" ")) {
        emphasis_style
    } else {
        return;
    };
    style.text_emphasis_style = emphasis_style;
    style.text_emphasis_color = color.flatten();
}

pub(super) fn apply_text_emphasis_style(style: &mut TextStyle, value: &str) {
    if let Some(emphasis_style) = parse_emphasis_style(value) {
        style.text_emphasis_style = emphasis_style;
    }
}

pub(super) fn apply_text_emphasis_color(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("currentcolor") {
        style.text_emphasis_color = None;
    } else if let Some((channels, opacity)) = parse_color_with_opacity(value) {
        style.text_emphasis_color = Some([
            channels[0],
            channels[1],
            channels[2],
            (opacity * 255.0).round() as u8,
        ]);
    }
}

pub(super) fn apply_text_emphasis_position(style: &mut TextStyle, value: &str) {
    let values = value.split_ascii_whitespace().collect::<Vec<_>>();
    if values.is_empty() || values.len() > 2 {
        return;
    }
    let position = match values[0].to_ascii_lowercase().as_str() {
        "over" => TextEmphasisPosition::Over,
        "under" => TextEmphasisPosition::Under,
        _ => return,
    };
    if values
        .get(1)
        .is_some_and(|side| !matches!(side.to_ascii_lowercase().as_str(), "left" | "right"))
    {
        return;
    }
    style.text_emphasis_position = position;
}

pub(super) fn apply_text_emphasis_property(style: &mut TextStyle, property: &str, value: &str) {
    match property {
        "text-emphasis" => apply_text_emphasis_shorthand(style, value),
        "text-emphasis-style" => apply_text_emphasis_style(style, value),
        "text-emphasis-color" => apply_text_emphasis_color(style, value),
        "text-emphasis-position" => apply_text_emphasis_position(style, value),
        _ => {}
    }
}

pub(super) fn inherit_text_emphasis(style: &mut TextStyle, inherited: &TextStyle, property: &str) {
    match property {
        "text-emphasis" => {
            style.text_emphasis_style = inherited.text_emphasis_style;
            style.text_emphasis_color = inherited.text_emphasis_color;
        }
        "text-emphasis-style" => style.text_emphasis_style = inherited.text_emphasis_style,
        "text-emphasis-color" => style.text_emphasis_color = inherited.text_emphasis_color,
        "text-emphasis-position" => {
            style.text_emphasis_position = inherited.text_emphasis_position;
        }
        _ => {}
    }
}

pub(super) fn reset_text_emphasis(style: &mut TextStyle, property: &str) {
    match property {
        "text-emphasis" => {
            style.text_emphasis_style = TextEmphasisStyle::None;
            style.text_emphasis_color = None;
        }
        "text-emphasis-style" => style.text_emphasis_style = TextEmphasisStyle::None,
        "text-emphasis-color" => style.text_emphasis_color = None,
        "text-emphasis-position" => style.text_emphasis_position = TextEmphasisPosition::Over,
        _ => {}
    }
}

fn parse_emphasis_style(value: &str) -> Option<TextEmphasisStyle> {
    if value.eq_ignore_ascii_case("none") {
        return Some(TextEmphasisStyle::None);
    }
    let components = value.split_ascii_whitespace().collect::<Vec<_>>();
    if components.len() > 2 || components.is_empty() {
        return None;
    }
    let mut fill = true;
    let mut fill_seen = false;
    let mut shape = None;
    for component in components {
        match component.to_ascii_lowercase().as_str() {
            "filled" if !fill_seen => {
                fill = true;
                fill_seen = true;
            }
            "open" if !fill_seen => {
                fill = false;
                fill_seen = true;
            }
            "dot" if shape.is_none() => shape = Some("dot"),
            "circle" if shape.is_none() => shape = Some("circle"),
            "double-circle" if shape.is_none() => shape = Some("double-circle"),
            "triangle" if shape.is_none() => shape = Some("triangle"),
            "sesame" if shape.is_none() => shape = Some("sesame"),
            _ => return None,
        }
    }
    Some(match (shape.unwrap_or("circle"), fill) {
        ("dot", true) => TextEmphasisStyle::FilledDot,
        ("dot", false) => TextEmphasisStyle::OpenDot,
        ("circle", true) => TextEmphasisStyle::FilledCircle,
        ("circle", false) => TextEmphasisStyle::OpenCircle,
        ("double-circle", true) => TextEmphasisStyle::FilledDoubleCircle,
        ("double-circle", false) => TextEmphasisStyle::OpenDoubleCircle,
        ("triangle", true) => TextEmphasisStyle::FilledTriangle,
        ("triangle", false) => TextEmphasisStyle::OpenTriangle,
        ("sesame", true) => TextEmphasisStyle::FilledSesame,
        ("sesame", false) => TextEmphasisStyle::OpenSesame,
        _ => return None,
    })
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
    use super::{apply_text_emphasis_shorthand, apply_text_emphasis_style, parse_emphasis_style};
    use crate::style::{TextEmphasisStyle, TextStyle};

    #[test]
    fn parses_shape_fill_and_color_in_the_shorthand() {
        let mut style = TextStyle::default();
        apply_text_emphasis_shorthand(&mut style, "open sesame rgb(255 0 0 / 50%)");
        assert_eq!(style.text_emphasis_style, TextEmphasisStyle::OpenSesame);
        assert_eq!(style.text_emphasis_color, Some([255, 0, 0, 128]));

        apply_text_emphasis_shorthand(&mut style, "dot");
        assert_eq!(style.text_emphasis_style, TextEmphasisStyle::FilledDot);
        assert_eq!(style.text_emphasis_color, None);

        apply_text_emphasis_shorthand(&mut style, "open double-circle");
        assert_eq!(
            style.text_emphasis_style,
            TextEmphasisStyle::OpenDoubleCircle
        );

        apply_text_emphasis_shorthand(&mut style, "red");
        assert_eq!(style.text_emphasis_style, TextEmphasisStyle::None);
        assert_eq!(style.text_emphasis_color, Some([255, 0, 0, 255]));
    }

    #[test]
    fn rejects_conflicting_or_unsupported_style_values() {
        let mut style = TextStyle::default();
        style.text_emphasis_style = TextEmphasisStyle::OpenCircle;
        for value in ["filled open", "dot triangle", "custom", "none dot"] {
            apply_text_emphasis_style(&mut style, value);
            assert_eq!(style.text_emphasis_style, TextEmphasisStyle::OpenCircle);
        }
    }

    #[test]
    fn accepts_all_keyword_mark_shapes_and_default_fill() {
        let cases = [
            ("filled", TextEmphasisStyle::FilledCircle),
            ("open", TextEmphasisStyle::OpenCircle),
            ("filled dot", TextEmphasisStyle::FilledDot),
            ("open dot", TextEmphasisStyle::OpenDot),
            ("circle", TextEmphasisStyle::FilledCircle),
            ("open circle", TextEmphasisStyle::OpenCircle),
            ("double-circle", TextEmphasisStyle::FilledDoubleCircle),
            ("open double-circle", TextEmphasisStyle::OpenDoubleCircle),
            ("triangle", TextEmphasisStyle::FilledTriangle),
            ("open triangle", TextEmphasisStyle::OpenTriangle),
            ("sesame", TextEmphasisStyle::FilledSesame),
            ("open sesame", TextEmphasisStyle::OpenSesame),
        ];
        for (value, expected) in cases {
            assert_eq!(parse_emphasis_style(value), Some(expected), "{value}");
        }
        assert_eq!(parse_emphasis_style("none"), Some(TextEmphasisStyle::None));
    }
}
