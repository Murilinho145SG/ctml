use super::color4::{rec2020_linear_to_srgb, srgb_to_rec2020_linear};

pub(super) fn srgb_to_rec2100_pq(rgb: [u8; 3]) -> [f32; 3] {
    srgb_to_rec2020_linear(rgb).map(linear_to_pq)
}

pub(super) fn rec2100_pq_to_srgb(components: [f32; 3]) -> [u8; 3] {
    rec2020_linear_to_srgb(components.map(pq_to_linear))
}

pub(super) fn srgb_to_rec2100_hlg(rgb: [u8; 3]) -> [f32; 3] {
    srgb_to_rec2020_linear(rgb).map(linear_to_hlg)
}

pub(super) fn rec2100_hlg_to_srgb(components: [f32; 3]) -> [u8; 3] {
    rec2020_linear_to_srgb(components.map(hlg_to_linear))
}

fn linear_to_pq(value: f32) -> f32 {
    const N: f32 = 2610.0 / 16384.0;
    const M: f32 = 2523.0 / 32.0;
    const C1: f32 = 3424.0 / 4096.0;
    const C2: f32 = 2413.0 / 128.0;
    const C3: f32 = 2392.0 / 128.0;
    let scaled_luminance = (value * 203.0 / 10_000.0).max(0.0);
    let power = scaled_luminance.powf(N);
    ((C1 + C2 * power) / (1.0 + C3 * power)).powf(M)
}

fn pq_to_linear(value: f32) -> f32 {
    const N_INVERSE: f32 = 16384.0 / 2610.0;
    const M_INVERSE: f32 = 32.0 / 2523.0;
    const C1: f32 = 3424.0 / 4096.0;
    const C2: f32 = 2413.0 / 128.0;
    const C3: f32 = 2392.0 / 128.0;
    let powered = value.clamp(0.0, 1.0).powf(M_INVERSE);
    let luminance = ((powered - C1).max(0.0) / (C2 - C3 * powered)).powf(N_INVERSE);
    luminance * 10_000.0 / 203.0
}

fn linear_to_hlg(value: f32) -> f32 {
    const SCALE: f32 = 3.7743;
    const A: f32 = 0.17883277;
    const B: f32 = 0.28466892;
    const C: f32 = 0.55991073;
    let value = value / SCALE;
    let sign = value.signum();
    let magnitude = value.abs();
    sign * if magnitude <= 1.0 / 12.0 {
        (3.0 * magnitude).sqrt()
    } else {
        A * (12.0 * magnitude - B).ln() + C
    }
}

fn hlg_to_linear(value: f32) -> f32 {
    const SCALE: f32 = 3.7743;
    const A: f32 = 0.17883277;
    const B: f32 = 0.28466892;
    const C: f32 = 0.55991073;
    let magnitude = value.abs();
    let decoded = if magnitude <= 0.5 {
        magnitude * magnitude / 3.0
    } else {
        ((magnitude - C) / A).exp().mul_add(1.0, B) / 12.0
    } * SCALE;
    value.signum() * decoded
}
