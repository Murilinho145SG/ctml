use cosmic_text::{Color, Renderer};

use super::PAGE_PADDING;

pub(super) struct PixelRenderer<'a> {
    pub(super) pixels: &'a mut [u8],
    pub(super) width: u32,
    pub(super) height: u32,
}

impl Renderer for PixelRenderer<'_> {
    fn rectangle(&mut self, x: i32, y: i32, width: u32, height: u32, color: Color) {
        for offset_y in 0..height as i32 {
            for offset_x in 0..width as i32 {
                self.blend(
                    x + offset_x + PAGE_PADDING as i32,
                    y + offset_y + PAGE_PADDING as i32,
                    color,
                    color.a(),
                );
            }
        }
    }

    fn glyph(&mut self, _glyph: cosmic_text::PhysicalGlyph, _color: Color) {
        unreachable!("glyphs are rasterized through the alpha-aware page renderer")
    }
}

impl PixelRenderer<'_> {
    fn blend(&mut self, x: i32, y: i32, color: Color, alpha: u8) {
        blend_pixel(self.pixels, self.width, self.height, x, y, color, alpha);
    }
}

pub(super) fn blend_pixel(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    color: Color,
    alpha: u8,
) {
    blend_pixel_with_rounding(pixels, width, height, x, y, color, alpha, false);
}

pub(super) fn blend_pixel_rounded(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    color: Color,
    alpha: u8,
) {
    blend_pixel_with_rounding(pixels, width, height, x, y, color, alpha, true);
}

fn blend_pixel_with_rounding(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    color: Color,
    alpha: u8,
    round: bool,
) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 || alpha == 0 {
        return;
    }
    let offset = (y as usize * width as usize + x as usize) * 4;
    let rgba = color.as_rgba_tuple();
    let alpha = alpha as u16;
    for channel in 0..3 {
        pixels[offset + channel] = ((pixels[offset + channel] as u16 * (255 - alpha)
            + rgba_channel(rgba, channel) as u16 * alpha
            + if round { 127 } else { 0 })
            / 255) as u8;
    }
    pixels[offset + 3] = 255;
}

pub(super) fn fill_rect(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: f32,
    y: f32,
    rect_width: f32,
    rect_height: f32,
    color: [u8; 3],
) {
    let left = (x.floor() as i32).clamp(0, width as i32) as u32;
    let top = (y.floor() as i32).clamp(0, height as i32) as u32;
    let right = ((x + rect_width).ceil() as i32).clamp(0, width as i32) as u32;
    let bottom = ((y + rect_height).ceil() as i32).clamp(0, height as i32) as u32;
    for row in top..bottom {
        for column in left..right {
            let offset = (row as usize * width as usize + column as usize) * 4;
            pixels[offset..offset + 3].copy_from_slice(&color);
        }
    }
}

fn rgba_channel(rgba: (u8, u8, u8, u8), channel: usize) -> u8 {
    match channel {
        0 => rgba.0,
        1 => rgba.1,
        _ => rgba.2,
    }
}
