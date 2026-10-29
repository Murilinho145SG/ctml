const ROOT_FONT_SIZE: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontRelativeUnits {
    pub ex: f32,
    pub ch: f32,
    pub ic: f32,
    pub cap: f32,
}

impl Default for FontRelativeUnits {
    fn default() -> Self {
        Self {
            ex: 0.5,
            ch: 0.5,
            ic: 1.0,
            cap: 0.7,
        }
    }
}

pub fn parse_length(value: &str, relative_to: f32) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    if let Some(percentage) = value.strip_suffix('%') {
        return percentage
            .trim()
            .parse::<f32>()
            .ok()
            .map(|percentage| relative_to * percentage / 100.0)
            .filter(|value| value.is_finite() && *value > 0.0);
    }
    for (unit, pixels_per_unit) in [
        ("rem", ROOT_FONT_SIZE),
        ("px", 1.0),
        ("in", 96.0),
        ("cm", 96.0 / 2.54),
        ("mm", 96.0 / 25.4),
        ("q", 96.0 / 101.6),
        ("pt", 96.0 / 72.0),
        ("pc", 16.0),
        ("em", relative_to),
    ] {
        if let Some(number) = value.strip_suffix(unit) {
            return number
                .trim()
                .parse::<f32>()
                .ok()
                .map(|number| number * pixels_per_unit)
                .filter(|value| value.is_finite() && *value > 0.0);
        }
    }
    None
}

pub fn parse_length_with_line_height(
    value: &str,
    relative_to: f32,
    line_height: f32,
) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    if let Some(number) = value.strip_suffix("lh") {
        return number
            .trim()
            .parse::<f32>()
            .ok()
            .map(|number| number * line_height)
            .filter(|length| length.is_finite() && *length > 0.0);
    }
    parse_length(&value, relative_to)
}

pub fn parse_length_with_font_metrics(
    value: &str,
    relative_to: f32,
    font_metrics: FontRelativeUnits,
) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    for (unit, factor) in [
        ("ex", font_metrics.ex),
        ("ch", font_metrics.ch),
        ("ic", font_metrics.ic),
        ("cap", font_metrics.cap),
    ] {
        if let Some(number) = value.strip_suffix(unit) {
            return number
                .trim()
                .parse::<f32>()
                .ok()
                .map(|number| number * factor * relative_to)
                .filter(|length| length.is_finite() && *length > 0.0);
        }
    }
    parse_length(&value, relative_to)
}

pub fn parse_font_relative_length(
    value: &str,
    relative_to: f32,
    line_height: f32,
    font_metrics: FontRelativeUnits,
) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    if let Some(number) = value.strip_suffix("lh") {
        return number
            .trim()
            .parse::<f32>()
            .ok()
            .map(|number| number * line_height)
            .filter(|length| length.is_finite() && *length > 0.0);
    }
    parse_length_with_font_metrics(&value, relative_to, font_metrics)
}

pub fn parse_nonnegative_length(value: &str, relative_to: f32) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    if value.parse::<f32>().ok() == Some(0.0) {
        return Some(0.0);
    }
    if let Some(percentage) = value.strip_suffix('%') {
        return percentage
            .trim()
            .parse::<f32>()
            .ok()
            .map(|percentage| relative_to * percentage / 100.0)
            .filter(|length| length.is_finite() && *length >= 0.0);
    }
    for unit in ["rem", "px", "in", "cm", "mm", "q", "pt", "pc", "em"] {
        if let Some(number) = value.strip_suffix(unit)
            && number.trim().parse::<f32>().ok() == Some(0.0)
        {
            return Some(0.0);
        }
    }
    parse_length(&value, relative_to)
}

pub fn parse_nonnegative_length_with_line_height(
    value: &str,
    relative_to: f32,
    line_height: f32,
) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    if let Some(number) = value.strip_suffix("lh") {
        return number
            .trim()
            .parse::<f32>()
            .ok()
            .map(|number| number * line_height)
            .filter(|length| length.is_finite() && *length >= 0.0);
    }
    parse_nonnegative_length(&value, relative_to)
}

pub fn parse_signed_length_percentage(value: &str, relative_to: f32) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    if value.parse::<f32>().ok() == Some(0.0) {
        return Some(0.0);
    }
    if let Some(percentage) = value.strip_suffix('%') {
        return percentage
            .trim()
            .parse::<f32>()
            .ok()
            .map(|percentage| relative_to * percentage / 100.0)
            .filter(|length| length.is_finite());
    }
    for (unit, pixels_per_unit) in [
        ("rem", ROOT_FONT_SIZE),
        ("px", 1.0),
        ("in", 96.0),
        ("cm", 96.0 / 2.54),
        ("mm", 96.0 / 25.4),
        ("q", 96.0 / 101.6),
        ("pt", 96.0 / 72.0),
        ("pc", 16.0),
        ("em", relative_to),
    ] {
        if let Some(number) = value.strip_suffix(unit) {
            return number
                .trim()
                .parse::<f32>()
                .ok()
                .map(|number| number * pixels_per_unit)
                .filter(|length| length.is_finite());
        }
    }
    None
}

pub fn parse_signed_length_percentage_with_line_height(
    value: &str,
    relative_to: f32,
    line_height: f32,
) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    if let Some(number) = value.strip_suffix("lh") {
        return number
            .trim()
            .parse::<f32>()
            .ok()
            .map(|number| number * line_height)
            .filter(|length| length.is_finite());
    }
    parse_signed_length_percentage(&value, relative_to)
}

#[cfg(test)]
mod tests {
    use super::{
        FontRelativeUnits, parse_font_relative_length, parse_length_with_line_height,
        parse_nonnegative_length_with_line_height, parse_signed_length_percentage_with_line_height,
    };

    #[test]
    fn parses_lh_relative_lengths_and_percentages() {
        assert_eq!(parse_length_with_line_height("2lh", 16.0, 24.0), Some(48.0));
        assert_eq!(
            parse_nonnegative_length_with_line_height("0lh", 16.0, 24.0),
            Some(0.0)
        );
        assert_eq!(
            parse_signed_length_percentage_with_line_height("-0.5lh", 16.0, 24.0),
            Some(-12.0)
        );
        assert_eq!(parse_length_with_line_height("2em", 16.0, 24.0), Some(32.0));
    }

    #[test]
    fn parses_font_metric_relative_units() {
        let metrics = FontRelativeUnits {
            ex: 0.45,
            ch: 0.52,
            ic: 1.0,
            cap: 0.68,
        };
        assert_eq!(
            parse_font_relative_length("2ex", 20.0, 24.0, metrics),
            Some(18.0)
        );
        assert_eq!(
            parse_font_relative_length("2ch", 20.0, 24.0, metrics),
            Some(20.8)
        );
        assert_eq!(
            parse_font_relative_length("2ic", 20.0, 24.0, metrics),
            Some(40.0)
        );
        assert_eq!(
            parse_font_relative_length("2cap", 20.0, 24.0, metrics),
            Some(27.2)
        );
        assert_eq!(
            parse_font_relative_length("2lh", 20.0, 24.0, metrics),
            Some(48.0)
        );
    }
}
