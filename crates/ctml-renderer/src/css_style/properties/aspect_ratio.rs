use crate::style::TextStyle;

pub(super) fn apply_aspect_ratio(style: &mut TextStyle, value: &str) {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        style.aspect_ratio = None;
        return;
    }
    let parts = value.split('/').map(str::trim).collect::<Vec<_>>();
    let ratio = match parts.as_slice() {
        [numerator] => parse_positive_number(numerator),
        [numerator, denominator] => parse_positive_number(numerator).and_then(|numerator| {
            parse_positive_number(denominator).map(|denominator| numerator / denominator)
        }),
        _ => None,
    };
    if let Some(ratio) = ratio.filter(|ratio| ratio.is_finite() && *ratio > 0.0) {
        style.aspect_ratio = Some(ratio);
    }
}

fn parse_positive_number(value: &str) -> Option<f32> {
    value
        .parse::<f32>()
        .ok()
        .filter(|number| number.is_finite() && *number > 0.0)
}

pub(super) fn inherit_css_property(
    style: &mut TextStyle,
    inherited: &TextStyle,
    property: &str,
) -> bool {
    if property == "aspect-ratio" {
        style.aspect_ratio = inherited.aspect_ratio;
        true
    } else {
        false
    }
}

pub(super) fn initial_css_property(style: &mut TextStyle, property: &str) -> bool {
    if property == "aspect-ratio" {
        style.aspect_ratio = None;
        true
    } else {
        false
    }
}
