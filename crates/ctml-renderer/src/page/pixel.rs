use cosmic_text::{Color, Renderer};

use super::{local_writing::LocalVerticalTransform, overflow_clip::ClipRect, page_padding};

pub(super) struct PixelRenderer<'a> {
    pub(super) pixels: &'a mut [u8],
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) skip_ink_mask: Option<&'a [u8]>,
    pub(super) local_transform: Option<LocalVerticalTransform>,
    pub(super) clip_rect: Option<ClipRect>,
    pub(super) translate_x: f32,
    pub(super) translate_y: f32,
    pub(super) decoration_opacity_scale: f32,
}

impl Renderer for PixelRenderer<'_> {
    fn rectangle(&mut self, x: i32, y: i32, width: u32, height: u32, color: Color) {
        let (x, y, width, height) = self.local_transform.map_or_else(
            || {
                (
                    x + page_padding() as i32,
                    y + page_padding() as i32,
                    width,
                    height,
                )
            },
            |transform| {
                let (left, top, mapped_width, mapped_height) = transform.map_rect(
                    x as f32 + page_padding(),
                    y as f32 + page_padding(),
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
        let x = x - self.translate_x.round() as i32;
        let y = y - self.translate_y.round() as i32;
        let color = cosmic_text::Color::rgba(
            color.r(),
            color.g(),
            color.b(),
            (f32::from(color.a()) * self.decoration_opacity_scale)
                .round()
                .clamp(0.0, 255.0) as u8,
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
                if self.clip_rect.is_some_and(|clip| {
                    (pixel_x as f32) < clip.left
                        || (pixel_x as f32) >= clip.right
                        || (pixel_y as f32) < clip.top
                        || (pixel_y as f32) >= clip.bottom
                }) {
                    continue;
                }
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

    pub(super) fn set_clip_rect(&mut self, clip_rect: Option<ClipRect>) {
        self.clip_rect = clip_rect;
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
    let source_alpha = f32::from(alpha) / 255.0;
    let destination_alpha = f32::from(pixels[offset + 3]) / 255.0;
    let output_alpha = source_alpha + destination_alpha * (1.0 - source_alpha);
    for channel in 0..3 {
        let source = f32::from(rgba_channel(rgba, channel)) * source_alpha;
        let destination = f32::from(pixels[offset + channel]);
        let value = source + destination * (1.0 - source_alpha);
        pixels[offset + channel] = if round {
            value.round().clamp(0.0, 255.0) as u8
        } else {
            value.floor().clamp(0.0, 255.0) as u8
        };
    }
    pixels[offset + 3] = (output_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
}

pub(super) fn fill_rect_alpha_clipped(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: f32,
    y: f32,
    rect_width: f32,
    rect_height: f32,
    color: [u8; 4],
    clip: Option<ClipRect>,
) {
    let left = ((x + page_padding()).floor() as i32).clamp(0, width as i32);
    let top = ((y + page_padding()).floor() as i32).clamp(0, height as i32);
    let right = ((x + rect_width + page_padding()).ceil() as i32).clamp(0, width as i32);
    let bottom = ((y + rect_height + page_padding()).ceil() as i32).clamp(0, height as i32);
    let left = clip.map_or(left, |clip| left.max(clip.left.floor() as i32));
    let top = clip.map_or(top, |clip| top.max(clip.top.floor() as i32));
    let right = clip.map_or(right, |clip| right.min(clip.right.ceil() as i32));
    let bottom = clip.map_or(bottom, |clip| bottom.min(clip.bottom.ceil() as i32));
    if right <= left || bottom <= top || color[3] == 0 {
        return;
    }
    if color[3] == 255 {
        let row_pixels = color.repeat((right - left) as usize);
        for row in top..bottom {
            let start = (row as usize * width as usize + left as usize) * 4;
            pixels[start..start + row_pixels.len()].copy_from_slice(&row_pixels);
        }
        return;
    }
    let color = Color::rgba(color[0], color[1], color[2], color[3]);
    for row in top..bottom {
        for column in left..right {
            blend_pixel(pixels, width, height, column, row, color, color.a());
        }
    }
}

pub(super) fn fill_rounded_rect_alpha_clipped(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    rect: ClipRect,
    radii: [[f32; 2]; 4],
    color: [u8; 4],
    clip: Option<ClipRect>,
) {
    let left = (rect.left.floor() as i32).clamp(0, width as i32);
    let top = (rect.top.floor() as i32).clamp(0, height as i32);
    let right = (rect.right.ceil() as i32).clamp(0, width as i32);
    let bottom = (rect.bottom.ceil() as i32).clamp(0, height as i32);
    let left = clip.map_or(left, |clip| left.max(clip.left.floor() as i32));
    let top = clip.map_or(top, |clip| top.max(clip.top.floor() as i32));
    let right = clip.map_or(right, |clip| right.min(clip.right.ceil() as i32));
    let bottom = clip.map_or(bottom, |clip| bottom.min(clip.bottom.ceil() as i32));
    if right <= left || bottom <= top || color[3] == 0 {
        return;
    }
    let source = Color::rgba(color[0], color[1], color[2], color[3]);

    // Anti-aliasing only affects the four corner bounding boxes. Everything
    // else is fully covered, so split the rectangle into a solid interior and
    // four sampled corners instead of sampling every pixel.
    let left_band = (radii[0][0].max(radii[3][0]).ceil() as i32).clamp(0, right - left);
    let right_band = (radii[1][0].max(radii[2][0]).ceil() as i32).clamp(0, right - left);
    let top_band = (radii[0][1].max(radii[1][1]).ceil() as i32).clamp(0, bottom - top);
    let bottom_band = (radii[2][1].max(radii[3][1]).ceil() as i32).clamp(0, bottom - top);

    if top + top_band > bottom - bottom_band || left + left_band > right - right_band {
        // Corner bands overlap; the shape has no fully covered interior.
        for y in top..bottom {
            for x in left..right {
                let coverage = rounded_coverage(x, y, rect, radii);
                if coverage > 0 {
                    let alpha = (u16::from(source.a()) * coverage as u16 / 16) as u8;
                    blend_pixel_rounded(pixels, width, height, x, y, source, alpha);
                }
            }
        }
        return;
    }

    let inner_top = top + top_band;
    let inner_bottom = bottom - bottom_band;
    let inner_left = left + left_band;
    let inner_right = right - right_band;
    fill_covered_rect(pixels, width, height, left, inner_top, right, inner_bottom, source);
    fill_covered_rect(pixels, width, height, inner_left, top, inner_right, inner_top, source);
    fill_covered_rect(
        pixels,
        width,
        height,
        inner_left,
        inner_bottom,
        inner_right,
        bottom,
        source,
    );

    let corners = [
        (left, top, left + left_band, top + top_band),
        (inner_right, top, right, top + top_band),
        (inner_right, inner_bottom, right, bottom),
        (left, inner_bottom, left + left_band, bottom),
    ];
    for (region_left, region_top, region_right, region_bottom) in corners {
        for y in region_top..region_bottom {
            for x in region_left..region_right {
                let coverage = rounded_coverage(x, y, rect, radii);
                if coverage > 0 {
                    let alpha = (u16::from(source.a()) * coverage as u16 / 16) as u8;
                    blend_pixel_rounded(pixels, width, height, x, y, source, alpha);
                }
            }
        }
    }
}

fn fill_covered_rect(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    source: Color,
) {
    if right <= left || bottom <= top {
        return;
    }
    if source.a() == 255 {
        let rgba = [source.r(), source.g(), source.b(), source.a()];
        let row_pixels = rgba.repeat((right - left) as usize);
        for row in top..bottom {
            let start = (row as usize * width as usize + left as usize) * 4;
            pixels[start..start + row_pixels.len()].copy_from_slice(&row_pixels);
        }
        return;
    }
    for row in top..bottom {
        for column in left..right {
            blend_pixel_rounded(pixels, width, height, column, row, source, source.a());
        }
    }
}

fn rounded_coverage(x: i32, y: i32, rect: ClipRect, radii: [[f32; 2]; 4]) -> u32 {
    const SAMPLES: [f32; 4] = [0.125, 0.375, 0.625, 0.875];
    let mut coverage = 0;
    for sample_y in SAMPLES {
        for sample_x in SAMPLES {
            if point_in_rounded_rect(x as f32 + sample_x, y as f32 + sample_y, rect, radii) {
                coverage += 1;
            }
        }
    }
    coverage
}

fn point_in_rounded_rect(x: f32, y: f32, rect: ClipRect, radii: [[f32; 2]; 4]) -> bool {
    let corners = [
        (
            rect.left + radii[0][0],
            rect.top + radii[0][1],
            radii[0],
            x < rect.left + radii[0][0] && y < rect.top + radii[0][1],
        ),
        (
            rect.right - radii[1][0],
            rect.top + radii[1][1],
            radii[1],
            x > rect.right - radii[1][0] && y < rect.top + radii[1][1],
        ),
        (
            rect.right - radii[2][0],
            rect.bottom - radii[2][1],
            radii[2],
            x > rect.right - radii[2][0] && y > rect.bottom - radii[2][1],
        ),
        (
            rect.left + radii[3][0],
            rect.bottom - radii[3][1],
            radii[3],
            x < rect.left + radii[3][0] && y > rect.bottom - radii[3][1],
        ),
    ];
    for (center_x, center_y, [radius_x, radius_y], in_corner) in corners {
        if in_corner && radius_x > 0.0 && radius_y > 0.0 {
            let dx = (x - center_x) / radius_x;
            let dy = (y - center_y) / radius_y;
            if dx * dx + dy * dy > 1.0 {
                return false;
            }
        }
    }
    true
}

fn rgba_channel(rgba: (u8, u8, u8, u8), channel: usize) -> u8 {
    match channel {
        0 => rgba.0,
        1 => rgba.1,
        _ => rgba.2,
    }
}

// Reuse the upload buffer, retaining only the current viewport's capacity.
pub(super) fn filled_pixels(mut pixels: Vec<u8>, color: [u8; 4], count: usize) -> Vec<u8> {
    if pixels.capacity() == 0 {
        return color.repeat(count);
    }
    let length = count * 4;
    pixels.resize(length, 0);
    if pixels.capacity() > length {
        pixels.shrink_to_fit();
    }
    if length != 0 {
        pixels[..4].copy_from_slice(&color);
        let mut initialized = 4;
        while initialized < length {
            let copied = initialized.min(length - initialized);
            pixels.copy_within(..copied, initialized);
            initialized += copied;
        }
    }
    pixels
}

#[cfg(test)]
mod tests {
    use super::{
        ClipRect, blend_pixel_rounded, fill_rounded_rect_alpha_clipped, filled_pixels,
        point_in_rounded_rect,
    };
    use cosmic_text::Color;

    fn sample_every_pixel_reference(
        pixels: &mut [u8],
        width: u32,
        height: u32,
        rect: ClipRect,
        radii: [[f32; 2]; 4],
        color: [u8; 4],
        clip: Option<ClipRect>,
    ) {
        let left = (rect.left.floor() as i32).clamp(0, width as i32);
        let top = (rect.top.floor() as i32).clamp(0, height as i32);
        let right = (rect.right.ceil() as i32).clamp(0, width as i32);
        let bottom = (rect.bottom.ceil() as i32).clamp(0, height as i32);
        let left = clip.map_or(left, |clip| left.max(clip.left.floor() as i32));
        let top = clip.map_or(top, |clip| top.max(clip.top.floor() as i32));
        let right = clip.map_or(right, |clip| right.min(clip.right.ceil() as i32));
        let bottom = clip.map_or(bottom, |clip| bottom.min(clip.bottom.ceil() as i32));
        let source = Color::rgba(color[0], color[1], color[2], color[3]);
        let samples = [0.125, 0.375, 0.625, 0.875];
        for y in top..bottom {
            for x in left..right {
                let coverage = samples
                    .iter()
                    .flat_map(|sample_y| {
                        samples.iter().filter(move |sample_x| {
                            point_in_rounded_rect(
                                x as f32 + **sample_x,
                                y as f32 + *sample_y,
                                rect,
                                radii,
                            )
                        })
                    })
                    .count();
                if coverage > 0 {
                    let alpha = (u16::from(source.a()) * coverage as u16 / 16) as u8;
                    blend_pixel_rounded(pixels, width, height, x, y, source, alpha);
                }
            }
        }
    }

    #[test]
    fn rounded_rect_fast_path_matches_full_sampling() {
        let (width, height) = (64, 56);
        let rects = [
            ClipRect {
                left: 4.0,
                top: 6.0,
                right: 52.0,
                bottom: 48.0,
            },
            ClipRect {
                left: -10.0,
                top: -8.0,
                right: 70.0,
                bottom: 60.0,
            },
            ClipRect {
                left: 10.25,
                top: 12.75,
                right: 40.5,
                bottom: 30.25,
            },
        ];
        let radii = [
            [[32.0, 32.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]],
            [[12.0, 4.0], [4.0, 12.0], [12.0, 4.0], [4.0, 12.0]],
            [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]],
            [[20.0, 30.0], [20.0, 30.0], [20.0, 30.0], [20.0, 30.0]],
        ];
        let colors = [
            [20, 30, 200, 255],
            [200, 60, 90, 128],
            [10, 200, 120, 1],
            [0, 0, 0, 0],
        ];
        let clips = [
            None,
            Some(ClipRect {
                left: 8.0,
                top: 8.0,
                right: 48.0,
                bottom: 40.0,
            }),
        ];
        for rect in rects {
            for radii in radii {
                for color in colors {
                    for clip in clips {
                        let mut expected = vec![0u8; width as usize * height as usize * 4];
                        let mut actual = expected.clone();
                        sample_every_pixel_reference(
                            &mut expected, width, height, rect, radii, color, clip,
                        );
                        fill_rounded_rect_alpha_clipped(
                            &mut actual, width, height, rect, radii, color, clip,
                        );
                        assert_eq!(
                            expected, actual,
                            "rect={rect:?} radii={radii:?} color={color:?} clip={clip:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn upload_buffer_reuses_storage_and_releases_capacity_after_shrink() {
        let pixels = filled_pixels(Vec::new(), [1, 2, 3, 255], 4096);
        let pointer = pixels.as_ptr();
        let pixels = filled_pixels(pixels, [4, 5, 6, 255], 4096);
        assert_eq!(pixels.as_ptr(), pointer);
        assert!(pixels.chunks_exact(4).all(|pixel| pixel == [4, 5, 6, 255]));
        let pixels = filled_pixels(pixels, [7, 8, 9, 255], 16);
        assert_eq!(pixels.len(), 64);
        assert_eq!(pixels.capacity(), 64);
        assert!(pixels.chunks_exact(4).all(|pixel| pixel == [7, 8, 9, 255]));
    }
}
