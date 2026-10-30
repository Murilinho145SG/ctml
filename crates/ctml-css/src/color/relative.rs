use cssparser::{Parser, ParserInput, Token};

use super::parse_color_with_opacity_depth_and_profiles;

#[derive(Clone, Copy)]
enum ColorSpace {
    Srgb,
    SrgbLinear,
    DisplayP3,
    DisplayP3Linear,
    A98Rgb,
    ProPhotoRgb,
    Rec2020,
    Rec2100Pq,
    Rec2100Hlg,
    Rec2020Linear,
    XyzD50,
    XyzD65,
    Custom(usize),
}

pub(super) fn parse_relative_color(
    value: &str,
    depth: usize,
    profiles: &[super::super::CssColorProfile],
) -> Option<([u8; 3], f32)> {
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    let function = parser.expect_function().ok()?.to_ascii_lowercase();
    let is_hsl = function == "hsl";
    let is_hwb = function == "hwb";
    let is_lab = function == "lab";
    let is_lch = function == "lch";
    let is_oklab = function == "oklab";
    let is_oklch = function == "oklch";
    let is_color = function == "color";
    if function != "rgb"
        && !is_hsl
        && !is_hwb
        && !is_lab
        && !is_lch
        && !is_oklab
        && !is_oklch
        && !is_color
    {
        return None;
    }

    let color = parser
        .parse_nested_block(|input| {
            input.expect_ident_matching("from")?;
            let source_color = parse_source_color(input, depth, profiles)?;
            let color_space = if is_color {
                Some(parse_color_space(input, profiles)?)
            } else {
                None
            };
            let channels = if is_hsl {
                let source_hsl = super::rgb_to_hsl(source_color.0);
                let hue = parse_hue_channel(input, source_hsl.0)?;
                let saturation = parse_hsl_unit_channel(input, source_hsl.1, "s")?;
                let lightness = parse_hsl_unit_channel(input, source_hsl.2, "l")?;
                super::hsl_to_rgb(hue, saturation, lightness)
            } else if is_hwb {
                let source_hwb = super::rgb_to_hwb(source_color.0);
                let hue = parse_hue_channel(input, source_hwb.0)?;
                let whiteness = parse_hsl_unit_channel(input, source_hwb.1, "w")?;
                let blackness = parse_hsl_unit_channel(input, source_hwb.2, "b")?;
                super::hwb_to_rgb(hue, whiteness, blackness)
            } else if is_lab {
                let source_lab = super::color4::srgb_to_lab(source_color.0);
                let lightness = parse_relative_component(input, source_lab.0, "l", 100.0)?;
                let a = parse_relative_component(input, source_lab.1, "a", 125.0)?;
                let b = parse_relative_component(input, source_lab.2, "b", 125.0)?;
                super::color4::lab_to_srgb((lightness, a, b))
            } else if is_lch {
                let source_lab = super::color4::srgb_to_lab(source_color.0);
                let chroma = source_lab.1.hypot(source_lab.2);
                let source_hue = source_lab
                    .2
                    .atan2(source_lab.1)
                    .to_degrees()
                    .rem_euclid(360.0);
                let lightness = parse_relative_component(input, source_lab.0, "l", 100.0)?;
                let chroma = parse_relative_component(input, chroma, "c", 150.0)?;
                let hue = parse_hue_channel(input, source_hue)?.to_radians();
                super::color4::lab_to_srgb((lightness, chroma * hue.cos(), chroma * hue.sin()))
            } else if is_oklab {
                let source_lab = super::color4::srgb_to_oklab(source_color.0);
                let lightness = parse_relative_component(input, source_lab.0, "l", 1.0)?;
                let a = parse_relative_component(input, source_lab.1, "a", 0.4)?;
                let b = parse_relative_component(input, source_lab.2, "b", 0.4)?;
                super::color4::oklab_to_srgb((lightness, a, b))
            } else if is_oklch {
                let source_lab = super::color4::srgb_to_oklab(source_color.0);
                let chroma = source_lab.1.hypot(source_lab.2);
                let source_hue = source_lab
                    .2
                    .atan2(source_lab.1)
                    .to_degrees()
                    .rem_euclid(360.0);
                let lightness = parse_relative_component(input, source_lab.0, "l", 1.0)?;
                let chroma = parse_relative_component(input, chroma, "c", 0.4)?;
                let hue = parse_hue_channel(input, source_hue)?.to_radians();
                super::color4::oklab_to_srgb((lightness, chroma * hue.cos(), chroma * hue.sin()))
            } else if let Some(space) = color_space {
                if let ColorSpace::Custom(index) = space {
                    let profile = &profiles[index];
                    let source_components = profile
                        .from_srgb(source_color.0.map(|channel| channel as f32 / 255.0))
                        .ok_or_else(|| input.new_custom_error(()))?;
                    let names = profile
                        .components
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>();
                    let components = profile
                        .components
                        .iter()
                        .map(|_| parse_color_component(input, &source_components, &names))
                        .collect::<Result<Vec<_>, _>>()?;
                    profile
                        .to_srgb(&components)
                        .ok_or_else(|| input.new_custom_error(()))?
                        .map(|channel| (channel * 255.0).round().clamp(0.0, 255.0) as u8)
                } else {
                    let source_components = convert_source_color(space, source_color.0);
                    let names = match space {
                        ColorSpace::XyzD50 | ColorSpace::XyzD65 => ["x", "y", "z"],
                        _ => ["r", "g", "b"],
                    };
                    let components = [
                        parse_color_component(input, &source_components, &names)?,
                        parse_color_component(input, &source_components, &names)?,
                        parse_color_component(input, &source_components, &names)?,
                    ];
                    convert_color_space(space, components)
                }
            } else {
                [
                    parse_channel(input, source_color)?,
                    parse_channel(input, source_color)?,
                    parse_channel(input, source_color)?,
                ]
            };
            let alpha = if input.is_exhausted() {
                source_color.1
            } else {
                input.expect_delim('/')?;
                parse_alpha_channel(input, source_color)?
            };
            if !input.is_exhausted() {
                return Err(input.new_custom_error(()));
            }
            Ok((channels, alpha))
        })
        .ok()?;

    parser.is_exhausted().then_some(color)
}

