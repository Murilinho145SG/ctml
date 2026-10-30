use skia_safe::{
    AlphaType, ClipOp, Color4f, ColorSpace, ColorType, Data, Image, ImageInfo, Matrix, Paint,
    Point, RRect, Rect, SamplingOptions, TileMode, Vector, gradient, surfaces,
};
use std::{collections::HashMap, sync::Arc};

use crate::style::{
    BackgroundAttachment, BackgroundBox, BackgroundFit, BackgroundImage, BackgroundPosition,
    BackgroundRepeat, BackgroundSize, BackgroundSizeAxis, ConicGradient, LinearGradient,
    RadialGradient, RadialGradientExtent, RadialGradientShape,
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
    border_radii: Option<[[f32; 2]; 4]>,
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
            BackgroundImage::None => continue,
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
            BackgroundImage::RadialGradient(gradient) => {
                (radial_gradient_shader(gradient, origin_rect), None)
            }
            BackgroundImage::ConicGradient(gradient) => {
                (conic_gradient_shader(gradient, origin_rect), None)
            }
        };
        let Some(shader) = shader else { continue };
        canvas.save();
        canvas.clip_rect(layer_rect, ClipOp::Intersect, false);
        if let Some(radii) = border_radii {
            let rounded_rect = RRect::new_rect_radii(
                layer_rect,
                &[
                    Vector::new(radii[0][0], radii[0][1]),
                    Vector::new(radii[1][0], radii[1][1]),
                    Vector::new(radii[2][0], radii[2][1]),
                    Vector::new(radii[3][0], radii[3][1]),
                ],
            );
            canvas.clip_rrect(rounded_rect, ClipOp::Intersect, true);
        }
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

fn radial_gradient_shader(gradient: &RadialGradient, rect: Rect) -> Option<skia_safe::Shader> {
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
    let mut positions = gradient
        .stops
        .iter()
        .map(|(_, position)| *position)
        .collect::<Vec<_>>();
    let (tile_mode, repeat_scale) = if gradient.repeating {
        let first = *positions.first()?;
        let last = *positions.last()?;
        let span = last - first;
        if first > f32::EPSILON || span <= 0.0 {
            return None;
        }
        for position in &mut positions {
            *position /= span;
        }
        (TileMode::Repeat, span)
    } else {
        (TileMode::Clamp, 1.0)
    };
    let colors = gradient::Colors::new(&colors, Some(&positions), tile_mode, None);
    let shader_gradient = gradient::Gradient::new(colors, gradient::Interpolation::default());
    let center_x = rect.left + rect.width() * gradient.center_x_percent;
    let center_y = rect.top + rect.height() * gradient.center_y_percent;
    let distances = [
        (center_x - rect.left, center_y - rect.top),
        (rect.right - center_x, center_y - rect.top),
        (center_x - rect.left, rect.bottom - center_y),
        (rect.right - center_x, rect.bottom - center_y),
    ];
    let horizontal = (center_x - rect.left, rect.right - center_x);
    let vertical = (center_y - rect.top, rect.bottom - center_y);
    let (radius_x, radius_y) = match (gradient.shape, gradient.extent) {
        (RadialGradientShape::Circle, RadialGradientExtent::ClosestSide) => {
            let radius = horizontal
                .0
                .min(horizontal.1)
                .min(vertical.0)
                .min(vertical.1);
            (radius, radius)
        }
        (RadialGradientShape::Circle, RadialGradientExtent::FarthestSide) => {
            let radius = horizontal
                .0
                .max(horizontal.1)
                .max(vertical.0)
                .max(vertical.1);
            (radius, radius)
        }
        (RadialGradientShape::Circle, RadialGradientExtent::ClosestCorner) => {
            let radius = distances
                .iter()
                .map(|(x, y)| x.hypot(*y))
                .fold(f32::INFINITY, f32::min);
            (radius, radius)
        }
        (RadialGradientShape::Circle, RadialGradientExtent::FarthestCorner) => {
            let radius = distances
                .iter()
                .map(|(x, y)| x.hypot(*y))
                .fold(0.0, f32::max);
            (radius, radius)
        }
        (RadialGradientShape::Ellipse, RadialGradientExtent::ClosestSide) => {
            (horizontal.0.min(horizontal.1), vertical.0.min(vertical.1))
        }
        (RadialGradientShape::Ellipse, RadialGradientExtent::FarthestSide) => {
            (horizontal.0.max(horizontal.1), vertical.0.max(vertical.1))
        }
        (RadialGradientShape::Ellipse, extent) => {
            let distance = |(x, y): &(f32, f32)| x.hypot(*y);
            let corner = if extent == RadialGradientExtent::ClosestCorner {
                distances
                    .iter()
                    .min_by(|a, b| distance(a).total_cmp(&distance(b)))?
            } else {
                distances
                    .iter()
                    .max_by(|a, b| distance(a).total_cmp(&distance(b)))?
            };
            *corner
        }
    };
    let radius_x = radius_x * repeat_scale;
    let radius_y = radius_y * repeat_scale;
    if radius_x <= 0.0 || radius_y <= 0.0 {
        return None;
    }
    if gradient.shape == RadialGradientShape::Circle {
        gradient::shaders::radial_gradient(
            (Point::new(center_x, center_y), radius_x),
            &shader_gradient,
            None,
        )
    } else {
        let matrix = Matrix::scale_translate((radius_x, radius_y), (center_x, center_y));
        gradient::shaders::radial_gradient(
            (Point::new(0.0, 0.0), 1.0),
            &shader_gradient,
            Some(&matrix),
        )
    }
}

fn conic_gradient_shader(gradient: &ConicGradient, rect: Rect) -> Option<skia_safe::Shader> {
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
    let mut positions = gradient
        .stops
        .iter()
        .map(|(_, position)| *position)
        .collect::<Vec<_>>();
    let (start, end, tile_mode) = if gradient.repeating {
        let first = *positions.first()?;
        let last = *positions.last()?;
        let span = last - first;
        if span <= 0.0 {
            return None;
        }
        for position in &mut positions {
            *position = (*position - first) / span;
        }
        let start = gradient.from_degrees + first * 360.0 - 90.0;
        (start, start + span * 360.0, TileMode::Repeat)
    } else {
        let start = gradient.from_degrees - 90.0;
        (start, start + 360.0, TileMode::Clamp)
    };
    let colors = gradient::Colors::new(&colors, Some(&positions), tile_mode, None);
    let shader_gradient = gradient::Gradient::new(colors, gradient::Interpolation::default());
    let center = Point::new(
        rect.left + rect.width() * gradient.center_x_percent,
        rect.top + rect.height() * gradient.center_y_percent,
    );
    gradient::shaders::sweep_gradient(center, (start, end), &shader_gradient, None)
}
