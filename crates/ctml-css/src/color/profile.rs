use lcms2::{ColorSpaceSignature, ColorSpaceSignatureExt, Intent, PixelFormat, Profile, Transform};

pub(super) fn valid_icc_profile(data: &[u8]) -> Option<usize> {
    let profile = Profile::new_icc(data).ok()?;
    matches!(
        profile.device_class(),
        lcms2::ProfileClassSignature::InputClass
            | lcms2::ProfileClassSignature::DisplayClass
            | lcms2::ProfileClassSignature::OutputClass
            | lcms2::ProfileClassSignature::ColorSpaceClass
    )
    .then_some(profile.color_space().channels() as usize)
    .filter(|channels| (1..=15).contains(channels))
}

pub(super) fn convert_to_srgb(
    data: &[u8],
    components: &[f32],
    rendering_intent: &str,
) -> Option<[f32; 3]> {
    let profile = Profile::new_icc(data).ok()?;
    if components.len() != profile.color_space().channels() as usize {
        return None;
    }
    let srgb = Profile::new_srgb();
    let output = match profile.color_space() {
        ColorSpaceSignature::GrayData => transform_one::<[f32; 1], [f32; 3]>(
            &profile,
            PixelFormat::GRAY_FLT,
            &srgb,
            PixelFormat::RGB_FLT,
            [components[0]],
            rendering_intent,
        )?,
        ColorSpaceSignature::RgbData => transform_one::<[f32; 3], [f32; 3]>(
            &profile,
            PixelFormat::RGB_FLT,
            &srgb,
            PixelFormat::RGB_FLT,
            [components[0], components[1], components[2]],
            rendering_intent,
        )?,
        ColorSpaceSignature::CmykData => transform_one::<[f32; 4], [f32; 3]>(
            &profile,
            PixelFormat::CMYK_FLT,
            &srgb,
            PixelFormat::RGB_FLT,
            [components[0], components[1], components[2], components[3]],
            rendering_intent,
        )?,
        signature if is_multichannel(signature) => convert_multichannel_to_srgb(
            &profile,
            components,
            signature.channels() as usize,
            rendering_intent,
        )?,
        _ => return None,
    };
    Some(output)
}

pub(super) fn convert_from_srgb(
    data: &[u8],
    components: [f32; 3],
    rendering_intent: &str,
) -> Option<Vec<f32>> {
    let profile = Profile::new_icc(data).ok()?;
    let srgb = Profile::new_srgb();
    let output = match profile.color_space() {
        ColorSpaceSignature::GrayData => transform_one::<[f32; 3], [f32; 1]>(
            &srgb,
            PixelFormat::RGB_FLT,
            &profile,
            PixelFormat::GRAY_FLT,
            components,
            rendering_intent,
        )?
        .to_vec(),
        ColorSpaceSignature::RgbData => transform_one::<[f32; 3], [f32; 3]>(
            &srgb,
            PixelFormat::RGB_FLT,
            &profile,
            PixelFormat::RGB_FLT,
            components,
            rendering_intent,
        )?
        .to_vec(),
        ColorSpaceSignature::CmykData => transform_one::<[f32; 3], [f32; 4]>(
            &srgb,
            PixelFormat::RGB_FLT,
            &profile,
            PixelFormat::CMYK_FLT,
            components,
            rendering_intent,
        )?
        .to_vec(),
        signature if is_multichannel(signature) => convert_srgb_to_multichannel(
            &profile,
            components,
            signature.channels() as usize,
            rendering_intent,
        )?,
        _ => return None,
    };
    Some(output)
}

fn convert_multichannel_to_srgb(
    profile: &Profile,
    components: &[f32],
    channel_count: usize,
    rendering_intent: &str,
) -> Option<[f32; 3]> {
    let format = multichannel_float_format(channel_count)?;
    let mut input = Vec::with_capacity(channel_count * std::mem::size_of::<f32>());
    for component in components {
        input.extend_from_slice(&component.to_ne_bytes());
    }
    let srgb = Profile::new_srgb();
    let transform = Transform::<u8, [f32; 3]>::new(
        profile,
        format,
        &srgb,
        PixelFormat::RGB_FLT,
        parse_intent(rendering_intent)?,
    )
    .ok()?;
    let mut output = [[0.0; 3]];
    transform.transform_pixels(&input, &mut output);
    Some(output[0])
}

fn convert_srgb_to_multichannel(
    profile: &Profile,
    components: [f32; 3],
    channel_count: usize,
    rendering_intent: &str,
) -> Option<Vec<f32>> {
    let format = multichannel_float_format(channel_count)?;
    let srgb = Profile::new_srgb();
    let transform = Transform::<[f32; 3], u8>::new(
        &srgb,
        PixelFormat::RGB_FLT,
        profile,
        format,
        parse_intent(rendering_intent)?,
    )
    .ok()?;
    let mut output = vec![0; channel_count * std::mem::size_of::<f32>()];
    transform.transform_pixels(&[components], &mut output);
    Some(
        output
            .chunks_exact(std::mem::size_of::<f32>())
            .map(|bytes| Some(f32::from_ne_bytes(bytes.try_into().ok()?)))
            .collect::<Option<Vec<_>>>()?,
    )
}

