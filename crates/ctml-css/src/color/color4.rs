use cssparser::{Parser, ParserInput, Token};

pub(super) fn parse_color_function(
    value: &str,
    profiles: &[super::super::CssColorProfile],
) -> Option<([u8; 3], f32)> {
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    let function = parser.expect_function().ok()?.to_ascii_lowercase();
    if !matches!(
        function.as_str(),
        "lab" | "lch" | "oklab" | "oklch" | "color"
    ) {
        return None;
    }
    let color = parser
        .parse_nested_block(|input| match function.as_str() {
            "lab" => parse_lab(input, false),
            "lch" => parse_lab(input, true),
            "oklab" => parse_oklab(input, false),
            "oklch" => parse_oklab(input, true),
            "color" => parse_color_space(input, profiles),
            _ => Err(input.new_custom_error(())),
        })
        .ok()?;
    parser.is_exhausted().then_some(color)
}

fn parse_lab<'i, 't>(
    input: &mut Parser<'i, 't>,
    cylindrical: bool,
) -> Result<([u8; 3], f32), cssparser::ParseError<'i, ()>> {
    let lightness = parse_scaled_component(input, 100.0)?.clamp(0.0, 100.0);
    let second = parse_scaled_component(input, if cylindrical { 150.0 } else { 125.0 })?;
    let third = if cylindrical {
        parse_color_hue(input)?
    } else {
        parse_scaled_component(input, 125.0)?
    };
    let alpha = parse_color_alpha(input)?;
    let lab = if cylindrical {
        let hue = third.to_radians();
        let chroma = second.max(0.0);
        (lightness, chroma * hue.cos(), chroma * hue.sin())
    } else {
        (lightness, second, third)
    };
    if !input.is_exhausted() {
        return Err(input.new_custom_error(()));
    }
    Ok((lab_to_srgb(lab), alpha))
}

fn parse_oklab<'i, 't>(
    input: &mut Parser<'i, 't>,
    cylindrical: bool,
) -> Result<([u8; 3], f32), cssparser::ParseError<'i, ()>> {
    let lightness = parse_scaled_component(input, 1.0)?.clamp(0.0, 1.0);
    let second = parse_scaled_component(input, 0.4)?;
    let third = if cylindrical {
        parse_color_hue(input)?
    } else {
        parse_scaled_component(input, 0.4)?
    };
    let alpha = parse_color_alpha(input)?;
    let lab = if cylindrical {
        let hue = third.to_radians();
        let chroma = second.max(0.0);
        (lightness, chroma * hue.cos(), chroma * hue.sin())
    } else {
        (lightness, second, third)
    };
    if !input.is_exhausted() {
        return Err(input.new_custom_error(()));
    }
    Ok((oklab_to_srgb(lab), alpha))
}

fn parse_color_space<'i, 't>(
    input: &mut Parser<'i, 't>,
    profiles: &[super::super::CssColorProfile],
) -> Result<([u8; 3], f32), cssparser::ParseError<'i, ()>> {
    let space_name = input.expect_ident_cloned()?.to_string();
    if space_name.starts_with("--") {
        return parse_custom_color_space(input, profiles, &space_name);
    }
    let space = space_name.to_ascii_lowercase();
    let red = parse_scaled_component(input, 1.0)?;
    let green = parse_scaled_component(input, 1.0)?;
    let blue = parse_scaled_component(input, 1.0)?;
    let alpha = parse_color_alpha(input)?;
    if !input.is_exhausted() {
        return Err(input.new_custom_error(()));
    }
    let rgb = match space.as_str() {
        "srgb" => [red, green, blue].map(component_to_byte),
        "srgb-linear" => [red, green, blue].map(encode_srgb),
        "display-p3-linear" => display_p3_linear_to_srgb([red, green, blue]),
        "display-p3" => {
            let [red, green, blue] = [red, green, blue].map(decode_srgb);
            xyz_d65_to_srgb([
                0.48657095 * red + 0.26566769 * green + 0.19821729 * blue,
                0.22897456 * red + 0.69173852 * green + 0.07928691 * blue,
                0.04511338 * green + 1.04394437 * blue,
            ])
        }
        "a98-rgb" => a98_to_srgb([red, green, blue]),
        "prophoto-rgb" => prophoto_to_srgb([red, green, blue]),
        "rec2020" => rec2020_to_srgb([red, green, blue]),
        "rec2100-pq" => super::hdr::rec2100_pq_to_srgb([red, green, blue]),
        "rec2100-hlg" => super::hdr::rec2100_hlg_to_srgb([red, green, blue]),
        "rec2100-linear" => rec2020_linear_to_srgb([red, green, blue]),
        "xyz" | "xyz-d65" => xyz_d65_to_srgb([red, green, blue]),
        "xyz-d50" => xyz_d50_to_srgb([red, green, blue]),
        _ => return Err(input.new_custom_error(())),
    };
    Ok((rgb, alpha))
}