fn parse_color_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    source_components: &[f32],
    names: &[&str],
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    let start = input.position();
    match input.next()? {
        Token::Ident(name) if name.eq_ignore_ascii_case("none") => Ok(0.0),
        Token::Ident(name) => names
            .iter()
            .position(|component| name.eq_ignore_ascii_case(component))
            .map(|index| source_components[index])
            .ok_or_else(|| input.new_custom_error(())),
        Token::Number { value, .. } if value.is_finite() => Ok(*value),
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => Ok(*unit_value),
        Token::Function(_) => evaluate_math_component(
            input,
            start,
            &names
                .iter()
                .zip(source_components.iter())
                .map(|(name, value)| (*name, *value))
                .collect::<Vec<_>>(),
            1.0,
        ),
        _ => Err(input.new_custom_error(())),
    }
}

fn parse_color_space<'i, 't>(
    input: &mut Parser<'i, 't>,
    profiles: &[super::super::CssColorProfile],
) -> Result<ColorSpace, cssparser::ParseError<'i, ()>> {
    let name = input.expect_ident_cloned()?.to_string();
    if name.starts_with("--") {
        return profiles
            .iter()
            .rposition(|profile| profile.name == name)
            .filter(|index| profiles[*index].channel_count().is_some())
            .map(ColorSpace::Custom)
            .ok_or_else(|| input.new_custom_error(()));
    }
    match name.to_ascii_lowercase().as_str() {
        "srgb" => Ok(ColorSpace::Srgb),
        "srgb-linear" => Ok(ColorSpace::SrgbLinear),
        "display-p3" => Ok(ColorSpace::DisplayP3),
        "display-p3-linear" => Ok(ColorSpace::DisplayP3Linear),
        "a98-rgb" => Ok(ColorSpace::A98Rgb),
        "prophoto-rgb" => Ok(ColorSpace::ProPhotoRgb),
        "rec2020" => Ok(ColorSpace::Rec2020),
        "rec2100-pq" => Ok(ColorSpace::Rec2100Pq),
        "rec2100-hlg" => Ok(ColorSpace::Rec2100Hlg),
        "rec2100-linear" => Ok(ColorSpace::Rec2020Linear),
        "xyz-d50" => Ok(ColorSpace::XyzD50),
        "xyz" | "xyz-d65" => Ok(ColorSpace::XyzD65),
        _ => Err(input.new_custom_error(())),
    }
}

