use lcms2::{ColorSpaceSignatureExt, Intent, PixelFormat, Profile, Transform};

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
    let output = match components.len() {
        1 => transform_one::<[f32; 1], [f32; 3]>(
            &profile,
            PixelFormat::GRAY_FLT,
            &srgb,
            PixelFormat::RGB_FLT,
            [components[0]],
            rendering_intent,
        )?,
        3 => transform_one::<[f32; 3], [f32; 3]>(
            &profile,
            PixelFormat::RGB_FLT,
            &srgb,
            PixelFormat::RGB_FLT,
            [components[0], components[1], components[2]],
            rendering_intent,
        )?,
        4 => transform_one::<[f32; 4], [f32; 3]>(
            &profile,
            PixelFormat::CMYK_FLT,
            &srgb,
            PixelFormat::RGB_FLT,
            [components[0], components[1], components[2], components[3]],
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
    let output = match profile.color_space().channels() {
        1 => transform_one::<[f32; 3], [f32; 1]>(
            &srgb,
            PixelFormat::RGB_FLT,
            &profile,
            PixelFormat::GRAY_FLT,
            components,
            rendering_intent,
        )?
        .to_vec(),
        3 => transform_one::<[f32; 3], [f32; 3]>(
            &srgb,
            PixelFormat::RGB_FLT,
            &profile,
            PixelFormat::RGB_FLT,
            components,
            rendering_intent,
        )?
        .to_vec(),
        4 => transform_one::<[f32; 3], [f32; 4]>(
            &srgb,
            PixelFormat::RGB_FLT,
            &profile,
            PixelFormat::CMYK_FLT,
            components,
            rendering_intent,
        )?
        .to_vec(),
        _ => return None,
    };
    Some(output)
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
    use super::{convert_from_srgb, convert_to_srgb, valid_icc_profile};
    use lcms2::Profile;

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
}
