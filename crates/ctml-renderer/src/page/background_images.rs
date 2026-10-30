use skia_safe::{
    AlphaType, ClipOp, Color4f, ColorSpace, ColorType, Data, Image, ImageInfo, Matrix, Paint,
    Point, Rect, SamplingOptions, TileMode, gradient, surfaces,
};
use std::{collections::HashMap, sync::Arc};

use crate::style::{
    BackgroundAttachment, BackgroundBox, BackgroundFit, BackgroundImage, BackgroundPosition,
    BackgroundRepeat, BackgroundSize, BackgroundSizeAxis, LinearGradient,
};

use super::overflow_clip::{BackgroundBoxes, ClipRect};

pub(super) type ImageCache = HashMap<Arc<[u8]>, Option<Image>>;

pub(super) fn draw_background_image_layers(
    images: &[BackgroundImage],
    repeats: &[BackgroundRepeat],
    positions: &[BackgroundPosition],
    sizes: &[BackgroundSize],
    attachments: &[BackgroundAttachment],
    origins: &[BackgroundBox],
    clips: &[BackgroundBox],
    scroll_translation: (f32, f32),
    element_scroll: (f32, f32),
    opacity: f32,
    pixels: &mut [u8],
    width: u32,
    height: u32,
    boxes: BackgroundBoxes,
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
    for (index, background) in images.iter().enumerate().rev() {
        let attachment = attachments
            .get(index % attachments.len().max(1))
            .copied()
            .unwrap_or_default();
        let (attachment_x, attachment_y) = match attachment {
            BackgroundAttachment::Scroll => (0.0, 0.0),
            BackgroundAttachment::Fixed => scroll_translation,
            BackgroundAttachment::Local => (-element_scroll.0, -element_scroll.1),
        };
        let layer_boxes = boxes.offset(attachment_x, attachment_y);
        let origin_box = origins
            .get(index % origins.len().max(1))
            .copied()
            .unwrap_or_default();
        let clip_box = clips
            .get(index % clips.len().max(1))
            .copied()
            .unwrap_or_default();
        let origin_rect = layer_boxes.get(origin_box);
        let layer_rect = layer_boxes.get(clip_box);
        let origin_rect = Rect::from_ltrb(
            origin_rect.left,
            origin_rect.top,
            origin_rect.right,
            origin_rect.bottom,
        );
        let layer_rect = Rect::from_ltrb(
            layer_rect.left,
            layer_rect.top,
            layer_rect.right,
            layer_rect.bottom,
        );
        let repeat = repeats
            .get(index % repeats.len().max(1))
            .copied()
            .unwrap_or(BackgroundRepeat::REPEAT);
        let position = positions
            .get(index % positions.len().max(1))
            .copied()
            .unwrap_or_default();
        let size = sizes
            .get(index % sizes.len().max(1))
            .copied()
            .unwrap_or_default();
        let (shader, tile_clip) = match background {
            BackgroundImage::Encoded(encoded) => {
                let image = image_cache
                    .entry(Arc::clone(encoded))
                    .or_insert_with(|| Image::from_encoded(Data::new_copy(encoded.as_ref())));
                let Some(image) = image.as_ref() else {
                    continue;
                };
                let (image_width, image_height) = resolve_background_size(
                    size,
                    origin_rect.width(),
                    origin_rect.height(),
                    image.width() as f32,
                    image.height() as f32,
                );
                if image_width <= 0.0 || image_height <= 0.0 {
                    continue;
                }
                let scale_x = image_width / image.width() as f32;
                let scale_y = image_height / image.height() as f32;
                let image_left = origin_rect.left
                    + (origin_rect.width() - image_width) * position.x_percent
                    + position.x_offset;
                let image_top = origin_rect.top
                    + (origin_rect.height() - image_height) * position.y_percent
                    + position.y_offset;
                let clip = (!repeat.x || !repeat.y).then(|| {
                    Rect::from_ltrb(
                        if repeat.x {
                            layer_rect.left
                        } else {
                            image_left.max(layer_rect.left)
                        },
                        if repeat.y {
                            layer_rect.top
                        } else {
                            image_top.max(layer_rect.top)
                        },
                        if repeat.x {
                            layer_rect.right
                        } else {
                            (image_left + image_width).min(layer_rect.right)
                        },
                        if repeat.y {
                            layer_rect.bottom
                        } else {
                            (image_top + image_height).min(layer_rect.bottom)
                        },
                    )
                });
                let image_origin =
                    Matrix::scale_translate((scale_x, scale_y), (image_left, image_top));
                (
                    image.to_shader(
                        Some((TileMode::Repeat, TileMode::Repeat)),
                        SamplingOptions::default(),
                        Some(&image_origin),
                    ),
                    clip,
                )
            }
            BackgroundImage::LinearGradient(gradient) => {
                (linear_gradient_shader(gradient, origin_rect), None)
            }
        };
        let Some(shader) = shader else { continue };
        canvas.save();
        canvas.clip_rect(layer_rect, ClipOp::Intersect, false);
        if let Some(tile_clip) = tile_clip {
            canvas.clip_rect(tile_clip, ClipOp::Intersect, false);
        }
        let mut paint = Paint::default();
        paint.set_shader(shader);
        paint.set_alpha_f(opacity.clamp(0.0, 1.0));
        canvas.draw_rect(layer_rect, &paint);
        canvas.restore();
    }
    canvas.restore();
}