fn convert_source_color(space: ColorSpace, color: [u8; 3]) -> [f32; 3] {
    match space {
        ColorSpace::Srgb => color.map(|channel| channel as f32 / 255.0),
        ColorSpace::SrgbLinear => {
            color.map(|channel| super::color4::decode_srgb(channel as f32 / 255.0))
        }
        ColorSpace::DisplayP3 => super::color4::srgb_to_display_p3(color),
        ColorSpace::DisplayP3Linear => super::color4::srgb_to_display_p3_linear(color),
        ColorSpace::A98Rgb => super::color4::srgb_to_a98(color),
        ColorSpace::ProPhotoRgb => super::color4::srgb_to_prophoto(color),
        ColorSpace::Rec2020 => super::color4::srgb_to_rec2020(color),
        ColorSpace::Rec2100Pq => super::hdr::srgb_to_rec2100_pq(color),
        ColorSpace::Rec2100Hlg => super::hdr::srgb_to_rec2100_hlg(color),
        ColorSpace::Rec2020Linear => super::color4::srgb_to_rec2020_linear(color),
        ColorSpace::XyzD50 => super::color4::srgb_to_xyz_d50(color),
        ColorSpace::XyzD65 => super::color4::srgb_to_xyz_d65(color),
        ColorSpace::Custom(_) => {
            unreachable!("custom color spaces are converted with their ICC profile")
        }
    }
}

fn convert_color_space(space: ColorSpace, components: [f32; 3]) -> [u8; 3] {
    match space {
        ColorSpace::Srgb => {
            components.map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8)
        }
        ColorSpace::SrgbLinear => components.map(super::color4::encode_srgb),
        ColorSpace::DisplayP3 => super::color4::display_p3_to_srgb(components),
        ColorSpace::DisplayP3Linear => super::color4::display_p3_linear_to_srgb(components),
        ColorSpace::A98Rgb => super::color4::a98_to_srgb(components),
        ColorSpace::ProPhotoRgb => super::color4::prophoto_to_srgb(components),
        ColorSpace::Rec2020 => super::color4::rec2020_to_srgb(components),
        ColorSpace::Rec2100Pq => super::hdr::rec2100_pq_to_srgb(components),
        ColorSpace::Rec2100Hlg => super::hdr::rec2100_hlg_to_srgb(components),
        ColorSpace::Rec2020Linear => super::color4::rec2020_linear_to_srgb(components),
        ColorSpace::XyzD50 => super::color4::xyz_d50_to_srgb(components),
        ColorSpace::XyzD65 => super::color4::xyz_d65_to_srgb(components),
        ColorSpace::Custom(_) => {
            unreachable!("custom color spaces are converted with their ICC profile")
        }
    }
}

fn parse_relative_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    source_value: f32,
    name: &str,
    percentage_scale: f32,
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    let start = input.position();
    match input.next()? {
        Token::Ident(value) if value.eq_ignore_ascii_case("none") => Ok(0.0),
        Token::Ident(value) if value.eq_ignore_ascii_case(name) => Ok(source_value),
        Token::Number { value, .. } if value.is_finite() => Ok(*value),
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => {
            Ok(*unit_value * percentage_scale)
        }
        Token::Function(_) => {
            evaluate_math_component(input, start, &[(name, source_value)], percentage_scale)
        }
        _ => Err(input.new_custom_error(())),
    }
}

fn evaluate_math_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    start: cssparser::SourcePosition,
    references: &[(&str, f32)],
    percentage_scale: f32,
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    evaluate_math_component_with_angles(input, start, references, percentage_scale, false)
}

