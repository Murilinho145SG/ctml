use crate::style::BackgroundBlendMode;

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
