const ROOT_FONT_SIZE: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontRelativeUnits {
    pub ex: f32,
    pub ch: f32,
    pub ic: f32,
    pub cap: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewportSize {
    pub width: f32,
    pub height: f32,
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

pub fn parse_font_relative_length_with_viewport(
    value: &str,
    relative_to: f32,
    line_height: f32,
    font_metrics: FontRelativeUnits,
    viewport: ViewportSize,
) -> Option<f32> {
    parse_viewport_relative_length(value, viewport)
        .or_else(|| parse_font_relative_length(value, relative_to, line_height, font_metrics))
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

pub fn parse_nonnegative_font_relative_length(
    value: &str,
    relative_to: f32,
    line_height: f32,
    font_metrics: FontRelativeUnits,
) -> Option<f32> {
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
    for unit in [
        "rem", "px", "in", "cm", "mm", "q", "pt", "pc", "em", "ex", "ch", "ic", "cap", "lh",
    ] {
        if let Some(number) = value.strip_suffix(unit)
            && number.trim().parse::<f32>().ok() == Some(0.0)
        {
            return Some(0.0);
        }
    }
    parse_font_relative_length(&value, relative_to, line_height, font_metrics)
}

pub fn parse_nonnegative_font_relative_length_with_viewport(
    value: &str,
    relative_to: f32,
    line_height: f32,
    font_metrics: FontRelativeUnits,
    viewport: ViewportSize,
) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    parse_viewport_relative_length(&value, viewport)
        .filter(|length| *length >= 0.0)
        .or_else(|| {
            parse_nonnegative_font_relative_length(&value, relative_to, line_height, font_metrics)
        })
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

pub fn parse_signed_font_relative_length(
    value: &str,
    relative_to: f32,
    line_height: f32,
    font_metrics: FontRelativeUnits,
) -> Option<f32> {
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
    if let Some(number) = value.strip_suffix("lh") {
        return number
            .trim()
            .parse::<f32>()
            .ok()
            .map(|number| number * line_height)
            .filter(|length| length.is_finite());
    }
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
                .filter(|length| length.is_finite());
        }
    }
    parse_signed_length_percentage(&value, relative_to)
}

pub fn parse_signed_font_relative_length_with_viewport(
    value: &str,
    relative_to: f32,
    line_height: f32,
    font_metrics: FontRelativeUnits,
    viewport: ViewportSize,
) -> Option<f32> {
    parse_viewport_relative_length(value, viewport).or_else(|| {
        parse_signed_font_relative_length(value, relative_to, line_height, font_metrics)
    })
}

fn parse_viewport_relative_length(value: &str, viewport: ViewportSize) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    let (number, unit) = [
        "svmin", "svmax", "lvmin", "lvmax", "dvmin", "dvmax", "svw", "svh", "svi", "svb", "lvw",
        "lvh", "lvi", "lvb", "dvw", "dvh", "dvi", "dvb", "vmin", "vmax", "vw", "vh", "vi", "vb",
    ]
    .into_iter()
    .find_map(|unit| value.strip_suffix(unit).map(|number| (number, unit)))?;
    let number = number.trim().parse::<f32>().ok()?;
    let base_unit = unit
        .strip_prefix("sv")
        .or_else(|| unit.strip_prefix("lv"))
        .or_else(|| unit.strip_prefix("dv"))
        .unwrap_or(unit);
    let width = viewport.width.max(0.0);
    let height = viewport.height.max(0.0);
    let viewport_size = match base_unit {
        "vw" | "vi" | "w" | "i" => width,
        "vh" | "vb" | "h" | "b" => height,
        "vmin" | "min" => width.min(height),
        "vmax" | "max" => width.max(height),
        _ => return None,
    };
    let length = number * viewport_size / 100.0;
    (length.is_finite()).then_some(length)
}

#[cfg(test)]
mod tests {
    use super::{
        FontRelativeUnits, ViewportSize, parse_font_relative_length,
        parse_font_relative_length_with_viewport, parse_length_with_line_height,
        parse_nonnegative_font_relative_length,
        parse_nonnegative_font_relative_length_with_viewport,
        parse_nonnegative_length_with_line_height, parse_signed_font_relative_length,
        parse_signed_font_relative_length_with_viewport,
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
        assert_eq!(
            parse_nonnegative_font_relative_length("0.5cap", 20.0, 24.0, metrics),
            Some(6.8)
        );
        assert_eq!(
            parse_signed_font_relative_length("-2ch", 20.0, 24.0, metrics),
            Some(-20.8)
        );
        assert_eq!(
            parse_signed_font_relative_length("-0.5lh", 20.0, 24.0, metrics),
            Some(-12.0)
        );
    }

    #[test]
    fn parses_viewport_relative_lengths() {
        let units = FontRelativeUnits::default();
        let viewport = ViewportSize {
            width: 800.0,
            height: 600.0,
        };
        for unit in ["vw", "svw", "lvw", "dvw", "vi", "svi", "lvi", "dvi"] {
            let value = format!("10{unit}");
            assert_eq!(
                parse_font_relative_length_with_viewport(&value, 16.0, 20.0, units, viewport),
                Some(80.0),
                "{unit}"
            );
        }
        for unit in ["vh", "svh", "lvh", "dvh", "vb", "svb", "lvb", "dvb"] {
            let value = format!("10{unit}");
            assert_eq!(
                parse_nonnegative_font_relative_length_with_viewport(
                    &value, 16.0, 20.0, units, viewport
                ),
                Some(60.0),
                "{unit}"
            );
        }
        for unit in ["vmin", "svmin", "lvmin", "dvmin"] {
            let value = format!("10{unit}");
            assert_eq!(
                parse_signed_font_relative_length_with_viewport(
                    &value, 16.0, 20.0, units, viewport
                ),
                Some(60.0),
                "{unit}"
            );
        }
        for unit in ["vmax", "svmax", "lvmax", "dvmax"] {
            let value = format!("10{unit}");
            assert_eq!(
                parse_font_relative_length_with_viewport(&value, 16.0, 20.0, units, viewport),
                Some(80.0),
                "{unit}"
            );
        }
        assert_eq!(
            parse_signed_font_relative_length_with_viewport("-5dvw", 16.0, 20.0, units, viewport),
            Some(-40.0)
        );
    }
}