fn evaluate_math_component_with_angles<'i, 't>(
    input: &mut Parser<'i, 't>,
    start: cssparser::SourcePosition,
    references: &[(&str, f32)],
    percentage_scale: f32,
    convert_angles: bool,
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    input.parse_nested_block(|nested| consume_all_tokens(nested, 0))?;
    let end = input.position();
    let expression = substitute_math_references(
        input.slice(start..end),
        references,
        percentage_scale,
        convert_angles,
    );
    crate::parse_math_number(&expression).ok_or_else(|| input.new_custom_error(()))
}

fn substitute_math_references(
    expression: &str,
    references: &[(&str, f32)],
    percentage_scale: f32,
    convert_angles: bool,
) -> String {
    let mut result = String::with_capacity(expression.len());
    let mut cursor = 0;
    while cursor < expression.len() {
        let remaining = &expression[cursor..];
        let first = remaining.chars().next().expect("cursor is in bounds");
        if first.is_ascii_alphabetic() {
            let identifier_length = remaining
                .char_indices()
                .take_while(|(_, character)| character.is_ascii_alphanumeric() || *character == '-')
                .map(|(index, character)| index + character.len_utf8())
                .last()
                .unwrap_or(first.len_utf8());
            let identifier = &remaining[..identifier_length];
            if let Some((_, value)) = references
                .iter()
                .find(|(name, _)| identifier.eq_ignore_ascii_case(name))
            {
                result.push_str(&value.to_string());
            } else {
                result.push_str(identifier);
            }
            cursor += identifier_length;
        } else if first.is_ascii_digit()
            || (first == '.' && remaining.as_bytes().get(1).is_some_and(u8::is_ascii_digit))
        {
            let number_length = math_number_length(remaining);
            let number = &remaining[..number_length];
            if remaining.as_bytes().get(number_length) == Some(&b'%') {
                let value = number.parse::<f32>().unwrap_or(f32::NAN) / 100.0 * percentage_scale;
                result.push_str(&value.to_string());
                cursor += number_length + 1;
            } else if convert_angles {
                let unit_length = remaining[number_length..]
                    .chars()
                    .take_while(char::is_ascii_alphabetic)
                    .map(char::len_utf8)
                    .sum::<usize>();
                let unit = &remaining[number_length..number_length + unit_length];
                let value = number.parse::<f32>().unwrap_or(f32::NAN);
                let degrees = match unit.to_ascii_lowercase().as_str() {
                    "deg" => Some(value),
                    "grad" => Some(value * 0.9),
                    "rad" => Some(value.to_degrees()),
                    "turn" => Some(value * 360.0),
                    _ => None,
                };
                if let Some(degrees) = degrees {
                    result.push_str(&degrees.to_string());
                    cursor += number_length + unit_length;
                } else {
                    result.push_str(number);
                    cursor += number_length;
                }
            } else {
                result.push_str(number);
                cursor += number_length;
            }
        } else {
            result.push(first);
            cursor += first.len_utf8();
        }
    }
    result
}

fn math_number_length(value: &str) -> usize {
    let bytes = value.as_bytes();
    let mut cursor = 0;
    while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    if bytes.get(cursor) == Some(&b'.') {
        cursor += 1;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
    }
    if bytes
        .get(cursor)
        .is_some_and(|byte| matches!(byte, b'e' | b'E'))
    {
        let exponent_start = cursor;
        cursor += 1;
        if bytes
            .get(cursor)
            .is_some_and(|byte| matches!(byte, b'+' | b'-'))
        {
            cursor += 1;
        }
        let digit_start = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        if digit_start == cursor {
            cursor = exponent_start;
        }
    }
    cursor
}

