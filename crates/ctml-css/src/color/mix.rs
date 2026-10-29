use cssparser::{Parser, ParserInput, Token};

use super::parse_color_with_opacity_depth;

pub(super) fn parse_color_mix(value: &str, depth: usize) -> Option<([u8; 3], f32)> {
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    if !parser
        .expect_function()
        .ok()?
        .eq_ignore_ascii_case("color-mix")
    {
        return None;
    }

    let mixed = parser
        .parse_nested_block(|input| {
            input.expect_ident_matching("in")?;
            let interpolation_space = input.expect_ident_cloned()?;
            let linear_srgb = if interpolation_space.eq_ignore_ascii_case("srgb") {
                false
            } else if interpolation_space.eq_ignore_ascii_case("srgb-linear") {
                true
            } else {
                return Err(input.new_custom_error(()));
            };
            input.expect_comma()?;

            let (first, first_percentage) = parse_mix_component(input, depth)?;
            input.expect_comma()?;
            let (second, second_percentage) = parse_mix_component(input, depth)?;
            if !input.is_exhausted() {
                return Err(input.new_custom_error(()));
            }

            let (first_weight, second_weight) = match (first_percentage, second_percentage) {
                (Some(first), Some(second)) => (first, second),
                (Some(first), None) => (first, 100.0 - first),
                (None, Some(second)) => (100.0 - second, second),
                (None, None) => (50.0, 50.0),
            };
            if first_weight < 0.0 || second_weight < 0.0 {
                return Err(input.new_custom_error(()));
            }
            let weight_sum = first_weight + second_weight;
            if !weight_sum.is_finite() || weight_sum <= 0.0 {
                return Err(input.new_custom_error(()));
            }

            let first_weight = first_weight / weight_sum;
            let second_weight = second_weight / weight_sum;
            let first_alpha = first.1 * first_weight;
            let second_alpha = second.1 * second_weight;
            let alpha = (first_alpha + second_alpha) * (weight_sum / 100.0).min(1.0);
            let channels = if alpha == 0.0 {
                [0; 3]
            } else {
                std::array::from_fn(|index| {
                    if linear_srgb {
                        let first_channel =
                            super::color4::decode_srgb(first.0[index] as f32 / 255.0);
                        let second_channel =
                            super::color4::decode_srgb(second.0[index] as f32 / 255.0);
                        let channel = (first_channel * first_alpha + second_channel * second_alpha)
                            / (first_alpha + second_alpha);
                        super::color4::encode_srgb(channel)
                    } else {
                        ((first.0[index] as f32 * first_alpha
                            + second.0[index] as f32 * second_alpha)
                            / (first_alpha + second_alpha))
                            .round()
                            .clamp(0.0, 255.0) as u8
                    }
                })
            };
            Ok((channels, alpha.clamp(0.0, 1.0)))
        })
        .ok()?;

    parser.is_exhausted().then_some(mixed)
}

fn parse_mix_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    depth: usize,
) -> Result<(([u8; 3], f32), Option<f32>), cssparser::ParseError<'i, ()>> {
    let start = input.position();
    match input.next()? {
        Token::Function(_) => input.parse_nested_block(|nested| consume_all_tokens(nested, 0))?,
        Token::ParenthesisBlock => {
            input.parse_nested_block(|nested| consume_all_tokens(nested, 0))?
        }
        _ => {}
    }
    let color_end = input.position();
    let color = parse_color_with_opacity_depth(input.slice(start..color_end).trim(), depth + 1)
        .ok_or_else(|| input.new_custom_error(()))?;
    let percentage = input
        .try_parse(|input| input.expect_percentage())
        .ok()
        .map(|percentage| percentage * 100.0);
    if percentage.is_some_and(|percentage| !percentage.is_finite() || percentage < 0.0) {
        return Err(input.new_custom_error(()));
    }
    Ok((color, percentage))
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
