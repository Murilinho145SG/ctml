#[derive(Clone, Copy)]
pub(super) enum HueInterpolation {
    Shorter,
    Longer,
    Increasing,
    Decreasing,
}

pub(super) fn interpolate_hue(
    first_hue: f32,
    first_powerless: bool,
    second_hue: f32,
    second_powerless: bool,
    first_alpha: f32,
    second_alpha: f32,
    interpolation: HueInterpolation,
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
    let difference = (second_hue - first_hue).rem_euclid(360.0);
    let hue_delta = match interpolation {
        HueInterpolation::Shorter => {
            if difference > 180.0 {
                difference - 360.0
            } else {
                difference
            }
        }
        HueInterpolation::Longer => {
            if difference == 0.0 {
                360.0
            } else if difference < 180.0 {
                difference - 360.0
            } else {
                difference
            }
        }
        HueInterpolation::Increasing => difference,
        HueInterpolation::Decreasing => {
            if difference == 0.0 {
                0.0
            } else {
                difference - 360.0
            }
        }
    };
    (first_hue + hue_delta * (second_alpha / (first_alpha + second_alpha))).rem_euclid(360.0)
}