fn parse_hue_channel<'i, 't>(
    input: &mut Parser<'i, 't>,
    source_hue: f32,
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    let start = input.position();
    let hue = match input.next()? {
        Token::Ident(name) if name.eq_ignore_ascii_case("none") => 0.0,
        Token::Ident(name) if name.eq_ignore_ascii_case("h") => source_hue,
        Token::Number { value, .. } if value.is_finite() => *value,
        Token::Function(_) => {
            evaluate_math_component_with_angles(input, start, &[("h", source_hue)], f32::NAN, true)?
        }
        Token::Dimension { value, unit, .. } if value.is_finite() => {
            let value = *value;
            match unit.to_ascii_lowercase().as_str() {
                "deg" => value,
                "grad" => value * 0.9,
                "rad" => value.to_degrees(),
                "turn" => value * 360.0,
                _ => return Err(input.new_custom_error(())),
            }
        }
        _ => return Err(input.new_custom_error(())),
    };
    Ok(hue.rem_euclid(360.0))
}

fn parse_hsl_unit_channel<'i, 't>(
    input: &mut Parser<'i, 't>,
    source_value: f32,
    name: &str,
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    let value = match input.next()? {
        Token::Ident(value) if value.eq_ignore_ascii_case("none") => 0.0,
        Token::Ident(value) if value.eq_ignore_ascii_case(name) => source_value,
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => *unit_value,
        _ => return Err(input.new_custom_error(())),
    };
    Ok(value.clamp(0.0, 1.0))
}

fn parse_source_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    depth: usize,
    profiles: &[super::super::CssColorProfile],
) -> Result<([u8; 3], f32), cssparser::ParseError<'i, ()>> {
    let start = input.position();
    match input.next()? {
        Token::Function(_) | Token::ParenthesisBlock => {
            input.parse_nested_block(|nested| consume_all_tokens(nested, 0))?
        }
        _ => {}
    }
    let end = input.position();
    parse_color_with_opacity_depth_and_profiles(input.slice(start..end).trim(), depth + 1, profiles)
        .ok_or_else(|| input.new_custom_error(()))
}

fn parse_channel<'i, 't>(
    input: &mut Parser<'i, 't>,
    source: ([u8; 3], f32),
) -> Result<u8, cssparser::ParseError<'i, ()>> {
    let start = input.position();
    let value = match input.next()? {
        Token::Ident(name) => match name.to_ascii_lowercase().as_str() {
            "none" => 0.0,
            "r" => source.0[0] as f32,
            "g" => source.0[1] as f32,
            "b" => source.0[2] as f32,
            "alpha" => source.1 * 255.0,
            _ => return Err(input.new_custom_error(())),
        },
        Token::Number { value, .. } if value.is_finite() => *value,
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => *unit_value * 255.0,
        Token::Function(_) => evaluate_math_component(
            input,
            start,
            &[
                ("r", source.0[0] as f32),
                ("g", source.0[1] as f32),
                ("b", source.0[2] as f32),
                ("alpha", source.1 * 255.0),
            ],
            255.0,
        )?,
        _ => return Err(input.new_custom_error(())),
    };
    Ok(value.clamp(0.0, 255.0).round() as u8)
}

fn parse_alpha_channel<'i, 't>(
    input: &mut Parser<'i, 't>,
    source: ([u8; 3], f32),
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    let start = input.position();
    let alpha = match input.next()? {
        Token::Ident(name) if name.eq_ignore_ascii_case("none") => 0.0,
        Token::Ident(name) if name.eq_ignore_ascii_case("alpha") => source.1,
        Token::Number { value, .. } if value.is_finite() => *value,
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => *unit_value,
        Token::Function(_) => evaluate_math_component(input, start, &[("alpha", source.1)], 1.0)?,
        _ => return Err(input.new_custom_error(())),
    };
    Ok(alpha.clamp(0.0, 1.0))
}

fn consume_all_tokens<'i, 't>(
    input: &mut Parser<'i, 't>,
    depth: usize,
) -> Result<(), cssparser::ParseError<'i, ()>> {
    if depth > 32 {
        return Err(input.new_custom_error(()));
    }
    while !input.is_exhausted() {
        match input.next()? {
            Token::Function(_) | Token::ParenthesisBlock => {
                input.parse_nested_block(|nested| consume_all_tokens(nested, depth + 1))?
            }
            _ => {}
        }
    }
    Ok(())
}
