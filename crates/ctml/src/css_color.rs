use cssparser::{Parser, ParserInput, Token};

pub(super) fn parse_color(value: &str) -> Option<[u8; 4]> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("transparent") {
        return Some([0, 0, 0, 0]);
    }
    if let Some(hex) = value.strip_prefix('#') {
        let (red, green, blue, alpha) = cssparser::color::parse_hash_color(hex.as_bytes()).ok()?;
        return Some([red, green, blue, (alpha * 255.0).round() as u8]);
    }

    if let Ok((red, green, blue)) = cssparser::color::parse_named_color(&value.to_ascii_lowercase())
    {
        return Some([red, green, blue, 255]);
    }

    parse_function_color(value)
}

fn parse_function_color(value: &str) -> Option<[u8; 4]> {
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    let function = parser.expect_function().ok()?.to_ascii_lowercase();
    let is_hsl = matches!(function.as_str(), "hsl" | "hsla");
    if !is_hsl && !matches!(function.as_str(), "rgb" | "rgba") {
        return None;
    }

    let color = parser
        .parse_nested_block(|input| {
            let red_or_hue = if is_hsl {
                parse_hue(input)?
            } else {
                parse_rgb_component(input)?
            };
            input.expect_comma()?;
            let green_or_saturation = if is_hsl {
                parse_unit_component(input)?
            } else {
                parse_component(input)?
            };
            input.expect_comma()?;
            let blue_or_lightness = if is_hsl {
                parse_unit_component(input)?
            } else {
                parse_component(input)?
            };

            let alpha = if input.is_exhausted() {
                255
            } else {
                input.expect_comma()?;
                parse_alpha(input)?
            };
            if !input.is_exhausted() {
                return Err(input.new_custom_error(()));
            }

            let (red, green, blue) = if is_hsl {
                let [red, green, blue] =
                    hsl_to_rgb(red_or_hue, green_or_saturation, blue_or_lightness);
                (red, green, blue)
            } else {
                (
                    red_or_hue as u8,
                    component_to_byte(green_or_saturation),
                    component_to_byte(blue_or_lightness),
                )
            };
            Ok([red, green, blue, alpha])
        })
        .ok()?;

    parser.is_exhausted().then_some(color)
}

fn parse_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    match input.next()? {
        Token::Number { value, .. } if value.is_finite() => Ok(*value),
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => Ok(*unit_value * 255.0),
        _ => Err(input.new_custom_error(())),
    }
}

fn parse_rgb_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    parse_component(input).map(|component| component.clamp(0.0, 255.0).round())
}

fn parse_unit_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    let value = input.expect_percentage()?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(input.new_custom_error(()))
    }
}

fn parse_hue<'i, 't>(input: &mut Parser<'i, 't>) -> Result<f32, cssparser::ParseError<'i, ()>> {
    match input.next()? {
        Token::Number { value, .. } if value.is_finite() => Ok(*value),
        Token::Dimension { value, unit, .. }
            if value.is_finite() && unit.eq_ignore_ascii_case("deg") =>
        {
            Ok(*value)
        }
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("grad") => {
            if value.is_finite() {
                Ok(*value * 0.9)
            } else {
                Err(input.new_custom_error(()))
            }
        }
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("rad") => {
            if value.is_finite() {
                Ok(value.to_degrees())
            } else {
                Err(input.new_custom_error(()))
            }
        }
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("turn") => {
            if value.is_finite() {
                Ok(*value * 360.0)
            } else {
                Err(input.new_custom_error(()))
            }
        }
        _ => Err(input.new_custom_error(())),
    }
}

fn parse_alpha<'i, 't>(input: &mut Parser<'i, 't>) -> Result<u8, cssparser::ParseError<'i, ()>> {
    let alpha = match input.next()? {
        Token::Number { value, .. } if value.is_finite() => *value,
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => *unit_value,
        _ => return Err(input.new_custom_error(())),
    };
    Ok((alpha.clamp(0.0, 1.0) * 255.0).round() as u8)
}

fn component_to_byte(component: f32) -> u8 {
    component.clamp(0.0, 255.0).round() as u8
}

fn hsl_to_rgb(hue: f32, saturation: f32, lightness: f32) -> [u8; 3] {
    let hue = hue.rem_euclid(360.0) / 60.0;
    let saturation = saturation.clamp(0.0, 1.0);
    let lightness = lightness.clamp(0.0, 1.0);
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let secondary = chroma * (1.0 - (hue.rem_euclid(2.0) - 1.0).abs());
    let offset = lightness - chroma / 2.0;
    let channels = match hue as u8 {
        0 => [chroma, secondary, 0.0],
        1 => [secondary, chroma, 0.0],
        2 => [0.0, chroma, secondary],
        3 => [0.0, secondary, chroma],
        4 => [secondary, 0.0, chroma],
        _ => [chroma, 0.0, secondary],
    };
    channels.map(|channel| ((channel + offset) * 255.0).round() as u8)
}

#[cfg(test)]
mod tests {
    use super::parse_color;

    #[test]
    fn parses_short_and_long_hex_alpha_colors() {
        assert_eq!(parse_color("#f008"), Some([255, 0, 0, 136]));
        assert_eq!(parse_color("#33669980"), Some([51, 102, 153, 128]));
    }

    #[test]
    fn parses_transparent_keyword() {
        assert_eq!(parse_color("transparent"), Some([0, 0, 0, 0]));
        assert_eq!(parse_color("TRANSPARENT"), Some([0, 0, 0, 0]));
    }

    #[test]
    fn parses_rgb_and_rgba_functions() {
        assert_eq!(parse_color("rgb(25%, 0, 255)"), Some([64, 0, 255, 255]));
        assert_eq!(
            parse_color("rgba(10, 20, 30, 50%)"),
            Some([10, 20, 30, 128])
        );
    }

    #[test]
    fn parses_hsl_angles_and_alpha() {
        assert_eq!(
            parse_color("hsl(120deg, 100%, 25%)"),
            Some([0, 128, 0, 255])
        );
        assert_eq!(
            parse_color("hsla(0.5turn, 100%, 50%, 0.25)"),
            Some([0, 255, 255, 64])
        );
    }

    #[test]
    fn rejects_unsupported_or_malformed_color_syntax() {
        assert_eq!(parse_color("hwb(120 20% 30%)"), None);
        assert_eq!(parse_color("rgb(10, nope, 20)"), None);
    }
}
