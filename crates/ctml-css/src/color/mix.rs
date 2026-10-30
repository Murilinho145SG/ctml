use cssparser::{Parser, ParserInput, Token};

use super::{
    missing::{MissingComponents, component_space, parse_missing_components},
    parse_color_with_opacity_depth,
};
mod hue;
use hue::{HueInterpolation, interpolate_hue};

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
            let component_space = component_space(&interpolation_space.to_ascii_lowercase());
            let linear_srgb = interpolation_space.eq_ignore_ascii_case("srgb-linear");
            let lab = interpolation_space.eq_ignore_ascii_case("lab");
            let lch = interpolation_space.eq_ignore_ascii_case("lch");
            let oklab = interpolation_space.eq_ignore_ascii_case("oklab");
            let oklch = interpolation_space.eq_ignore_ascii_case("oklch");
            let xyz_d50 = interpolation_space.eq_ignore_ascii_case("xyz-d50");
            let xyz_d65 = interpolation_space.eq_ignore_ascii_case("xyz")
                || interpolation_space.eq_ignore_ascii_case("xyz-d65");
            let display_p3 = interpolation_space.eq_ignore_ascii_case("display-p3");
            let display_p3_linear = interpolation_space.eq_ignore_ascii_case("display-p3-linear");
            let a98_rgb = interpolation_space.eq_ignore_ascii_case("a98-rgb");
            let prophoto_rgb = interpolation_space.eq_ignore_ascii_case("prophoto-rgb");
            let rec2020 = interpolation_space.eq_ignore_ascii_case("rec2020");
            let rec2020_linear = interpolation_space.eq_ignore_ascii_case("rec2100-linear");
            let hsl = interpolation_space.eq_ignore_ascii_case("hsl");
            let hwb = interpolation_space.eq_ignore_ascii_case("hwb");
            let srgb = interpolation_space.eq_ignore_ascii_case("srgb");
            if !interpolation_space.eq_ignore_ascii_case("srgb")
                && !linear_srgb
                && !lab
                && !lch
                && !oklab
                && !oklch
                && !xyz_d50
                && !xyz_d65
                && !display_p3
                && !display_p3_linear
                && !a98_rgb
                && !prophoto_rgb
                && !rec2020
                && !rec2020_linear
                && !hsl
                && !hwb
            {
                return Err(input.new_custom_error(()));
            }
            let hue_interpolation = if lch || oklch || hsl || hwb {
                if let Ok(method) = input.try_parse(|input| input.expect_ident_cloned()) {
                    input.expect_ident_matching("hue")?;
                    match method.to_ascii_lowercase().as_str() {
                        "shorter" => HueInterpolation::Shorter,
                        "longer" => HueInterpolation::Longer,
                        "increasing" => HueInterpolation::Increasing,
                        "decreasing" => HueInterpolation::Decreasing,
                        _ => return Err(input.new_custom_error(())),
                    }
                } else {
                    HueInterpolation::Shorter
                }
            } else {
                HueInterpolation::Shorter
            };
            input.expect_comma()?;

            let (first, first_percentage, first_missing) = parse_mix_component(input, depth)?;
            input.expect_comma()?;
            let (second, second_percentage, second_missing) = parse_mix_component(input, depth)?;
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
            let (mut first, mut second) = (first, second);
            let (first_missing_alpha, second_missing_alpha) =
                (first_missing.alpha, second_missing.alpha);
            let first_missing = first_missing.channels_for(component_space);
            let second_missing = second_missing.channels_for(component_space);
            match (first_missing_alpha, second_missing_alpha) {
                (true, true) => {
                    first.1 = 0.0;
                    second.1 = 0.0;
                }
                (true, false) => first.1 = second.1,
                (false, true) => second.1 = first.1,
                (false, false) => {}
            }
            if srgb {
                for index in 0..3 {
                    match (first_missing[index], second_missing[index]) {
                        (true, true) => {
                            first.0[index] = 0;
                            second.0[index] = 0;
                        }
                        (true, false) => first.0[index] = second.0[index],
                        (false, true) => second.0[index] = first.0[index],
                        (false, false) => {}
                    }
                }
            }
            let first_alpha = first.1 * first_weight;
            let second_alpha = second.1 * second_weight;
            let alpha = (first_alpha + second_alpha) * (weight_sum / 100.0).min(1.0);
            let channels = if alpha == 0.0 {
                [0; 3]
            } else if display_p3 {
                let mut first_p3 = super::color4::srgb_to_display_p3(first.0);
                let mut second_p3 = super::color4::srgb_to_display_p3(second.0);
                carry_missing_components(
                    &mut first_p3,
                    &mut second_p3,
                    first_missing,
                    second_missing,
                );
                let alpha_sum = first_alpha + second_alpha;
                let mixed_p3 = std::array::from_fn(|index| {
                    (first_p3[index] * first_alpha + second_p3[index] * second_alpha) / alpha_sum
                });
                super::color4::display_p3_to_srgb(mixed_p3)
            } else if display_p3_linear {
                let mut first_p3 = super::color4::srgb_to_display_p3_linear(first.0);
                let mut second_p3 = super::color4::srgb_to_display_p3_linear(second.0);
                carry_missing_components(
                    &mut first_p3,
                    &mut second_p3,
                    first_missing,
                    second_missing,
                );
                let alpha_sum = first_alpha + second_alpha;
                let mixed_p3 = std::array::from_fn(|index| {
                    (first_p3[index] * first_alpha + second_p3[index] * second_alpha) / alpha_sum
                });
                super::color4::display_p3_linear_to_srgb(mixed_p3)
            } else if a98_rgb || prophoto_rgb || rec2020 || rec2020_linear {
                let (convert, to_srgb): (fn([u8; 3]) -> [f32; 3], fn([f32; 3]) -> [u8; 3]) =
                    if a98_rgb {
                        (super::color4::srgb_to_a98, super::color4::a98_to_srgb)
                    } else if prophoto_rgb {
                        (
                            super::color4::srgb_to_prophoto,
                            super::color4::prophoto_to_srgb,
                        )
                    } else if rec2020 {
                        (
                            super::color4::srgb_to_rec2020,
                            super::color4::rec2020_to_srgb,
                        )
                    } else {
                        (
                            super::color4::srgb_to_rec2020_linear,
                            super::color4::rec2020_linear_to_srgb,
                        )
                    };
                let mut first_channels = convert(first.0);
                let mut second_channels = convert(second.0);
                carry_missing_components(
                    &mut first_channels,
                    &mut second_channels,
                    first_missing,
                    second_missing,
                );
                let alpha_sum = first_alpha + second_alpha;
                let mixed = std::array::from_fn(|index| {
                    (first_channels[index] * first_alpha + second_channels[index] * second_alpha)
                        / alpha_sum
                });
                to_srgb(mixed)
            } else if xyz_d50 || xyz_d65 {
                let convert = if xyz_d50 {
                    super::color4::srgb_to_xyz_d50
                } else {
                    super::color4::srgb_to_xyz_d65
                };
                let mut first_xyz = convert(first.0);
                let mut second_xyz = convert(second.0);
                carry_missing_components(
                    &mut first_xyz,
                    &mut second_xyz,
                    first_missing,
                    second_missing,
                );
                let alpha_sum = first_alpha + second_alpha;
                let mixed_xyz = std::array::from_fn(|index| {
                    (first_xyz[index] * first_alpha + second_xyz[index] * second_alpha) / alpha_sum
                });
                if xyz_d50 {
                    super::color4::xyz_d50_to_srgb(mixed_xyz)
                } else {
                    super::color4::xyz_d65_to_srgb(mixed_xyz)
                }
            } else if lab {
                let first_lab = super::color4::srgb_to_lab(first.0);
                let second_lab = super::color4::srgb_to_lab(second.0);
                let mut first_lab = [first_lab.0, first_lab.1, first_lab.2];
                let mut second_lab = [second_lab.0, second_lab.1, second_lab.2];
                carry_missing_components(
                    &mut first_lab,
                    &mut second_lab,
                    first_missing,
                    second_missing,
                );
                let alpha_sum = first_alpha + second_alpha;
                super::color4::lab_to_srgb((
                    (first_lab[0] * first_alpha + second_lab[0] * second_alpha) / alpha_sum,
                    (first_lab[1] * first_alpha + second_lab[1] * second_alpha) / alpha_sum,
                    (first_lab[2] * first_alpha + second_lab[2] * second_alpha) / alpha_sum,
                ))
            } else if lch {
                let first_lab = super::color4::srgb_to_lab(first.0);
                let second_lab = super::color4::srgb_to_lab(second.0);
                let mut first_lch = [
                    first_lab.0,
                    first_lab.1.hypot(first_lab.2),
                    first_lab
                        .2
                        .atan2(first_lab.1)
                        .to_degrees()
                        .rem_euclid(360.0),
                ];
                let mut second_lch = [
                    second_lab.0,
                    second_lab.1.hypot(second_lab.2),
                    second_lab
                        .2
                        .atan2(second_lab.1)
                        .to_degrees()
                        .rem_euclid(360.0),
                ];
                carry_missing_components(
                    &mut first_lch,
                    &mut second_lch,
                    first_missing,
                    second_missing,
                );
                let alpha_sum = first_alpha + second_alpha;
                let hue = interpolate_hue(
                    first_lch[2],
                    first_lch[1] < 0.0001 || first_missing[2],
                    second_lch[2],
                    second_lch[1] < 0.0001 || second_missing[2],
                    first_alpha,
                    second_alpha,
                    hue_interpolation,
                );
                let lightness =
                    (first_lch[0] * first_alpha + second_lch[0] * second_alpha) / alpha_sum;
                let chroma =
                    (first_lch[1] * first_alpha + second_lch[1] * second_alpha) / alpha_sum;
                let hue = hue.to_radians();
                super::color4::lab_to_srgb((lightness, chroma * hue.cos(), chroma * hue.sin()))
            } else if oklab {
                let first_lab = super::color4::srgb_to_oklab(first.0);
                let second_lab = super::color4::srgb_to_oklab(second.0);
                let mut first_lab = [first_lab.0, first_lab.1, first_lab.2];
                let mut second_lab = [second_lab.0, second_lab.1, second_lab.2];
                carry_missing_components(
                    &mut first_lab,
                    &mut second_lab,
                    first_missing,
                    second_missing,
                );
                let alpha_sum = first_alpha + second_alpha;
                super::color4::oklab_to_srgb((
                    (first_lab[0] * first_alpha + second_lab[0] * second_alpha) / alpha_sum,
                    (first_lab[1] * first_alpha + second_lab[1] * second_alpha) / alpha_sum,
                    (first_lab[2] * first_alpha + second_lab[2] * second_alpha) / alpha_sum,
                ))
            } else if oklch {
                let first_lab = super::color4::srgb_to_oklab(first.0);
                let second_lab = super::color4::srgb_to_oklab(second.0);
                let mut first_lch = [
                    first_lab.0,
                    first_lab.1.hypot(first_lab.2),
                    first_lab
                        .2
                        .atan2(first_lab.1)
                        .to_degrees()
                        .rem_euclid(360.0),
                ];
                let mut second_lch = [
                    second_lab.0,
                    second_lab.1.hypot(second_lab.2),
                    second_lab
                        .2
                        .atan2(second_lab.1)
                        .to_degrees()
                        .rem_euclid(360.0),
                ];
                carry_missing_components(
                    &mut first_lch,
                    &mut second_lch,
                    first_missing,
                    second_missing,
                );
                let alpha_sum = first_alpha + second_alpha;
                let hue = interpolate_hue(
                    first_lch[2],
                    first_lch[1] < 0.0001 || first_missing[2],
                    second_lch[2],
                    second_lch[1] < 0.0001 || second_missing[2],
                    first_alpha,
                    second_alpha,
                    hue_interpolation,
                );
                let lightness =
                    (first_lch[0] * first_alpha + second_lch[0] * second_alpha) / alpha_sum;
                let chroma =
                    (first_lch[1] * first_alpha + second_lch[1] * second_alpha) / alpha_sum;
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
                    hue_interpolation,
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
                    hue_interpolation,
                );
                let whiteness =
                    (first_hwb.1 * first_alpha + second_hwb.1 * second_alpha) / alpha_sum;
                let blackness =
                    (first_hwb.2 * first_alpha + second_hwb.2 * second_alpha) / alpha_sum;
                super::hwb_to_rgb(hue, whiteness, blackness)
            } else if linear_srgb {
                let mut first_linear = first
                    .0
                    .map(|channel| super::color4::decode_srgb(channel as f32 / 255.0));
                let mut second_linear = second
                    .0
                    .map(|channel| super::color4::decode_srgb(channel as f32 / 255.0));
                carry_missing_components(
                    &mut first_linear,
                    &mut second_linear,
                    first_missing,
                    second_missing,
                );
                std::array::from_fn(|index| {
                    let channel = (first_linear[index] * first_alpha
                        + second_linear[index] * second_alpha)
                        / (first_alpha + second_alpha);
                    super::color4::encode_srgb(channel)
                })
            } else {
                std::array::from_fn(|index| {
                    ((first.0[index] as f32 * first_alpha + second.0[index] as f32 * second_alpha)
                        / (first_alpha + second_alpha))
                        .round()
                        .clamp(0.0, 255.0) as u8
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
) -> Result<(([u8; 3], f32), Option<f32>, MissingComponents), cssparser::ParseError<'i, ()>> {
    let start = input.position();
    match input.next()? {
        Token::Function(_) => input.parse_nested_block(|nested| consume_all_tokens(nested, 0))?,
        Token::ParenthesisBlock => {
            input.parse_nested_block(|nested| consume_all_tokens(nested, 0))?
        }
        _ => {}
    }
    let color_end = input.position();
    let color_value = input.slice(start..color_end).trim();
    let color = parse_color_with_opacity_depth(color_value, depth + 1)
        .ok_or_else(|| input.new_custom_error(()))?;
    let missing = parse_missing_components(color_value);
    let percentage = input
        .try_parse(|input| input.expect_percentage())
        .ok()
        .map(|percentage| percentage * 100.0);
    if percentage.is_some_and(|percentage| !percentage.is_finite() || percentage < 0.0) {
        return Err(input.new_custom_error(()));
    }
    Ok((color, percentage, missing))
}

fn carry_missing_components(
    first: &mut [f32; 3],
    second: &mut [f32; 3],
    first_missing: [bool; 3],
    second_missing: [bool; 3],
) {
    for index in 0..3 {
        match (first_missing[index], second_missing[index]) {
            (true, true) => {
                first[index] = 0.0;
                second[index] = 0.0;
            }
            (true, false) => first[index] = second[index],
            (false, true) => second[index] = first[index],
            (false, false) => {}
        }
    }
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
