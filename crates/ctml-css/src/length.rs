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
    if let Some(length) =
        crate::math::parse_math_expression(value, |component| parse_length(component, relative_to))
    {
        return (length >= 0.0).then_some(length);
    }
    let value = value.trim();
    match crate::numeric::parse_css_numeric_token(value)? {
        crate::numeric::CssNumericToken::Number(number) if number == 0.0 => Some(0.0),
        crate::numeric::CssNumericToken::Percentage(percentage) => {
            let length = relative_to * percentage / 100.0;
            (length.is_finite() && length >= 0.0).then_some(length)
        }
        crate::numeric::CssNumericToken::Dimension(number, unit) => {
            let pixels_per_unit = match unit.as_str() {
                "rem" => ROOT_FONT_SIZE,
                "px" => 1.0,
                "in" => 96.0,
                "cm" => 96.0 / 2.54,
                "mm" => 96.0 / 25.4,
                "q" => 96.0 / 101.6,
                "pt" => 96.0 / 72.0,
                "pc" => 16.0,
                "em" => relative_to,
                _ => return None,
            };
            let length = number * pixels_per_unit;
            (length.is_finite() && length >= 0.0).then_some(length)
        }
        _ => None,
    }
}

pub fn parse_length_with_line_height(
    value: &str,
    relative_to: f32,
    line_height: f32,
) -> Option<f32> {
    if let Some(length) = crate::math::parse_math_expression(value, |component| {
        parse_length_with_line_height(component, relative_to, line_height)
    }) {
        return (length >= 0.0).then_some(length);
    }
    if let Some(crate::numeric::CssNumericToken::Dimension(number, unit)) =
        crate::numeric::parse_css_numeric_token(value.trim())
        && unit == "lh"
    {
        let length = number * line_height;
        return (length.is_finite() && length >= 0.0).then_some(length);
    }
    parse_length(value, relative_to)
}

pub fn parse_length_with_font_metrics(
    value: &str,
    relative_to: f32,
    font_metrics: FontRelativeUnits,
) -> Option<f32> {
    if let Some(length) = crate::math::parse_math_expression(value, |component| {
        parse_length_with_font_metrics(component, relative_to, font_metrics)
    }) {
        return (length >= 0.0).then_some(length);
    }
    if let Some(crate::numeric::CssNumericToken::Dimension(number, unit)) =
        crate::numeric::parse_css_numeric_token(value.trim())
    {
        let factor = match unit.as_str() {
            "ex" => font_metrics.ex,
            "ch" => font_metrics.ch,
            "ic" => font_metrics.ic,
            "cap" => font_metrics.cap,
            _ => return parse_length(value, relative_to),
        };
        let length = number * factor * relative_to;
        return (length.is_finite() && length >= 0.0).then_some(length);
    }
    parse_length(value, relative_to)
}

pub fn parse_font_relative_length(
    value: &str,
    relative_to: f32,
    line_height: f32,
    font_metrics: FontRelativeUnits,
) -> Option<f32> {
    if let Some(length) = crate::math::parse_math_expression(value, |component| {
        parse_font_relative_length(component, relative_to, line_height, font_metrics)
    }) {
        return (length >= 0.0).then_some(length);
    }
    if let Some(crate::numeric::CssNumericToken::Dimension(number, unit)) =
        crate::numeric::parse_css_numeric_token(value.trim())
        && unit == "lh"
    {
        let length = number * line_height;
        return (length.is_finite() && length >= 0.0).then_some(length);
    }
    parse_length_with_font_metrics(value, relative_to, font_metrics)
}

pub fn parse_font_relative_length_with_viewport(
    value: &str,
    relative_to: f32,
    line_height: f32,
    font_metrics: FontRelativeUnits,
    viewport: ViewportSize,
) -> Option<f32> {
    if let Some(length) = crate::math::parse_math_expression(value, |component| {
        parse_viewport_relative_length(component, viewport).or_else(|| {
            parse_font_relative_length(component, relative_to, line_height, font_metrics)
        })
    }) {
        return (length >= 0.0).then_some(length);
    }
    parse_viewport_relative_length(value, viewport)
        .or_else(|| parse_font_relative_length(value, relative_to, line_height, font_metrics))
}

