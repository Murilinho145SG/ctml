use cssparser::{Parser, ParserInput, Token};

use super::parse_color_with_opacity_depth;

pub(super) fn parse_relative_color(value: &str, depth: usize) -> Option<([u8; 3], f32)> {
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    if !parser.expect_function().ok()?.eq_ignore_ascii_case("rgb") {
        return None;
    }

    let color = parser
        .parse_nested_block(|input| {
            input.expect_ident_matching("from")?;
            let source_color = parse_source_color(input, depth)?;
            let red = parse_channel(input, source_color)?;
            let green = parse_channel(input, source_color)?;
            let blue = parse_channel(input, source_color)?;
            let alpha = if input.is_exhausted() {
                source_color.1
            } else {
                input.expect_delim('/')?;
                parse_alpha_channel(input, source_color)?
            };
            if !input.is_exhausted() {
                return Err(input.new_custom_error(()));
            }
            Ok(([red, green, blue], alpha))
        })
        .ok()?;

    parser.is_exhausted().then_some(color)
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
