use cssparser::{Parser, ParserInput, Token};

use super::parse_color_with_opacity_depth;

pub(super) fn parse_relative_color(value: &str, depth: usize) -> Option<([u8; 3], f32)> {
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    let function = parser.expect_function().ok()?.to_ascii_lowercase();
    let is_hsl = function == "hsl";
    let is_lab = function == "lab";
    let is_lch = function == "lch";
    let is_oklab = function == "oklab";
    let is_oklch = function == "oklch";
    if function != "rgb" && !is_hsl && !is_lab && !is_lch && !is_oklab && !is_oklch {
        return None;
    }

    let color = parser
        .parse_nested_block(|input| {
            input.expect_ident_matching("from")?;
            let source_color = parse_source_color(input, depth)?;
            let channels = if is_hsl {
                let source_hsl = super::rgb_to_hsl(source_color.0);
                let hue = parse_hue_channel(input, source_hsl.0)?;
                let saturation = parse_hsl_unit_channel(input, source_hsl.1, "s")?;
                let lightness = parse_hsl_unit_channel(input, source_hsl.2, "l")?;
                super::hsl_to_rgb(hue, saturation, lightness)
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

fn parse_relative_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    source_value: f32,
    name: &str,
    percentage_scale: f32,
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    match input.next()? {
        Token::Ident(value) if value.eq_ignore_ascii_case(name) => Ok(source_value),
        Token::Number { value, .. } if value.is_finite() => Ok(*value),
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => {
            Ok(*unit_value * percentage_scale)
        }
        _ => Err(input.new_custom_error(())),
    }
}

fn parse_hue_channel<'i, 't>(
    input: &mut Parser<'i, 't>,
    source_hue: f32,
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    let hue = match input.next()? {
        Token::Ident(name) if name.eq_ignore_ascii_case("h") => source_hue,
        Token::Number { value, .. } if value.is_finite() => *value,
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
        Token::Ident(value) if value.eq_ignore_ascii_case(name) => source_value,
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => *unit_value,
        _ => return Err(input.new_custom_error(())),
    };
    Ok(value.clamp(0.0, 1.0))
}

fn parse_source_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    depth: usize,
) -> Result<([u8; 3], f32), cssparser::ParseError<'i, ()>> {
    let start = input.position();
    match input.next()? {
        Token::Function(_) | Token::ParenthesisBlock => {
            input.parse_nested_block(|nested| consume_all_tokens(nested, 0))?
        }
        _ => {}
    }
    let end = input.position();
    parse_color_with_opacity_depth(input.slice(start..end).trim(), depth + 1)
        .ok_or_else(|| input.new_custom_error(()))
}

fn parse_channel<'i, 't>(
    input: &mut Parser<'i, 't>,
    source: ([u8; 3], f32),
) -> Result<u8, cssparser::ParseError<'i, ()>> {
    let value = match input.next()? {
        Token::Ident(name) => match name.to_ascii_lowercase().as_str() {
            "r" => source.0[0] as f32,
            "g" => source.0[1] as f32,
            "b" => source.0[2] as f32,
            "alpha" => source.1 * 255.0,
            _ => return Err(input.new_custom_error(())),
        },
        Token::Number { value, .. } if value.is_finite() => *value,
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => *unit_value * 255.0,
        _ => return Err(input.new_custom_error(())),
    };
    Ok(value.clamp(0.0, 255.0).round() as u8)
}

fn parse_alpha_channel<'i, 't>(
    input: &mut Parser<'i, 't>,
    source: ([u8; 3], f32),
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    let alpha = match input.next()? {
        Token::Ident(name) if name.eq_ignore_ascii_case("alpha") => source.1,
        Token::Number { value, .. } if value.is_finite() => *value,
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => *unit_value,
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
