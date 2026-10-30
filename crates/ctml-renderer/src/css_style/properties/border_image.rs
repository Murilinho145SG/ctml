use crate::style::{
    BorderImageOutset, BorderImageRepeat, BorderImageSlice, BorderImageWidth, TextStyle,
};

pub(super) fn apply_border_image_source(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("none") {
        style.border_image_source = None;
    } else if let Some(source) = super::background::parse_data_url(value) {
        style.border_image_source = Some(source);
    }
}

pub(super) fn apply_border_image_shorthand(style: &mut TextStyle, value: &str) {
    let parts = split_top_level_slashes(value);
    if parts.is_empty() || parts.len() > 3 {
        return;
    }
    let tokens = parts
        .iter()
        .map(|part| part.split_ascii_whitespace().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let mut source = None;
    let mut source_seen = false;
    let mut slices = Vec::new();
    let mut fill = false;
    let mut repeat_values = Vec::new();
    for token in &tokens[0] {
        if parse_shorthand_source(token, &mut source_seen, &mut source).is_none() {
            return;
        } else if is_shorthand_source(token) {
            continue;
        } else if token.eq_ignore_ascii_case("fill") {
            if fill {
                return;
            }
            fill = true;
        } else if let Some(repeat) = parse_repeat(token) {
            repeat_values.push(repeat);
        } else if let Some(slice) = parse_slice(token) {
            slices.push(slice);
        } else {
            return;
        }
    }
    let slice_values = if slices.is_empty() {
        [BorderImageSlice::default(); 4]
    } else {
        let Some(slices) = expand_four(&slices) else {
            return;
        };
        slices
    };
    let widths = if let Some(part) = tokens.get(1) {
        let mut values = Vec::new();
        for token in part {
            if parse_shorthand_source(token, &mut source_seen, &mut source).is_none() {
                return;
            } else if is_shorthand_source(token) {
                continue;
            } else if let Some(repeat) = parse_repeat(token) {
                repeat_values.push(repeat);
            } else if let Some(width) = parse_width(style, token) {
                values.push(width);
            } else {
                return;
            }
        }
        if values.is_empty() {
            [None; 4]
        } else {
            let Some(values) = expand_four(&values) else {
                return;
            };
            values.map(Some)
        }
    } else {
        [None; 4]
    };
    let outsets = if let Some(part) = tokens.get(2) {
        let mut values = Vec::new();
        for token in part {
            if parse_shorthand_source(token, &mut source_seen, &mut source).is_none() {
                return;
            } else if is_shorthand_source(token) {
                continue;
            } else if let Some(repeat) = parse_repeat(token) {
                repeat_values.push(repeat);
            } else if let Some(outset) = parse_outset(style, token) {
                values.push(outset);
            } else {
                return;
            }
        }
        if values.is_empty() {
            [None; 4]
        } else {
            let Some(values) = expand_four(&values) else {
                return;
            };
            values.map(Some)
        }
    } else {
        [None; 4]
    };
    let repeats = match repeat_values.as_slice() {
        [] => [BorderImageRepeat::Stretch; 2],
        [one] => [*one, *one],
        [horizontal, vertical] => [*horizontal, *vertical],
        _ => return,
    };
    style.border_image_source = source;
    style.border_image_slice = slice_values;
    style.border_image_fill = fill;
    style.border_image_width = widths;
    style.border_image_outset = outsets;
    style.border_image_repeat = repeats;
}

fn parse_shorthand_source(
    token: &str,
    source_seen: &mut bool,
    source: &mut Option<std::sync::Arc<[u8]>>,
) -> Option<()> {
    if !is_shorthand_source(token) {
        return Some(());
    }
    if *source_seen {
        return None;
    }
    *source_seen = true;
    if token.eq_ignore_ascii_case("none") {
        *source = None;
        return Some(());
    }
    *source = Some(super::background::parse_data_url(token)?);
    Some(())
}

fn is_shorthand_source(token: &str) -> bool {
    token.eq_ignore_ascii_case("none") || token.to_ascii_lowercase().starts_with("url(")
}

pub(super) fn apply_border_image_slice(style: &mut TextStyle, value: &str) {
    let mut values = Vec::new();
    let mut fill = false;
    for token in value.split_ascii_whitespace() {
        if token.eq_ignore_ascii_case("fill") {
            if fill {
                return;
            }
            fill = true;
        } else if let Some(slice) = parse_slice(token) {
            values.push(slice);
        } else {
            return;
        }
    }
    if values.is_empty() || values.len() > 4 {
        return;
    }
    let expanded = match values.as_slice() {
        [top] => [*top, *top, *top, *top],
        [top, right] => [*top, *right, *top, *right],
        [top, right, bottom] => [*top, *right, *bottom, *right],
        [top, right, bottom, left] => [*top, *right, *bottom, *left],
        _ => return,
    };
    style.border_image_slice = expanded;
    style.border_image_fill = fill;
}

pub(super) fn apply_border_image_repeat(style: &mut TextStyle, value: &str) {
    let repeats = value
        .split_ascii_whitespace()
        .map(parse_repeat)
        .collect::<Option<Vec<_>>>();
    let Some(repeats) = repeats else {
        return;
    };
    style.border_image_repeat = match repeats.as_slice() {
        [one] => [*one, *one],
        [horizontal, vertical] => [*horizontal, *vertical],
        _ => return,
    };
}

pub(super) fn apply_border_image_width(style: &mut TextStyle, value: &str) {
    let widths = value
        .split_ascii_whitespace()
        .map(|value| parse_width(style, value))
        .collect::<Option<Vec<_>>>();
    let Some(widths) = widths else {
        return;
    };
    style.border_image_width = match widths.as_slice() {
        [one] => [Some(*one); 4],
        [vertical, horizontal] => [
            Some(*vertical),
            Some(*horizontal),
            Some(*vertical),
            Some(*horizontal),
        ],
        [top, horizontal, bottom] => [
            Some(*top),
            Some(*horizontal),
            Some(*bottom),
            Some(*horizontal),
        ],
        [top, right, bottom, left] => [Some(*top), Some(*right), Some(*bottom), Some(*left)],
        _ => return,
    };
}

pub(super) fn apply_border_image_outset(style: &mut TextStyle, value: &str) {
    let outsets = value
        .split_ascii_whitespace()
        .map(|value| parse_outset(style, value))
        .collect::<Option<Vec<_>>>();
    let Some(outsets) = outsets else {
        return;
    };
    style.border_image_outset = match outsets.as_slice() {
        [one] => [Some(*one); 4],
        [vertical, horizontal] => [
            Some(*vertical),
            Some(*horizontal),
            Some(*vertical),
            Some(*horizontal),
        ],
        [top, horizontal, bottom] => [
            Some(*top),
            Some(*horizontal),
            Some(*bottom),
            Some(*horizontal),
        ],
        [top, right, bottom, left] => [Some(*top), Some(*right), Some(*bottom), Some(*left)],
        _ => return,
    };
}

fn parse_outset(style: &TextStyle, value: &str) -> Option<BorderImageOutset> {
    if let Some(number) = ctml_css::parse_css_number(value) {
        return (number >= 0.0 && number.is_finite()).then_some(BorderImageOutset::Number(number));
    }
    ctml_css::parse_nonnegative_font_relative_length_with_viewport(
        value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    )
    .filter(|length| length.is_finite())
    .map(BorderImageOutset::Length)
}

fn parse_width(style: &TextStyle, value: &str) -> Option<BorderImageWidth> {
    if value.eq_ignore_ascii_case("auto") {
        return Some(BorderImageWidth::Auto);
    }
    if let Some(percentage) = ctml_css::parse_css_percentage(value) {
        return (percentage >= 0.0).then_some(BorderImageWidth::Percentage(percentage / 100.0));
    }
    if let Some(number) = ctml_css::parse_css_number(value) {
        return (number >= 0.0 && number.is_finite()).then_some(BorderImageWidth::Number(number));
    }
    ctml_css::parse_nonnegative_font_relative_length_with_viewport(
        value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    )
    .filter(|length| length.is_finite())
    .map(BorderImageWidth::Length)
}

fn parse_slice(value: &str) -> Option<BorderImageSlice> {
    if let Some(percentage) = ctml_css::parse_css_percentage(value) {
        return (percentage >= 0.0).then_some(BorderImageSlice {
            value: percentage / 100.0,
            percentage: true,
        });
    }
    ctml_css::parse_css_number(value)
        .filter(|number| *number >= 0.0 && number.is_finite())
        .map(|value| BorderImageSlice {
            value,
            percentage: false,
        })
}

fn parse_repeat(value: &str) -> Option<BorderImageRepeat> {
    match value.to_ascii_lowercase().as_str() {
        "stretch" => Some(BorderImageRepeat::Stretch),
        "repeat" => Some(BorderImageRepeat::Repeat),
        "round" => Some(BorderImageRepeat::Round),
        "space" => Some(BorderImageRepeat::Space),
        _ => None,
    }
}

fn expand_four<T: Copy>(values: &[T]) -> Option<[T; 4]> {
    match values {
        [one] => Some([*one; 4]),
        [vertical, horizontal] => Some([*vertical, *horizontal, *vertical, *horizontal]),
        [top, horizontal, bottom] => Some([*top, *horizontal, *bottom, *horizontal]),
        [top, right, bottom, left] => Some([*top, *right, *bottom, *left]),
        _ => None,
    }
}

fn split_top_level_slashes(value: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut quote = None;
    let mut start = 0;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            '/' if depth == 0 => {
                parts.push(value[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(value[start..].trim());
    parts
}
