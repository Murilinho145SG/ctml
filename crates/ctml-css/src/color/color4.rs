use cssparser::{Parser, ParserInput, Token};

pub(super) fn parse_color_function(value: &str) -> Option<([u8; 3], f32)> {
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
            "color" => parse_color_space(input),
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
        super::parse_hue(input)?
    } else {
        parse_scaled_component(input, 125.0)?
    };
    let alpha = parse_alpha(input)?;
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
        super::parse_hue(input)?
    } else {
        parse_scaled_component(input, 0.4)?
    };
    let alpha = parse_alpha(input)?;
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
) -> Result<([u8; 3], f32), cssparser::ParseError<'i, ()>> {
    let space = input.expect_ident_cloned()?.to_ascii_lowercase();
    let red = parse_scaled_component(input, 1.0)?;
    let green = parse_scaled_component(input, 1.0)?;
    let blue = parse_scaled_component(input, 1.0)?;
    let alpha = parse_alpha(input)?;
    if !input.is_exhausted() {
        return Err(input.new_custom_error(()));
    }
    let rgb = match space.as_str() {
        "srgb" => [red, green, blue].map(component_to_byte),
        "srgb-linear" => [red, green, blue].map(encode_srgb),
        "display-p3" => {
            let [red, green, blue] = [red, green, blue].map(decode_srgb);
            xyz_d65_to_srgb([
                0.48657095 * red + 0.26566769 * green + 0.19821729 * blue,
                0.22897456 * red + 0.69173852 * green + 0.07928691 * blue,
                0.04511338 * green + 1.04394437 * blue,
            ])
        }
        "xyz" | "xyz-d65" => xyz_d65_to_srgb([red, green, blue]),
        "xyz-d50" => xyz_d50_to_srgb([red, green, blue]),
        _ => return Err(input.new_custom_error(())),
    };
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
        _ => Err(input.new_custom_error(())),
    }
}

fn parse_alpha<'i, 't>(input: &mut Parser<'i, 't>) -> Result<f32, cssparser::ParseError<'i, ()>> {
    if input.is_exhausted() {
        return Ok(1.0);
    }
    input.expect_delim('/')?;
    super::parse_alpha(input)
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
    let red = decode_srgb(red as f32 / 255.0);
    let green = decode_srgb(green as f32 / 255.0);
    let blue = decode_srgb(blue as f32 / 255.0);
    let xyz_d65 = [
        0.4123908 * red + 0.3575843 * green + 0.1804808 * blue,
        0.2126390 * red + 0.7151687 * green + 0.0721923 * blue,
        0.0193308 * red + 0.1191948 * green + 0.9505322 * blue,
    ];
    let [x, y, z] = [
        1.0478112 * xyz_d65[0] + 0.0228866 * xyz_d65[1] - 0.0501270 * xyz_d65[2],
        0.0295424 * xyz_d65[0] + 0.9904844 * xyz_d65[1] - 0.0170491 * xyz_d65[2],
        -0.0092345 * xyz_d65[0] + 0.0150436 * xyz_d65[1] + 0.7521316 * xyz_d65[2],
    ];
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
    let l = (lightness + 0.39633778 * a + 0.21580376 * b).powi(3);
    let m = (lightness - 0.10556135 * a - 0.06385417 * b).powi(3);
    let s = (lightness - 0.08948418 * a - 1.29148555 * b).powi(3);
    [
        4.07674166 * l - 3.30771159 * m + 0.23096993 * s,
        -1.268438 * l + 2.6097574 * m - 0.3413194 * s,
        -0.00419609 * l - 0.70341861 * m + 1.7076147 * s,
    ]
    .map(encode_srgb)
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

fn xyz_d50_to_srgb([x, y, z]: [f32; 3]) -> [u8; 3] {
    let xyz_d65 = [
        0.9555766 * x - 0.0230393 * y + 0.0631636 * z,
        -0.0282895 * x + 1.0099416 * y + 0.0210077 * z,
        0.0122982 * x - 0.020483 * y + 1.3299098 * z,
    ];
    xyz_d65_to_srgb(xyz_d65)
}

fn xyz_d65_to_srgb([x, y, z]: [f32; 3]) -> [u8; 3] {
    [
        3.2409699 * x - 1.5373832 * y - 0.4986108 * z,
        -0.9692436 * x + 1.8759675 * y + 0.0415551 * z,
        0.0556301 * x - 0.203977 * y + 1.0569715 * z,
    ]
    .map(encode_srgb)
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
