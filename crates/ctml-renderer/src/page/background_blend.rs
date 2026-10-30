use crate::style::BackgroundBlendMode;

use skia_safe::{AlphaType, ColorSpace, ColorType, Data, ImageInfo, Paint, Point, surfaces};

pub(super) fn to_skia(mode: BackgroundBlendMode) -> skia_safe::BlendMode {
    use skia_safe::BlendMode;
    match mode {
        BackgroundBlendMode::Normal => BlendMode::SrcOver,
        BackgroundBlendMode::Multiply => BlendMode::Multiply,
        BackgroundBlendMode::Screen => BlendMode::Screen,
        BackgroundBlendMode::Overlay => BlendMode::Overlay,
        BackgroundBlendMode::Darken => BlendMode::Darken,
        BackgroundBlendMode::Lighten => BlendMode::Lighten,
        BackgroundBlendMode::ColorDodge => BlendMode::ColorDodge,
        BackgroundBlendMode::ColorBurn => BlendMode::ColorBurn,
        BackgroundBlendMode::HardLight => BlendMode::HardLight,
        BackgroundBlendMode::SoftLight => BlendMode::SoftLight,
        BackgroundBlendMode::Difference => BlendMode::Difference,
        BackgroundBlendMode::Exclusion => BlendMode::Exclusion,
        BackgroundBlendMode::Hue => BlendMode::Hue,
        BackgroundBlendMode::Saturation => BlendMode::Saturation,
        BackgroundBlendMode::Color => BlendMode::Color,
        BackgroundBlendMode::Luminosity => BlendMode::Luminosity,
    }
}

pub(super) fn composite_source_layer(
    destination: &mut [u8],
    source: &[u8],
    opacity: f32,
    mode: BackgroundBlendMode,
    width: u32,
    height: u32,
) {
    if width == 0 || height == 0 {
        return;
    }
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        ColorSpace::new_srgb(),
    );
    let Some(image) =
        skia_safe::images::raster_from_data(&info, Data::new_copy(source), width as usize * 4)
    else {
        return;
    };
    let Some(mut surface) =
        surfaces::wrap_pixels(&info, destination, Some(width as usize * 4), None)
    else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_blend_mode(to_skia(mode));
    paint.set_alpha_f(opacity.clamp(0.0, 1.0));
    surface
        .canvas()
        .draw_image(&image, Point::new(0.0, 0.0), Some(&paint));
}