fn parse_custom_color_space<'i, 't>(
    input: &mut Parser<'i, 't>,
    profiles: &[super::super::CssColorProfile],
    name: &str,
) -> Result<([u8; 3], f32), cssparser::ParseError<'i, ()>> {
    let Some(profile) = profiles.iter().rev().find(|profile| profile.name == name) else {
        return Err(input.new_custom_error(()));
    };
    let Some(channel_count) = profile.channel_count() else {
        return Err(input.new_custom_error(()));
    };
    let mut components = Vec::with_capacity(channel_count);
    let mut component_count = 0;
    let mut alpha = 1.0;
    while !input.is_exhausted() {
        match input.next()? {
            Token::Delim('/') => {
                alpha = parse_color_alpha_value(input)?;
                break;
            }
            Token::Number { value, .. } if value.is_finite() => {
                if components.len() < channel_count {
                    components.push(value.clamp(0.0, 1.0));
                }
                component_count += 1;
            }
            Token::Percentage { unit_value, .. } if unit_value.is_finite() => {
                if components.len() < channel_count {
                    components.push(unit_value.clamp(0.0, 1.0));
                }
                component_count += 1;
            }
            Token::Ident(value) if value.eq_ignore_ascii_case("none") => {
                if components.len() < channel_count {
                    components.push(0.0);
                }
                component_count += 1;
            }
            _ => return Err(input.new_custom_error(())),
        }
    }
    if component_count == 0 {
        return Err(input.new_custom_error(()));
    }
    components.resize(channel_count, 0.0);
    if !input.is_exhausted() {
        return Err(input.new_custom_error(()));
    }
    let rgb = profile
        .to_srgb(&components)
        .ok_or_else(|| input.new_custom_error(()))?
        .map(component_to_byte);
    Ok((rgb, alpha))
}

fn parse_scaled_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    percentage_scale: f32,
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    match input.next()? {
        Token::Number { value, .. } if value.is_finite() => Ok(*value),
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => {
            Ok(*unit_value * percentage_scale)
        }
        Token::Ident(value) if value.eq_ignore_ascii_case("none") => Ok(0.0),
        _ => Err(input.new_custom_error(())),
    }
}

fn parse_color_hue<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        Ok(0.0)
    } else {
        super::parse_hue(input)
    }
}

fn parse_color_alpha<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    if input.is_exhausted() {
        return Ok(1.0);
    }
    input.expect_delim('/')?;
    parse_color_alpha_value(input)
}

fn parse_color_alpha_value<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<f32, cssparser::ParseError<'i, ()>> {
    let alpha = match input.next()? {
        Token::Number { value, .. } if value.is_finite() => *value,
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => *unit_value,
        Token::Ident(value) if value.eq_ignore_ascii_case("none") => 0.0,
        _ => return Err(input.new_custom_error(())),
    };
    Ok(alpha.clamp(0.0, 1.0))
}

pub(super) fn lab_to_srgb((lightness, a, b): (f32, f32, f32)) -> [u8; 3] {
    let fy = (lightness + 16.0) / 116.0;
    let fx = fy + a / 500.0;
    let fz = fy - b / 200.0;
    let xyz_d50 = [
        lab_inverse(fx) * 0.96422,
        lab_inverse(fy),
        lab_inverse(fz) * 0.82521,
    ];
    xyz_d50_to_srgb(xyz_d50)
}

pub(super) fn srgb_to_lab([red, green, blue]: [u8; 3]) -> (f32, f32, f32) {
    let [x, y, z] = srgb_to_xyz_d50([red, green, blue]);
    let fx = lab_forward(x / 0.96422);
    let fy = lab_forward(y);
    let fz = lab_forward(z / 0.82521);
    (116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz))
}

