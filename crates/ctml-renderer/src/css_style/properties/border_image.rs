use crate::style::{BorderImageRepeat, BorderImageSlice, TextStyle};

pub(super) fn apply_border_image_source(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("none") {
        style.border_image_source = None;
    } else if let Some(source) = super::background::parse_data_url(value) {
        style.border_image_source = Some(source);
    }
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
        .map(|value| match value.to_ascii_lowercase().as_str() {
            "stretch" => Some(BorderImageRepeat::Stretch),
            "repeat" => Some(BorderImageRepeat::Repeat),
            "round" => Some(BorderImageRepeat::Round),
            "space" => Some(BorderImageRepeat::Space),
            _ => None,
        })
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