fn multichannel_float_format(channel_count: usize) -> Option<PixelFormat> {
    if !(1..=15).contains(&channel_count) {
        return None;
    }
    let pixel_type = 14 + channel_count as u32;
    Some(PixelFormat(
        (pixel_type << 16) | ((channel_count as u32) << 3) | (1 << 22) | 4,
    ))
}

fn is_multichannel(signature: ColorSpaceSignature) -> bool {
    matches!(
        signature,
        ColorSpaceSignature::MCH1Data
            | ColorSpaceSignature::MCH2Data
            | ColorSpaceSignature::MCH3Data
            | ColorSpaceSignature::MCH4Data
            | ColorSpaceSignature::MCH5Data
            | ColorSpaceSignature::MCH6Data
            | ColorSpaceSignature::MCH7Data
            | ColorSpaceSignature::MCH8Data
            | ColorSpaceSignature::MCH9Data
            | ColorSpaceSignature::MCHAData
            | ColorSpaceSignature::MCHBData
            | ColorSpaceSignature::MCHCData
            | ColorSpaceSignature::MCHDData
            | ColorSpaceSignature::MCHEData
            | ColorSpaceSignature::MCHFData
    )
}

fn transform_one<Input: Copy + lcms2::Pod, Output: Copy + Default + lcms2::Pod>(
    input_profile: &Profile,
    input_format: PixelFormat,
    output_profile: &Profile,
    output_format: PixelFormat,
    input: Input,
    rendering_intent: &str,
) -> Option<Output> {
    let transform = Transform::<Input, Output>::new(
        input_profile,
        input_format,
        output_profile,
        output_format,
        parse_intent(rendering_intent)?,
    )
    .ok()?;
    let mut output = [Output::default()];
    transform.transform_pixels(&[input], &mut output);
    Some(output[0])
}

fn parse_intent(value: &str) -> Option<Intent> {
    match value {
        "relative-colorimetric" => Some(Intent::RelativeColorimetric),
        "absolute-colorimetric" => Some(Intent::AbsoluteColorimetric),
        "perceptual" => Some(Intent::Perceptual),
        "saturation" => Some(Intent::Saturation),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{convert_from_srgb, convert_to_srgb, multichannel_float_format, valid_icc_profile};
    use lcms2::{Profile, ToneCurve};

    #[test]
    fn validates_and_transforms_rgb_icc_profiles() {
        let bytes = Profile::new_srgb()
            .icc()
            .expect("sRGB ICC data should serialize");
        assert_eq!(valid_icc_profile(&bytes), Some(3));
        let converted = convert_to_srgb(&bytes, &[1.0, 0.0, 0.0], "relative-colorimetric")
            .expect("valid RGB profile should convert to sRGB");
        assert!((converted[0] - 1.0).abs() < 0.001);
        assert!(converted[1].abs() < 0.001);
        assert!(converted[2].abs() < 0.001);
        let round_trip = convert_from_srgb(&bytes, [1.0, 0.0, 0.0], "relative-colorimetric")
            .expect("sRGB should convert into a valid RGB profile");
        assert_eq!(round_trip.len(), 3);
        assert!((round_trip[0] - 1.0).abs() < 0.001);
        assert!(round_trip[1].abs() < 0.001);
        assert!(round_trip[2].abs() < 0.001);
    }

    #[test]
    fn validates_and_transforms_grayscale_icc_profiles() {
        let white_point = lcms2::white_point_from_temp(5000.0)
            .expect("valid temperature should produce a white point");
        let curve = ToneCurve::new(2.2);
        let profile = Profile::new_gray(&white_point, &curve)
            .expect("Little CMS should create a grayscale profile");
        let bytes = profile.icc().expect("grayscale profile should serialize");

        assert_eq!(valid_icc_profile(&bytes), Some(1));
        let converted = convert_to_srgb(&bytes, &[0.5], "relative-colorimetric")
            .expect("grayscale ICC profiles should convert to sRGB");
        assert!(converted[0] > 0.4 && converted[0] < 0.6);
        assert!((converted[0] - converted[1]).abs() < 0.001);
        assert!((converted[1] - converted[2]).abs() < 0.001);
        let round_trip = convert_from_srgb(&bytes, converted, "relative-colorimetric")
            .expect("sRGB should convert into a grayscale ICC profile");
        assert_eq!(round_trip.len(), 1);
        assert!((round_trip[0] - 0.5).abs() < 0.01);
    }

    #[test]
    fn builds_float_pixel_formats_for_icc_multichannel_spaces() {
        for channels in 1..=15 {
            let format = multichannel_float_format(channels)
                .expect("supported multichannel count should have a pixel format");
            assert_eq!(format.channels(), channels);
            assert_eq!(
                format.bytes_per_pixel(),
                channels * std::mem::size_of::<f32>()
            );
        }
        assert!(multichannel_float_format(0).is_none());
        assert!(multichannel_float_format(16).is_none());
    }
}
