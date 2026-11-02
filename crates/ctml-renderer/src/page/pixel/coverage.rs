use cosmic_text::Color;

use super::super::{overflow_clip::ClipRect, page_padding};
use super::blend_pixel_rounded;

pub(in crate::page) fn fill_rect_alpha_covered_clipped(
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
    let mut left = x + page_padding();
    let mut top = y + page_padding();
    let mut right = left + rect_width;
    let mut bottom = top + rect_height;
    if let Some(clip) = clip {
        left = left.max(clip.left);
        top = top.max(clip.top);
        right = right.min(clip.right);
        bottom = bottom.min(clip.bottom);
    }
    left = left.max(0.0);
    top = top.max(0.0);
    right = right.min(width as f32);
    bottom = bottom.min(height as f32);
    if right <= left || bottom <= top || color[3] == 0 {
        return;
    }

    let source = Color::rgba(color[0], color[1], color[2], color[3]);
    for pixel_y in top.floor() as i32..bottom.ceil() as i32 {
        let coverage_y = (bottom.min(pixel_y as f32 + 1.0) - top.max(pixel_y as f32)).max(0.0);
        for pixel_x in left.floor() as i32..right.ceil() as i32 {
            let coverage_x = (right.min(pixel_x as f32 + 1.0) - left.max(pixel_x as f32)).max(0.0);
            let alpha = (f32::from(color[3]) * coverage_x * coverage_y)
                .round()
                .clamp(0.0, 255.0) as u8;
            blend_pixel_rounded(pixels, width, height, pixel_x, pixel_y, source, alpha);
        }
    }
}