fn resolve_background_size(
    size: BackgroundSize,
    area_width: f32,
    area_height: f32,
    image_width: f32,
    image_height: f32,
) -> (f32, f32) {
    let ratio = image_width / image_height;
    let fit_scale = match size.fit {
        BackgroundFit::Auto => None,
        BackgroundFit::Cover => Some((area_width / image_width).max(area_height / image_height)),
        BackgroundFit::Contain => Some((area_width / image_width).min(area_height / image_height)),
    };
    if let Some(scale) = fit_scale {
        return (image_width * scale, image_height * scale);
    }
    let resolve = |axis: Option<BackgroundSizeAxis>, area: f32| {
        axis.map(|axis| {
            if axis.percentage {
                axis.value * area
            } else {
                axis.value
            }
        })
    };
    match (
        resolve(size.width, area_width),
        resolve(size.height, area_height),
    ) {
        (Some(width), Some(height)) => (width, height),
        (Some(width), None) => (width, width / ratio),
        (None, Some(height)) => (height * ratio, height),
        (None, None) => (image_width, image_height),
    }
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
    let raw_positions = gradient
        .stops
        .iter()
        .map(|(_, position)| *position)
        .collect::<Vec<_>>();
    let first_position = *raw_positions.first()?;
    let last_position = *raw_positions.last()?;
    let (positions, tile_mode) = if gradient.repeating {
        let period = last_position - first_position;
        if period <= 0.0 {
            return None;
        }
        (
            raw_positions
                .iter()
                .map(|position| (position - first_position) / period)
                .collect::<Vec<_>>(),
            TileMode::Repeat,
        )
    } else {
        (raw_positions, TileMode::Clamp)
    };
    let colors = gradient::Colors::new(&colors, Some(&positions), tile_mode, None);
    let shader_gradient = gradient::Gradient::new(colors, gradient::Interpolation::default());
    let angle = gradient.angle_degrees.to_radians();
    let direction = (angle.sin(), -angle.cos());
    let width = rect.width();
    let height = rect.height();
    let line_length = (width * direction.0.abs() + height * direction.1.abs()).max(1.0);
    let center = (rect.center_x(), rect.center_y());
    let half_length = line_length / 2.0;
    let base_start = Point::new(
        center.0 - direction.0 * half_length,
        center.1 - direction.1 * half_length,
    );
    let base_end = Point::new(
        center.0 + direction.0 * half_length,
        center.1 + direction.1 * half_length,
    );
    let (start, end) = if gradient.repeating {
        let period = last_position - first_position;
        let start = Point::new(
            base_start.x + direction.0 * line_length * first_position,
            base_start.y + direction.1 * line_length * first_position,
        );
        let end = Point::new(
            start.x + direction.0 * line_length * period,
            start.y + direction.1 * line_length * period,
        );
        (start, end)
    } else {
        (base_start, base_end)
    };
    gradient::shaders::linear_gradient((start, end), &shader_gradient, None)
}
