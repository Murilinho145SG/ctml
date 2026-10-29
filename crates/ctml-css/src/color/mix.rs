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
            let linear_srgb = interpolation_space.eq_ignore_ascii_case("srgb-linear");
            let lab = interpolation_space.eq_ignore_ascii_case("lab");
            let lch = interpolation_space.eq_ignore_ascii_case("lch");
            let oklab = interpolation_space.eq_ignore_ascii_case("oklab");
            let oklch = interpolation_space.eq_ignore_ascii_case("oklch");
            let hsl = interpolation_space.eq_ignore_ascii_case("hsl");
            let hwb = interpolation_space.eq_ignore_ascii_case("hwb");
            if !interpolation_space.eq_ignore_ascii_case("srgb")
                && !linear_srgb
                && !lab
                && !lch
                && !oklab
                && !oklch
                && !hsl
                && !hwb
            {
                return Err(input.new_custom_error(()));
            }
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
            } else if lab {
                let first_lab = super::color4::srgb_to_lab(first.0);
                let second_lab = super::color4::srgb_to_lab(second.0);
                let alpha_sum = first_alpha + second_alpha;
                super::color4::lab_to_srgb((
                    (first_lab.0 * first_alpha + second_lab.0 * second_alpha) / alpha_sum,
                    (first_lab.1 * first_alpha + second_lab.1 * second_alpha) / alpha_sum,
                    (first_lab.2 * first_alpha + second_lab.2 * second_alpha) / alpha_sum,
                ))
            } else if lch {
                let first_lab = super::color4::srgb_to_lab(first.0);
                let second_lab = super::color4::srgb_to_lab(second.0);
                let first_chroma = first_lab.1.hypot(first_lab.2);
                let second_chroma = second_lab.1.hypot(second_lab.2);
                let first_hue = first_lab
                    .2
                    .atan2(first_lab.1)
                    .to_degrees()
                    .rem_euclid(360.0);
                let second_hue = second_lab
                    .2
                    .atan2(second_lab.1)
                    .to_degrees()
                    .rem_euclid(360.0);
                let alpha_sum = first_alpha + second_alpha;
                let hue = interpolate_hue(
                    first_hue,
                    first_chroma < 0.0001,
                    second_hue,
                    second_chroma < 0.0001,
                    first_alpha,
                    second_alpha,
                );
                let lightness =
                    (first_lab.0 * first_alpha + second_lab.0 * second_alpha) / alpha_sum;
                let chroma =
                    (first_chroma * first_alpha + second_chroma * second_alpha) / alpha_sum;
                let hue = hue.to_radians();
                super::color4::lab_to_srgb((lightness, chroma * hue.cos(), chroma * hue.sin()))
            } else if oklab {
                let first_lab = super::color4::srgb_to_oklab(first.0);
                let second_lab = super::color4::srgb_to_oklab(second.0);
                let alpha_sum = first_alpha + second_alpha;
                super::color4::oklab_to_srgb((
                    (first_lab.0 * first_alpha + second_lab.0 * second_alpha) / alpha_sum,
                    (first_lab.1 * first_alpha + second_lab.1 * second_alpha) / alpha_sum,
                    (first_lab.2 * first_alpha + second_lab.2 * second_alpha) / alpha_sum,
                ))
            } else if oklch {
                let first_lab = super::color4::srgb_to_oklab(first.0);
                let second_lab = super::color4::srgb_to_oklab(second.0);
                let first_chroma = first_lab.1.hypot(first_lab.2);
                let second_chroma = second_lab.1.hypot(second_lab.2);
                let first_hue = first_lab
                    .2
                    .atan2(first_lab.1)
                    .to_degrees()
                    .rem_euclid(360.0);
                let second_hue = second_lab
                    .2
                    .atan2(second_lab.1)
                    .to_degrees()
                    .rem_euclid(360.0);
                let alpha_sum = first_alpha + second_alpha;
                let hue = interpolate_hue(
                    first_hue,
                    first_chroma < 0.0001,
                    second_hue,
                    second_chroma < 0.0001,
                    first_alpha,
                    second_alpha,
                );
                let lightness =
                    (first_lab.0 * first_alpha + second_lab.0 * second_alpha) / alpha_sum;
                let chroma =
                    (first_chroma * first_alpha + second_chroma * second_alpha) / alpha_sum;
                let hue = hue.to_radians();
                super::color4::oklab_to_srgb((lightness, chroma * hue.cos(), chroma * hue.sin()))
            } else if hsl {
                let first_hsl = super::rgb_to_hsl(first.0);
                let second_hsl = super::rgb_to_hsl(second.0);
                let alpha_sum = first_alpha + second_alpha;
                let hue = interpolate_hue(
                    first_hsl.0,
                    first_hsl.1 == 0.0,
                    second_hsl.0,
                    second_hsl.1 == 0.0,
                    first_alpha,
                    second_alpha,
                );
                let saturation =
                    (first_hsl.1 * first_alpha + second_hsl.1 * second_alpha) / alpha_sum;
                let lightness =
                    (first_hsl.2 * first_alpha + second_hsl.2 * second_alpha) / alpha_sum;
                super::hsl_to_rgb(hue, saturation, lightness)
            } else if hwb {
                let first_hwb = super::rgb_to_hwb(first.0);
                let second_hwb = super::rgb_to_hwb(second.0);
                let alpha_sum = first_alpha + second_alpha;
                let hue = interpolate_hue(
                    first_hwb.0,
                    first_hwb.1 + first_hwb.2 >= 1.0,
                    second_hwb.0,
                    second_hwb.1 + second_hwb.2 >= 1.0,
                    first_alpha,
                    second_alpha,
                );
                let whiteness =
                    (first_hwb.1 * first_alpha + second_hwb.1 * second_alpha) / alpha_sum;
                let blackness =
                    (first_hwb.2 * first_alpha + second_hwb.2 * second_alpha) / alpha_sum;
                super::hwb_to_rgb(hue, whiteness, blackness)
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

fn interpolate_hue(
    first_hue: f32,
    first_powerless: bool,
    second_hue: f32,
    second_powerless: bool,
    first_alpha: f32,
    second_alpha: f32,
) -> f32 {
    let first_hue = if first_powerless || first_alpha == 0.0 {
        second_hue
    } else {
        first_hue
    };
    let second_hue = if second_powerless || second_alpha == 0.0 {
        first_hue
    } else {
        second_hue
    };
    let hue_delta = (second_hue - first_hue + 180.0).rem_euclid(360.0) - 180.0;
    (first_hue + hue_delta * (second_alpha / (first_alpha + second_alpha))).rem_euclid(360.0)
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
