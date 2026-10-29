use cssparser::{Parser, ParserInput, Token};

mod color4;
mod mix;
mod relative;

pub fn parse_color(value: &str) -> Option<[u8; 4]> {
    let ([red, green, blue], alpha) = parse_color_with_opacity(value)?;
    Some([red, green, blue, (alpha * 255.0).round() as u8])
}

pub fn parse_color_with_opacity(value: &str) -> Option<([u8; 3], f32)> {
    parse_color_with_opacity_depth(value, 0)
}

pub(super) fn parse_color_with_opacity_depth(value: &str, depth: usize) -> Option<([u8; 3], f32)> {
    if depth > 32 {
        return None;
    }
    let value = value.trim();
    if value.eq_ignore_ascii_case("transparent") {
        return Some(([0, 0, 0], 0.0));
    }
    if let Some(hex) = value.strip_prefix('#') {
        let (red, green, blue, alpha) = cssparser::color::parse_hash_color(hex.as_bytes()).ok()?;
        return Some(([red, green, blue], alpha));
    }

    if let Ok((red, green, blue)) = cssparser::color::parse_named_color(&value.to_ascii_lowercase())
    {
        return Some(([red, green, blue], 1.0));
    }

    parse_function_color(value, depth)
}

fn parse_function_color(value: &str, depth: usize) -> Option<([u8; 3], f32)> {
    if let Some(color) = mix::parse_color_mix(value, depth) {
        return Some(color);
    }
    if let Some(color) = relative::parse_relative_color(value, depth) {
        return Some(color);
    }
    if let Some(color) = color4::parse_color_function(value) {
        return Some(color);
    }
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    let function = parser.expect_function().ok()?.to_ascii_lowercase();
    let is_hsl = matches!(function.as_str(), "hsl" | "hsla");
    let is_hwb = function == "hwb";
    if !is_hsl && !is_hwb && !matches!(function.as_str(), "rgb" | "rgba") {
        return None;
    }

    let color = parser
        .parse_nested_block(|input| {
            let red_or_hue = if is_hsl || is_hwb {
                parse_hue(input)?
            } else {
                parse_rgb_component(input)?
            };
            let legacy_syntax = input.try_parse(|input| input.expect_comma()).is_ok();
            if is_hwb && legacy_syntax {
                return Err(input.new_custom_error(()));
            }
            let green_or_saturation = if is_hsl || is_hwb {
                parse_unit_component(input)?
            } else {
                parse_component(input)?
            };
            if legacy_syntax {
                input.expect_comma()?;
            }
            let blue_or_lightness = if is_hsl || is_hwb {
                parse_unit_component(input)?
            } else {
                parse_component(input)?
            };

            let alpha = if legacy_syntax {
                if input.is_exhausted() {
                    1.0
                } else {
                    input.expect_comma()?;
                    parse_alpha(input)?
                }
            } else if input.is_exhausted() {
                1.0
            } else {
                input.expect_delim('/')?;
                parse_alpha(input)?
            };
            if !input.is_exhausted() {
                return Err(input.new_custom_error(()));
            }

            let (red, green, blue) = if is_hwb {
                let [red, green, blue] =
                    hwb_to_rgb(red_or_hue, green_or_saturation, blue_or_lightness);
                (red, green, blue)
            } else if is_hsl {
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
            Ok(([red, green, blue], alpha))
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

fn parse_alpha<'i, 't>(input: &mut Parser<'i, 't>) -> Result<f32, cssparser::ParseError<'i, ()>> {
    let alpha = match input.next()? {
        Token::Number { value, .. } if value.is_finite() => *value,
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => *unit_value,
        _ => return Err(input.new_custom_error(())),
    };
    Ok(alpha.clamp(0.0, 1.0))
}

fn component_to_byte(component: f32) -> u8 {
    component.clamp(0.0, 255.0).round() as u8
}

pub(super) fn hsl_to_rgb(hue: f32, saturation: f32, lightness: f32) -> [u8; 3] {
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

pub(super) fn rgb_to_hsl([red, green, blue]: [u8; 3]) -> (f32, f32, f32) {
    let red = red as f32 / 255.0;
    let green = green as f32 / 255.0;
    let blue = blue as f32 / 255.0;
    let maximum = red.max(green).max(blue);
    let minimum = red.min(green).min(blue);
    let delta = maximum - minimum;
    let lightness = (maximum + minimum) / 2.0;
    if delta == 0.0 {
        return (0.0, 0.0, lightness);
    }
    let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let hue = if maximum == red {
        60.0 * ((green - blue) / delta).rem_euclid(6.0)
    } else if maximum == green {
        60.0 * ((blue - red) / delta + 2.0)
    } else {
        60.0 * ((red - green) / delta + 4.0)
    };
    (hue, saturation, lightness)
}

pub(super) fn hwb_to_rgb(hue: f32, whiteness: f32, blackness: f32) -> [u8; 3] {
    let whiteness = whiteness.clamp(0.0, 1.0);
    let blackness = blackness.clamp(0.0, 1.0);
    if whiteness + blackness >= 1.0 {
        let gray = (whiteness / (whiteness + blackness) * 255.0).round() as u8;
        return [gray; 3];
    }

    let pure_hue = hsl_to_rgb(hue, 1.0, 0.5);
    let chroma = 1.0 - whiteness - blackness;
    pure_hue.map(|channel| ((channel as f32 / 255.0 * chroma + whiteness) * 255.0).round() as u8)
}

pub(super) fn rgb_to_hwb(color: [u8; 3]) -> (f32, f32, f32) {
    let hue = rgb_to_hsl(color).0;
    let red = color[0] as f32 / 255.0;
    let green = color[1] as f32 / 255.0;
    let blue = color[2] as f32 / 255.0;
    (
        hue,
        red.min(green).min(blue),
        1.0 - red.max(green).max(blue),
    )
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
    fn parses_modern_space_separated_rgb_and_hsl() {
        assert_eq!(
            parse_color("rgb(100% 0% 50% / 50%)"),
            Some([255, 0, 128, 128])
        );
        assert_eq!(
            parse_color("hsl(120 100% 25% / 50%)"),
            Some([0, 128, 0, 128])
        );
    }

    #[test]
    fn parses_hwb_colors_with_alpha() {
        assert_eq!(parse_color("hwb(0 0% 0%)"), Some([255, 0, 0, 255]));
        assert_eq!(
            parse_color("hwb(120 20% 10% / 50%)"),
            Some([51, 230, 51, 128])
        );
        assert_eq!(parse_color("hwb(0 100% 100%)"), Some([128, 128, 128, 255]));
    }

    #[test]
    fn rejects_unsupported_or_malformed_color_syntax() {
        assert_eq!(parse_color("hwb(120, 20%, 30%)"), None);
        assert_eq!(parse_color("rgb(10, nope, 20)"), None);
        assert_eq!(parse_color("rgb(10, 20 30)"), None);
    }

    #[test]
    fn parses_lab_lch_oklab_oklch_and_color_functions() {
        for color in [
            "lab(54.2917% 80.8125 69.8851)",
            "lch(54.2917% 106.839 40.853deg)",
            "oklab(0.627955 0.224863 0.125846)",
            "oklch(0.627955 0.257683 29.233deg)",
            "color(srgb 1 0 0)",
            "color(srgb-linear 1 0 0)",
            "color(display-p3 1 0 0)",
            "color(xyz 0.4123908 0.2126390 0.0193308)",
            "color(xyz-d50 0.4360747 0.2225045 0.0139322)",
            "color(xyz-d65 0.4123908 0.2126390 0.0193308)",
        ] {
            let Some([red, green, blue, alpha]) = parse_color(color) else {
                panic!("failed to parse {color}");
            };
            assert!(red >= 250, "unexpected red channel for {color}: {red}");
            assert!(green <= 5, "unexpected green channel for {color}: {green}");
            assert!(blue <= 5, "unexpected blue channel for {color}: {blue}");
            assert_eq!(alpha, 255);
        }
        assert_eq!(
            parse_color("color(srgb 1 0 0 / 50%)"),
            Some([255, 0, 0, 128])
        );
        assert_eq!(parse_color("lab(50% nope 0)"), None);
    }

    #[test]
    fn parses_srgb_color_mixes_with_default_and_explicit_weights() {
        assert_eq!(
            parse_color("color-mix(in srgb, red, blue)"),
            Some([128, 0, 128, 255])
        );
        assert_eq!(
            parse_color("color-mix(in srgb, red 25%, blue)"),
            Some([64, 0, 191, 255])
        );
        assert_eq!(
            parse_color("color-mix(in srgb, red, blue 25%)"),
            Some([191, 0, 64, 255])
        );
        assert_eq!(
            parse_color("color-mix(in srgb, red 25%, blue 25%)"),
            Some([128, 0, 128, 128])
        );
    }

    #[test]
    fn parses_linear_srgb_color_mixes_in_linear_light() {
        assert_eq!(
            parse_color("color-mix(in srgb-linear, red, blue)"),
            Some([188, 0, 188, 255])
        );
        assert_eq!(
            parse_color("color-mix(in srgb-linear, transparent, red)"),
            Some([255, 0, 0, 128])
        );
    }

    #[test]
    fn parses_oklab_color_mixes_perceptually() {
        assert_eq!(
            parse_color("color-mix(in oklab, red, blue)"),
            Some([140, 83, 162, 255])
        );
        assert_eq!(
            parse_color("color-mix(in oklab, transparent, red)"),
            Some([255, 0, 0, 128])
        );
    }

    #[test]
    fn parses_hsl_color_mixes_with_shorter_hue_interpolation() {
        assert_eq!(
            parse_color("color-mix(in hsl, red, blue)"),
            Some([255, 0, 255, 255])
        );
        assert_eq!(
            parse_color("color-mix(in hsl, transparent, red)"),
            Some([255, 0, 0, 128])
        );
        assert_eq!(
            parse_color("color-mix(in hsl, red 25%, blue)"),
            Some([128, 0, 255, 255])
        );
    }

    #[test]
    fn parses_hwb_color_mixes_with_shorter_hue_interpolation() {
        assert_eq!(
            parse_color("color-mix(in hwb, red, blue)"),
            Some([255, 0, 255, 255])
        );
        assert_eq!(
            parse_color("color-mix(in hwb, transparent, red)"),
            Some([255, 0, 0, 128])
        );
        assert_eq!(
            parse_color("color-mix(in hwb, white, blue)"),
            Some([128, 128, 255, 255])
        );
    }

    #[test]
    fn parses_lab_and_lch_color_mixes() {
        let red_lab = super::color4::srgb_to_lab([255, 0, 0]);
        assert!((red_lab.0 - 54.29).abs() < 0.1);
        assert!((red_lab.1 - 80.81).abs() < 0.1);
        assert!((red_lab.2 - 69.89).abs() < 0.1);
        assert_eq!(
            parse_color("color-mix(in lab, red, blue)"),
            Some([193, 0, 136, 255])
        );
        assert_eq!(
            parse_color("color-mix(in lch, red, blue)"),
            Some([245, 0, 134, 255])
        );
        assert_eq!(
            parse_color("color-mix(in oklch, red, blue)"),
            Some([186, 0, 194, 255])
        );
        for space in ["lab", "lch", "oklch"] {
            assert_eq!(
                parse_color(&format!("color-mix(in {space}, transparent, red)")),
                Some([255, 0, 0, 128]),
                "unexpected transparent mix in {space}"
            );
        }
    }

    #[test]
    fn parses_xyz_color_mixes_with_matching_white_points() {
        for space in ["xyz", "xyz-d65", "xyz-d50"] {
            assert_eq!(
                parse_color(&format!("color-mix(in {space}, red, blue)")),
                Some([188, 0, 188, 255]),
                "unexpected XYZ mix in {space}"
            );
        }
    }

    #[test]
    fn parses_display_p3_color_mixes_from_supported_srgb_colors() {
        assert_eq!(
            parse_color("color-mix(in display-p3, red, blue)"),
            Some([188, 0, 188, 255])
        );
        assert_eq!(
            parse_color("color-mix(in display-p3, transparent, red)"),
            Some([255, 0, 0, 128])
        );
    }

    #[test]
    fn parses_rgb_relative_colors_with_channel_and_alpha_references() {
        assert_eq!(parse_color("rgb(from red r g b)"), Some([255, 0, 0, 255]));
        assert_eq!(
            parse_color("rgb(from rgb(10 20 30 / 25%) b g r / alpha)"),
            Some([30, 20, 10, 64])
        );
        assert_eq!(
            parse_color("rgb(from red 20 40% 255 / 50%)"),
            Some([20, 102, 255, 128])
        );
    }

    #[test]
    fn rejects_invalid_rgb_relative_color_components() {
        for value in [
            "rgb(from red r g)",
            "rgb(from red x y z)",
            "rgb(from red r g b / nope)",
            "rgb(from nope r g b)",
        ] {
            assert_eq!(parse_color(value), None, "unexpectedly parsed {value}");
        }
    }

    #[test]
    fn parses_hsl_relative_colors_with_angle_units_and_channel_references() {
        assert_eq!(parse_color("hsl(from red h s l)"), Some([255, 0, 0, 255]));
        assert_eq!(
            parse_color("hsl(from red 120deg s l)"),
            Some([0, 255, 0, 255])
        );
        assert_eq!(
            parse_color("hsl(from red 0.5turn s l)"),
            Some([0, 255, 255, 255])
        );
        assert_eq!(
            parse_color("hsl(from hsl(240 100% 50% / 25%) h s l / alpha)"),
            Some([0, 0, 255, 64])
        );
    }

    #[test]
    fn color_mixes_premultiply_alpha_and_allow_nested_color_values() {
        assert_eq!(
            parse_color("color-mix(in srgb, transparent, red)"),
            Some([255, 0, 0, 128])
        );
        assert_eq!(
            parse_color(
                "color-mix(in srgb, rgb(255 0 0 / 50%) 50%, color-mix(in srgb, blue, white) 50%)"
            ),
            Some([170, 85, 170, 191])
        );
    }

    #[test]
    fn rejects_unsupported_or_invalid_color_mix_syntax() {
        for value in [
            "color-mix(in display-p3, red, blue)",
            "color-mix(in srgb, red -10%, blue)",
            "color-mix(in srgb, red 0%, blue 0%)",
            "color-mix(in srgb, red 80%, blue 30%, white)",
            "color-mix(in srgb, red, nope)",
        ] {
            assert_eq!(parse_color(value), None, "unexpectedly parsed {value}");
        }
    }
}
