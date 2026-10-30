use cosmic_text::{Color, Renderer};

use super::{PAGE_PADDING, local_writing::LocalVerticalTransform};

pub(super) struct PixelRenderer<'a> {
    pub(super) pixels: &'a mut [u8],
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) skip_ink_mask: Option<&'a [u8]>,
    pub(super) local_transform: Option<LocalVerticalTransform>,
}

impl Renderer for PixelRenderer<'_> {
    fn rectangle(&mut self, x: i32, y: i32, width: u32, height: u32, color: Color) {
        let (x, y, width, height) = self.local_transform.map_or_else(
            || {
                (
                    x + PAGE_PADDING as i32,
                    y + PAGE_PADDING as i32,
                    width,
                    height,
                )
            },
            |transform| {
                let (left, top, mapped_width, mapped_height) = transform.map_rect(
                    x as f32 + PAGE_PADDING,
                    y as f32 + PAGE_PADDING,
                    width as f32,
                    height as f32,
                );
                (
                    left.round() as i32,
                    top.round() as i32,
                    mapped_width.round() as u32,
                    mapped_height.round() as u32,
                )
            },
        );
        let blend = if color.a() < 255 {
            blend_pixel_rounded
        } else {
            blend_pixel
        };
        for offset_y in 0..height as i32 {
            for offset_x in 0..width as i32 {
                let pixel_x = x + offset_x;
                let pixel_y = y + offset_y;
                if let Some(mask) = self.skip_ink_mask
                    && pixel_x >= 0
                    && pixel_y >= 0
                    && pixel_x < self.width as i32
                    && pixel_y < self.height as i32
                    && (0..=4).any(|offset_y| {
                        (-1..=1).any(|offset_x| {
                            let mask_x = pixel_x + offset_x;
                            let mask_y = pixel_y + offset_y;
                            mask_x >= 0
                                && mask_y >= 0
                                && mask_x < self.width as i32
                                && mask_y < self.height as i32
                                && mask[mask_y as usize * self.width as usize + mask_x as usize] > 0
                        })
                    })
                {
                    continue;
                }
                blend(
                    self.pixels,
                    self.width,
                    self.height,
                    pixel_x,
                    pixel_y,
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

impl<'a> PixelRenderer<'a> {
    pub(super) fn set_skip_ink_mask(&mut self, mask: Option<&'a [u8]>) {
        self.skip_ink_mask = mask;
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

pub(super) fn fill_rect_alpha(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: f32,
    y: f32,
    rect_width: f32,
    rect_height: f32,
    color: [u8; 4],
) {
    let left = ((x + PAGE_PADDING).floor() as i32).clamp(0, width as i32);
    let top = ((y + PAGE_PADDING).floor() as i32).clamp(0, height as i32);
    let right = ((x + rect_width + PAGE_PADDING).ceil() as i32).clamp(0, width as i32);
    let bottom = ((y + rect_height + PAGE_PADDING).ceil() as i32).clamp(0, height as i32);
    let color = Color::rgba(color[0], color[1], color[2], color[3]);
    for row in top..bottom {
        for column in left..right {
            blend_pixel(pixels, width, height, column, row, color, color.a());
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