fn lab_forward(value: f32) -> f32 {
    if value > 216.0 / 24389.0 {
        value.cbrt()
    } else {
        (24389.0 / 27.0 * value + 16.0) / 116.0
    }
}

fn lab_inverse(value: f32) -> f32 {
    let cube = value * value * value;
    if cube > 216.0 / 24389.0 {
        cube
    } else {
        (116.0 * value - 16.0) / (24389.0 / 27.0)
    }
}

pub(super) fn oklab_to_srgb((lightness, a, b): (f32, f32, f32)) -> [u8; 3] {
    linear_srgb_to_bytes(gamut_map_linear_srgb(oklab_to_linear_srgb([
        lightness, a, b,
    ])))
}

pub(super) fn srgb_to_oklab([red, green, blue]: [u8; 3]) -> (f32, f32, f32) {
    let red = decode_srgb(red as f32 / 255.0);
    let green = decode_srgb(green as f32 / 255.0);
    let blue = decode_srgb(blue as f32 / 255.0);
    let l = (0.4122214708 * red + 0.5363325363 * green + 0.0514459929 * blue).cbrt();
    let m = (0.2119034982 * red + 0.6806995451 * green + 0.1073969566 * blue).cbrt();
    let s = (0.0883024619 * red + 0.2817188376 * green + 0.6299787005 * blue).cbrt();
    (
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    )
}

pub(super) fn xyz_d50_to_srgb([x, y, z]: [f32; 3]) -> [u8; 3] {
    let [x, y, z] = [
        0.955473421488075 * x - 0.02309845494876471 * y + 0.06325924320057072 * z,
        -0.0283697093338637 * x + 1.0099953980813041 * y + 0.021041441191917323 * z,
        0.012314014864481998 * x - 0.020507649298898964 * y + 1.330365926242124 * z,
    ];
    xyz_d65_to_srgb([x, y, z])
}

pub(super) fn xyz_d65_to_srgb([x, y, z]: [f32; 3]) -> [u8; 3] {
    let linear = [
        3.2409699 * x - 1.5373832 * y - 0.4986108 * z,
        -0.9692436 * x + 1.8759675 * y + 0.0415551 * z,
        0.0556301 * x - 0.203977 * y + 1.0569715 * z,
    ];
    linear_srgb_to_bytes(gamut_map_linear_srgb(linear))
}

fn linear_srgb_to_bytes(linear: [f32; 3]) -> [u8; 3] {
    linear.map(encode_srgb)
}

fn oklab_to_linear_srgb([lightness, a, b]: [f32; 3]) -> [f32; 3] {
    let l = (lightness + 0.39633778 * a + 0.21580376 * b).powi(3);
    let m = (lightness - 0.10556135 * a - 0.06385417 * b).powi(3);
    let s = (lightness - 0.08948418 * a - 1.29148555 * b).powi(3);
    [
        4.07674166 * l - 3.30771159 * m + 0.23096993 * s,
        -1.268438 * l + 2.6097574 * m - 0.3413194 * s,
        -0.00419609 * l - 0.70341861 * m + 1.7076147 * s,
    ]
}

