use skia_safe::{
    AlphaType, ClipOp, ColorSpace, ColorType, Data, Image, ImageInfo, Matrix, Paint, Rect,
    SamplingOptions, TileMode, surfaces,
};
use std::{collections::HashMap, sync::Arc};

use super::overflow_clip::ClipRect;

pub(super) type ImageCache = HashMap<Arc<[u8]>, Option<Image>>;

pub(super) fn draw_repeating_images(
    encoded_images: &[Arc<[u8]>],
    opacity: f32,
    pixels: &mut [u8],
    width: u32,
    height: u32,
    rect: ClipRect,
    clip: Option<ClipRect>,
    image_cache: &mut ImageCache,
) {
    if encoded_images.is_empty() || opacity <= 0.0 || width == 0 || height == 0 {
        return;
    }
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        ColorSpace::new_srgb(),
    );
    let Some(mut surface) = surfaces::wrap_pixels(&info, pixels, Some(width as usize * 4), None)
    else {
        return;
    };
    let canvas = surface.canvas();
    canvas.save();
    if let Some(clip) = clip {
        canvas.clip_rect(
            Rect::from_ltrb(clip.left, clip.top, clip.right, clip.bottom),
            ClipOp::Intersect,
            false,
        );
    }
    let tile_origin = Matrix::translate((rect.left, rect.top));
    let rect = Rect::from_ltrb(rect.left, rect.top, rect.right, rect.bottom);
    for encoded in encoded_images.iter().rev() {
        let image = image_cache
            .entry(Arc::clone(encoded))
            .or_insert_with(|| Image::from_encoded(Data::new_copy(encoded.as_ref())));
        let Some(image) = image.as_ref() else {
            continue;
        };
        let Some(shader) = image.to_shader(
            Some((TileMode::Repeat, TileMode::Repeat)),
            SamplingOptions::default(),
            Some(&tile_origin),
        ) else {
            continue;
        };
        let mut paint = Paint::default();
        paint.set_shader(shader);
        paint.set_alpha_f(opacity.clamp(0.0, 1.0));
        canvas.draw_rect(rect, &paint);
    }
    canvas.restore();
}
