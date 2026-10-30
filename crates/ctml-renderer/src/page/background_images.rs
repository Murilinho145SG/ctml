use skia_safe::{
    AlphaType, ClipOp, Color4f, ColorSpace, ColorType, Data, Image, ImageInfo, Matrix, Paint,
    Point, Rect, SamplingOptions, TileMode, gradient, surfaces,
};
use std::{collections::HashMap, sync::Arc};

use crate::style::{BackgroundImage, BackgroundRepeat, LinearGradient};

use super::overflow_clip::ClipRect;

pub(super) type ImageCache = HashMap<Arc<[u8]>, Option<Image>>;

pub(super) fn draw_background_image_layers(
    images: &[BackgroundImage],
    repeats: &[BackgroundRepeat],
    opacity: f32,
    pixels: &mut [u8],
    width: u32,
    height: u32,
    rect: ClipRect,
    clip: Option<ClipRect>,
    image_cache: &mut ImageCache,
) {
    if images.is_empty() || opacity <= 0.0 || width == 0 || height == 0 {
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
    for (index, background) in images.iter().enumerate().rev() {
        let repeat = repeats
            .get(index % repeats.len().max(1))
            .copied()
            .unwrap_or(BackgroundRepeat::REPEAT);
        let (shader, tile_clip) = match background {
            BackgroundImage::Encoded(encoded) => {
                let image = image_cache
                    .entry(Arc::clone(encoded))
                    .or_insert_with(|| Image::from_encoded(Data::new_copy(encoded.as_ref())));
                let Some(image) = image.as_ref() else {
                    continue;
                };
                let clip = (!repeat.x || !repeat.y).then(|| {
                    Rect::from_ltrb(
                        rect.left,
                        rect.top,
                        if repeat.x {
                            rect.right
                        } else {
                            (rect.left + image.width() as f32).min(rect.right)
                        },
                        if repeat.y {
                            rect.bottom
                        } else {
                            (rect.top + image.height() as f32).min(rect.bottom)
                        },
                    )
                });
                (
                    image.to_shader(
                        Some((TileMode::Repeat, TileMode::Repeat)),
                        SamplingOptions::default(),
                        Some(&tile_origin),
                    ),
                    clip,
                )
            }
            BackgroundImage::LinearGradient(gradient) => {
                (linear_gradient_shader(gradient, rect), None)
            }
        };
        let Some(shader) = shader else { continue };
        canvas.save();
        if let Some(tile_clip) = tile_clip {
            canvas.clip_rect(tile_clip, ClipOp::Intersect, false);
        }
        let mut paint = Paint::default();
        paint.set_shader(shader);
        paint.set_alpha_f(opacity.clamp(0.0, 1.0));
        canvas.draw_rect(rect, &paint);
        canvas.restore();
    }
    canvas.restore();
}

fn linear_gradient_shader(gradient: &LinearGradient, rect: Rect) -> Option<skia_safe::Shader> {
    let colors = gradient
        .stops
        .iter()
        .map(|(color, _)| {
            Color4f::new(
                f32::from(color[0]) / 255.0,
                f32::from(color[1]) / 255.0,
                f32::from(color[2]) / 255.0,
                f32::from(color[3]) / 255.0,
            )
        })
        .collect::<Vec<_>>();
    let positions = gradient
        .stops
        .iter()
        .map(|(_, position)| *position)
        .collect::<Vec<_>>();
    let colors = gradient::Colors::new(&colors, Some(&positions), TileMode::Clamp, None);
    let shader_gradient = gradient::Gradient::new(colors, gradient::Interpolation::default());
    let angle = gradient.angle_degrees.to_radians();
    let direction = (angle.sin(), -angle.cos());
    let width = rect.width();
    let height = rect.height();
    let line_length = (width * direction.0.abs() + height * direction.1.abs()).max(1.0);
    let center = (rect.center_x(), rect.center_y());
    let half_length = line_length / 2.0;
    let start = Point::new(
        center.0 - direction.0 * half_length,
        center.1 - direction.1 * half_length,
    );
    let end = Point::new(
        center.0 + direction.0 * half_length,
        center.1 + direction.1 * half_length,
    );
    gradient::shaders::linear_gradient((start, end), &shader_gradient, None)
}