fn linear_srgb_to_oklab([red, green, blue]: [f32; 3]) -> [f32; 3] {
    let l = (0.4122214708 * red + 0.5363325363 * green + 0.0514459929 * blue).cbrt();
    let m = (0.2119034982 * red + 0.6806995451 * green + 0.1073969566 * blue).cbrt();
    let s = (0.0883024619 * red + 0.2817188376 * green + 0.6299787005 * blue).cbrt();
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

fn gamut_map_linear_srgb(rgb: [f32; 3]) -> [f32; 3] {
    if in_srgb_gamut(rgb) {
        return rgb;
    }

    let origin = linear_srgb_to_oklab(rgb);
    if origin[0] <= 0.0 {
        return [0.0; 3];
    }
    if origin[0] >= 1.0 {
        return [1.0; 3];
    }

    let chroma = origin[1].hypot(origin[2]);
    let hue = origin[2].atan2(origin[1]);
    let mut current = origin;
    let mut clipped = clip_srgb(oklab_to_linear_srgb(current));
    const JND: f32 = 0.02;
    const EPSILON: f32 = 0.0001;
    if delta_e_ok(linear_srgb_to_oklab(clipped), current) < JND {
        return clipped;
    }

    let (mut min, mut max) = (0.0, chroma);
    let mut min_in_gamut = true;
    while max - min > EPSILON {
        let current_chroma = (min + max) / 2.0;
        current = [
            origin[0],
            current_chroma * hue.cos(),
            current_chroma * hue.sin(),
        ];
        let candidate = oklab_to_linear_srgb(current);
        if min_in_gamut && in_srgb_gamut(candidate) {
            min = current_chroma;
            continue;
        }
        clipped = clip_srgb(candidate);
        let difference = delta_e_ok(linear_srgb_to_oklab(clipped), current);
        if difference < JND {
            if JND - difference < EPSILON {
                return clipped;
            }
            min_in_gamut = false;
            min = current_chroma;
        } else {
            max = current_chroma;
        }
    }
    clipped
}

fn in_srgb_gamut(rgb: [f32; 3]) -> bool {
    rgb.into_iter()
        .all(|channel| (0.0..=1.0).contains(&channel))
}

fn clip_srgb(rgb: [f32; 3]) -> [f32; 3] {
    rgb.map(|channel| channel.clamp(0.0, 1.0))
}

fn delta_e_ok(first: [f32; 3], second: [f32; 3]) -> f32 {
    ((first[0] - second[0]).powi(2)
        + (first[1] - second[1]).powi(2)
        + (first[2] - second[2]).powi(2))
    .sqrt()
}

pub(super) fn srgb_to_xyz_d65([red, green, blue]: [u8; 3]) -> [f32; 3] {
    let red = decode_srgb(red as f32 / 255.0);
    let green = decode_srgb(green as f32 / 255.0);
    let blue = decode_srgb(blue as f32 / 255.0);
    [
        0.4123908 * red + 0.3575843 * green + 0.1804808 * blue,
        0.2126390 * red + 0.7151687 * green + 0.0721923 * blue,
        0.0193308 * red + 0.1191948 * green + 0.9505322 * blue,
    ]
}

pub(super) fn srgb_to_xyz_d50(rgb: [u8; 3]) -> [f32; 3] {
    xyz_d65_to_d50(srgb_to_xyz_d65(rgb))
}

pub(super) fn srgb_to_display_p3_linear(rgb: [u8; 3]) -> [f32; 3] {
    let [x, y, z] = srgb_to_xyz_d65(rgb);
    [
        2.49349691 * x - 0.93138362 * y - 0.40271078 * z,
        -0.82948897 * x + 1.76266406 * y + 0.02362469 * z,
        0.03584583 * x - 0.07617239 * y + 0.95688452 * z,
    ]
}

pub(super) fn display_p3_linear_to_srgb([red, green, blue]: [f32; 3]) -> [u8; 3] {
    xyz_d65_to_srgb([
        0.48657095 * red + 0.26566769 * green + 0.19821729 * blue,
        0.22897456 * red + 0.69173852 * green + 0.07928691 * blue,
        0.04511338 * green + 1.04394437 * blue,
    ])
}

pub(super) fn srgb_to_display_p3(rgb: [u8; 3]) -> [f32; 3] {
    srgb_to_display_p3_linear(rgb).map(encode_srgb_component)
}

pub(super) fn display_p3_to_srgb(components: [f32; 3]) -> [u8; 3] {
    display_p3_linear_to_srgb(components.map(decode_srgb))
}

fn encode_srgb_component(linear: f32) -> f32 {
    if linear <= 0.0031308 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

pub(super) fn a98_to_srgb([red, green, blue]: [f32; 3]) -> [u8; 3] {
    let linear = [red, green, blue].map(|value| signed_pow(value, 563.0 / 256.0));
    let [red, green, blue] = linear;
    xyz_d65_to_srgb([
        (573536.0 / 994567.0) * red + (263643.0 / 1420810.0) * green + (187206.0 / 994567.0) * blue,
        (591459.0 / 1989134.0) * red
            + (6239551.0 / 9945670.0) * green
            + (374412.0 / 4972835.0) * blue,
        (53769.0 / 1989134.0) * red
            + (351524.0 / 4972835.0) * green
            + (4929758.0 / 4972835.0) * blue,
    ])
}

pub(super) fn prophoto_to_srgb([red, green, blue]: [f32; 3]) -> [u8; 3] {
    let linear = [red, green, blue].map(decode_prophoto);
    let [red, green, blue] = linear;
    xyz_d50_to_srgb([
        0.79776664490064230 * red + 0.13518129740053308 * green + 0.03134773412839220 * blue,
        0.28807482881940130 * red + 0.71183523424187300 * green + 0.00008993693872564 * blue,
        0.82510460251046020 * blue,
    ])
}

pub(super) fn rec2020_to_srgb([red, green, blue]: [f32; 3]) -> [u8; 3] {
    rec2020_linear_to_srgb([red, green, blue].map(|value| signed_pow(value, 2.4)))
}

pub(super) fn rec2020_linear_to_srgb([red, green, blue]: [f32; 3]) -> [u8; 3] {
    xyz_d65_to_srgb([
        (63426534.0 / 99577255.0) * red
            + (20160776.0 / 139408157.0) * green
            + (47086771.0 / 278816314.0) * blue,
        (26158966.0 / 99577255.0) * red
            + (472592308.0 / 697040785.0) * green
            + (8267143.0 / 139408157.0) * blue,
        (19567812.0 / 697040785.0) * green + (295819943.0 / 278816314.0) * blue,
    ])
}

pub(super) fn srgb_to_a98(rgb: [u8; 3]) -> [f32; 3] {
    let [x, y, z] = srgb_to_xyz_d65(rgb);
    [
        (1829569.0 / 896150.0) * x - (506331.0 / 896150.0) * y - (308931.0 / 896150.0) * z,
        -(851781.0 / 878810.0) * x + (1648619.0 / 878810.0) * y + (36519.0 / 878810.0) * z,
        (16779.0 / 1248040.0) * x - (147721.0 / 1248040.0) * y + (1266979.0 / 1248040.0) * z,
    ]
    .map(|value| signed_pow(value, 256.0 / 563.0))
}

pub(super) fn srgb_to_prophoto(rgb: [u8; 3]) -> [f32; 3] {
    let [x, y, z] = xyz_d65_to_d50(srgb_to_xyz_d65(rgb));
    [
        1.3457868816471583 * x - 0.25557208737979464 * y - 0.05110186497554526 * z,
        -0.5446307051249019 * x + 1.5082477428451468 * y + 0.02052744743642139 * z,
        1.2119675456389452 * z,
    ]
    .map(encode_prophoto)
}

pub(super) fn srgb_to_rec2020(rgb: [u8; 3]) -> [f32; 3] {
    srgb_to_rec2020_linear(rgb).map(|value| signed_pow(value, 1.0 / 2.4))
}

pub(super) fn srgb_to_rec2020_linear(rgb: [u8; 3]) -> [f32; 3] {
    let [x, y, z] = srgb_to_xyz_d65(rgb);
    [
        (30757411.0 / 17917100.0) * x - (6372589.0 / 17917100.0) * y - (4539589.0 / 17917100.0) * z,
        -(19765991.0 / 29648200.0) * x
            + (47925759.0 / 29648200.0) * y
            + (467509.0 / 29648200.0) * z,
        (792561.0 / 44930125.0) * x - (1921689.0 / 44930125.0) * y + (42328811.0 / 44930125.0) * z,
    ]
}

fn xyz_d65_to_d50([x, y, z]: [f32; 3]) -> [f32; 3] {
    [
        1.0479297925449969 * x + 0.022946870601609652 * y - 0.05019226628920524 * z,
        0.02962780877005599 * x + 0.9904344267538799 * y - 0.017073799063418826 * z,
        -0.009243040646204504 * x + 0.015055191490298152 * y + 0.7518742814281371 * z,
    ]
}

fn decode_prophoto(value: f32) -> f32 {
    if value.abs() <= 16.0 / 512.0 {
        value / 16.0
    } else {
        signed_pow(value, 1.8)
    }
}

fn encode_prophoto(value: f32) -> f32 {
    if value.abs() < 1.0 / 512.0 {
        value * 16.0
    } else {
        signed_pow(value, 1.0 / 1.8)
    }
}

fn signed_pow(value: f32, exponent: f32) -> f32 {
    value.signum() * value.abs().powf(exponent)
}

pub(super) fn encode_srgb(linear: f32) -> u8 {
    let linear = linear.clamp(0.0, 1.0);
    let encoded = if linear <= 0.0031308 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round() as u8
}

pub(super) fn decode_srgb(encoded: f32) -> f32 {
    let encoded = encoded.clamp(0.0, 1.0);
    if encoded <= 0.04045 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}

fn component_to_byte(component: f32) -> u8 {
    (component.clamp(0.0, 1.0) * 255.0).round() as u8
}