pub fn parse_nonnegative_length(value: &str, relative_to: f32) -> Option<f32> {
    if let Some(length) = crate::math::parse_math_expression(value, |component| {
        parse_nonnegative_length(component, relative_to)
    }) {
        return (length >= 0.0).then_some(length);
    }
    match crate::numeric::parse_css_numeric_token(value.trim())? {
        crate::numeric::CssNumericToken::Number(number) if number == 0.0 => Some(0.0),
        crate::numeric::CssNumericToken::Percentage(percentage) => {
            let length = relative_to * percentage / 100.0;
            (length.is_finite() && length >= 0.0).then_some(length)
        }
        crate::numeric::CssNumericToken::Dimension(number, unit)
            if number == 0.0
                && ["rem", "px", "in", "cm", "mm", "q", "pt", "pc", "em"]
                    .contains(&unit.as_str()) =>
        {
            Some(0.0)
        }
        _ => parse_length(value, relative_to),
    }
}

pub fn parse_nonnegative_length_with_line_height(
    value: &str,
    relative_to: f32,
    line_height: f32,
) -> Option<f32> {
    if let Some(length) = crate::math::parse_math_expression(value, |component| {
        parse_nonnegative_length_with_line_height(component, relative_to, line_height)
    }) {
        return (length >= 0.0).then_some(length);
    }
    if let Some(crate::numeric::CssNumericToken::Dimension(number, unit)) =
        crate::numeric::parse_css_numeric_token(value.trim())
        && unit == "lh"
    {
        let length = number * line_height;
        return (length.is_finite() && length >= 0.0).then_some(length);
    }
    parse_nonnegative_length(value, relative_to)
}

pub fn parse_nonnegative_font_relative_length(
    value: &str,
    relative_to: f32,
    line_height: f32,
    font_metrics: FontRelativeUnits,
) -> Option<f32> {
    if let Some(length) = crate::math::parse_math_expression(value, |component| {
        parse_nonnegative_font_relative_length(component, relative_to, line_height, font_metrics)
    }) {
        return (length >= 0.0).then_some(length);
    }
    match crate::numeric::parse_css_numeric_token(value.trim())? {
        crate::numeric::CssNumericToken::Number(number) if number == 0.0 => Some(0.0),
        crate::numeric::CssNumericToken::Percentage(percentage) => {
            let length = relative_to * percentage / 100.0;
            (length.is_finite() && length >= 0.0).then_some(length)
        }
        crate::numeric::CssNumericToken::Dimension(number, unit)
            if number == 0.0
                && [
                    "rem", "px", "in", "cm", "mm", "q", "pt", "pc", "em", "ex", "ch", "ic", "cap",
                    "lh",
                ]
                .contains(&unit.as_str()) =>
        {
            Some(0.0)
        }
        _ => parse_font_relative_length(value, relative_to, line_height, font_metrics),
    }
}

pub fn parse_nonnegative_font_relative_length_with_viewport(
    value: &str,
    relative_to: f32,
    line_height: f32,
    font_metrics: FontRelativeUnits,
    viewport: ViewportSize,
) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    if let Some(length) = crate::math::parse_math_expression(&value, |component| {
        parse_viewport_relative_length(component, viewport).or_else(|| {
            parse_nonnegative_font_relative_length(
                component,
                relative_to,
                line_height,
                font_metrics,
            )
        })
    }) {
        return (length >= 0.0).then_some(length);
    }
    parse_viewport_relative_length(&value, viewport)
        .filter(|length| *length >= 0.0)
        .or_else(|| {
            parse_nonnegative_font_relative_length(&value, relative_to, line_height, font_metrics)
        })
}

