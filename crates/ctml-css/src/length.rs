const ROOT_FONT_SIZE: f32 = 16.0;

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
        parse_length_with_line_height, parse_nonnegative_length_with_line_height,
        parse_signed_length_percentage_with_line_height,
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
}