pub fn parse_signed_length_percentage(value: &str, relative_to: f32) -> Option<f32> {
    if let Some(length) = crate::math::parse_math_expression(value, |component| {
        parse_signed_length_percentage(component, relative_to)
    }) {
        return Some(length);
    }
    match crate::numeric::parse_css_numeric_token(value.trim())? {
        crate::numeric::CssNumericToken::Number(number) if number == 0.0 => Some(0.0),
        crate::numeric::CssNumericToken::Percentage(percentage) => {
            let length = relative_to * percentage / 100.0;
            length.is_finite().then_some(length)
        }
        crate::numeric::CssNumericToken::Dimension(number, unit) => {
            let pixels_per_unit = match unit.as_str() {
                "rem" => ROOT_FONT_SIZE,
                "px" => 1.0,
                "in" => 96.0,
                "cm" => 96.0 / 2.54,
                "mm" => 96.0 / 25.4,
                "q" => 96.0 / 101.6,
                "pt" => 96.0 / 72.0,
                "pc" => 16.0,
                "em" => relative_to,
                _ => return None,
            };
            let length = number * pixels_per_unit;
            length.is_finite().then_some(length)
        }
        _ => None,
    }
}

pub fn parse_signed_length_percentage_with_line_height(
    value: &str,
    relative_to: f32,
    line_height: f32,
) -> Option<f32> {
    if let Some(length) = crate::math::parse_math_expression(value, |component| {
        parse_signed_length_percentage_with_line_height(component, relative_to, line_height)
    }) {
        return Some(length);
    }
    if let Some(crate::numeric::CssNumericToken::Dimension(number, unit)) =
        crate::numeric::parse_css_numeric_token(value.trim())
        && unit == "lh"
    {
        let length = number * line_height;
        return length.is_finite().then_some(length);
    }
    parse_signed_length_percentage(value, relative_to)
}

pub fn parse_signed_font_relative_length(
    value: &str,
    relative_to: f32,
    line_height: f32,
    font_metrics: FontRelativeUnits,
) -> Option<f32> {
    if let Some(length) = crate::math::parse_math_expression(value, |component| {
        parse_signed_font_relative_length(component, relative_to, line_height, font_metrics)
    }) {
        return Some(length);
    }
    match crate::numeric::parse_css_numeric_token(value.trim())? {
        crate::numeric::CssNumericToken::Number(number) if number == 0.0 => Some(0.0),
        crate::numeric::CssNumericToken::Percentage(percentage) => {
            let length = relative_to * percentage / 100.0;
            length.is_finite().then_some(length)
        }
        crate::numeric::CssNumericToken::Dimension(number, unit) if unit == "lh" => {
            let length = number * line_height;
            length.is_finite().then_some(length)
        }
        crate::numeric::CssNumericToken::Dimension(number, unit) => {
            let factor = match unit.as_str() {
                "ex" => font_metrics.ex,
                "ch" => font_metrics.ch,
                "ic" => font_metrics.ic,
                "cap" => font_metrics.cap,
                _ => return parse_signed_length_percentage(value, relative_to),
            };
            let length = number * factor * relative_to;
            length.is_finite().then_some(length)
        }
        _ => None,
    }
}

pub fn parse_signed_font_relative_length_with_viewport(
    value: &str,
    relative_to: f32,
    line_height: f32,
    font_metrics: FontRelativeUnits,
    viewport: ViewportSize,
) -> Option<f32> {
    if let Some(length) = crate::math::parse_math_expression(value, |component| {
        parse_viewport_relative_length(component, viewport).or_else(|| {
            parse_signed_font_relative_length(component, relative_to, line_height, font_metrics)
        })
    }) {
        return Some(length);
    }
    parse_viewport_relative_length(value, viewport).or_else(|| {
        parse_signed_font_relative_length(value, relative_to, line_height, font_metrics)
    })
}

fn parse_viewport_relative_length(value: &str, viewport: ViewportSize) -> Option<f32> {
    let crate::numeric::CssNumericToken::Dimension(number, unit) =
        crate::numeric::parse_css_numeric_token(value.trim())?
    else {
        return None;
    };
    if ![
        "svmin", "svmax", "lvmin", "lvmax", "dvmin", "dvmax", "svw", "svh", "svi", "svb", "lvw",
        "lvh", "lvi", "lvb", "dvw", "dvh", "dvi", "dvb", "vmin", "vmax", "vw", "vh", "vi", "vb",
    ]
    .contains(&unit.as_str())
    {
        return None;
    }
    let base_unit = unit
        .strip_prefix("sv")
        .or_else(|| unit.strip_prefix("lv"))
        .or_else(|| unit.strip_prefix("dv"))
        .unwrap_or(&unit);
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
#[path = "length_tests.rs"]
mod tests;
