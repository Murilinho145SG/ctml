use crate::style::{BorderImageOutset, BorderImageRepeat, LayoutSize};
use skia_safe::{
    AlphaType, ColorSpace, ColorType, Data, Image, ImageInfo, Paint, Rect,
    canvas::SrcRectConstraint, surfaces,
};

use super::overflow_clip;

#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    size: &LayoutSize,
    rect: overflow_clip::ClipRect,
    border_widths: [f32; 4],
    opacity: f32,
    clip: Option<overflow_clip::ClipRect>,
) -> bool {
    let Some(encoded) = size.border_image_source.as_ref() else {
        return false;
    };
    let Some(image) = Image::from_encoded(Data::new_copy(encoded.as_ref())) else {
        return false;
    };
    let image_width = image.width() as f32;
    let image_height = image.height() as f32;
    if image_width <= 0.0 || image_height <= 0.0 {
        return false;
    }
    let mut slices = size.border_image_slice.map(|slice| {
        if slice.percentage {
            slice.value
        } else {
            slice.value
        }
    });
    slices[0] *= if size.border_image_slice[0].percentage {
        image_height
    } else {
        1.0
    };
    slices[2] *= if size.border_image_slice[2].percentage {
        image_height
    } else {
        1.0
    };
    slices[1] *= if size.border_image_slice[1].percentage {
        image_width
    } else {
        1.0
    };
    slices[3] *= if size.border_image_slice[3].percentage {
        image_width
    } else {
        1.0
    };
    (slices[3], slices[1]) = scale_opposing_slices(slices[3], slices[1], image_width);
    (slices[0], slices[2]) = scale_opposing_slices(slices[0], slices[2], image_height);

    let outsets: [f32; 4] = std::array::from_fn(|side| match size.border_image_outset[side] {
        Some(BorderImageOutset::Number(multiplier)) => border_widths[side] * multiplier,
        Some(BorderImageOutset::Length(length)) => length,
        None => 0.0,
    });
    let left = rect.left - outsets[3];
    let top = rect.top - outsets[0];
    let right = rect.right + outsets[1];
    let bottom = rect.bottom + outsets[2];
    let image_widths: [f32; 4] = std::array::from_fn(|side| {
        let border_width = border_widths[side];
        match size.border_image_width[side] {
            Some(crate::style::BorderImageWidth::Number(multiplier)) => border_width * multiplier,
            Some(crate::style::BorderImageWidth::Length(length)) => length,
            Some(crate::style::BorderImageWidth::Percentage(percentage)) => {
                let axis_size = if side % 2 == 0 {
                    bottom - top
                } else {
                    right - left
                };
                axis_size * percentage
            }
            Some(crate::style::BorderImageWidth::Auto) => slices[side],
            None => border_width,
        }
    });
    let mut dest = [
        image_widths[3],
        image_widths[1],
        image_widths[0],
        image_widths[2],
    ];
    (dest[0], dest[1]) = scale_opposing_slices(dest[0], dest[1], (right - left).max(0.0));
    (dest[2], dest[3]) = scale_opposing_slices(dest[2], dest[3], (bottom - top).max(0.0));
    let source_x = [0.0, slices[3], image_width - slices[1], image_width];
    let source_y = [0.0, slices[0], image_height - slices[2], image_height];
    let dest_x = [left, left + dest[0], right - dest[1], right];
    let dest_y = [top, top + dest[2], bottom - dest[3], bottom];

    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        ColorSpace::new_srgb(),
    );
    let Some(mut surface) = surfaces::wrap_pixels(&info, pixels, Some(width as usize * 4), None)
    else {
        return false;
    };
    let canvas = surface.canvas();
    canvas.save();
    if let Some(clip) = clip {
        canvas.clip_rect(
            Rect::from_ltrb(clip.left, clip.top, clip.right, clip.bottom),
            skia_safe::ClipOp::Intersect,
            false,
        );
    }
    let mut paint = Paint::default();
    paint.set_alpha_f(opacity.clamp(0.0, 1.0));
    for row in 0..3 {
        for column in 0..3 {
            if row == 1 && column == 1 && !size.border_image_fill {
                continue;
            }
            let source = Rect::from_ltrb(
                source_x[column],
                source_y[row],
                source_x[column + 1],
                source_y[row + 1],
            );
            let destination = Rect::from_ltrb(
                dest_x[column],
                dest_y[row],
                dest_x[column + 1],
                dest_y[row + 1],
            );
            if source.width() <= 0.0
                || source.height() <= 0.0
                || destination.width() <= 0.0
                || destination.height() <= 0.0
            {
                continue;
            }
            let repeat_x = match (row, column) {
                (1, 1) | (0 | 2, 1) => size.border_image_repeat[0],
                _ => BorderImageRepeat::Stretch,
            };
            let repeat_y = match (row, column) {
                (1, 1) | (1, 0 | 2) => size.border_image_repeat[1],
                _ => BorderImageRepeat::Stretch,
            };
            let center_natural_size = (row == 1 && column == 1).then(|| {
                let horizontal_scale = if slices[3] > 0.0 {
                    dest[0] / slices[3]
                } else {
                    1.0
                };
                let vertical_scale = if slices[0] > 0.0 {
                    dest[2] / slices[0]
                } else {
                    1.0
                };
                (
                    source.width() * horizontal_scale,
                    source.height() * vertical_scale,
                )
            });
            draw_region(
                canvas,
                &image,
                source,
                destination,
                &paint,
                repeat_x,
                repeat_y,
                center_natural_size,
            );
        }
    }
    canvas.restore();
    true
}

fn draw_region(
    canvas: &skia_safe::Canvas,
    image: &Image,
    source: Rect,
    destination: Rect,
    paint: &Paint,
    repeat_x: BorderImageRepeat,
    repeat_y: BorderImageRepeat,
    center_natural_size: Option<(f32, f32)>,
) {
    let (natural_width, natural_height) = center_natural_size.unwrap_or_else(|| {
        (
            source.width() * destination.height() / source.height(),
            source.height() * destination.width() / source.width(),
        )
    });
    let horizontal = axis_tiles(
        destination.left,
        destination.right,
        source.left,
        source.right,
        repeat_x,
        natural_width,
    );
    let vertical = axis_tiles(
        destination.top,
        destination.bottom,
        source.top,
        source.bottom,
        repeat_y,
        natural_height,
    );
    for (left, right, source_left, source_right) in horizontal {
        for (top, bottom, source_top, source_bottom) in &vertical {
            let source = Rect::from_ltrb(source_left, *source_top, source_right, *source_bottom);
            let destination = Rect::from_ltrb(left, *top, right, *bottom);
            canvas.draw_image_rect(
                image,
                Some((&source, SrcRectConstraint::Strict)),
                &destination,
                paint,
            );
        }
    }
}

fn axis_tiles(
    start: f32,
    end: f32,
    source_start: f32,
    source_end: f32,
    repeat: BorderImageRepeat,
    natural_size: f32,
) -> Vec<(f32, f32, f32, f32)> {
    let length = end - start;
    let source_length = source_end - source_start;
    if length <= 0.0 || source_length <= 0.0 {
        return Vec::new();
    }
    if repeat == BorderImageRepeat::Stretch || natural_size <= 0.0 {
        return vec![(start, end, source_start, source_end)];
    }
    let count = match repeat {
        BorderImageRepeat::Stretch => 1,
        BorderImageRepeat::Repeat => (length / natural_size).ceil().clamp(1.0, 8192.0) as usize,
        BorderImageRepeat::Round => (length / natural_size).round().clamp(1.0, 8192.0) as usize,
        BorderImageRepeat::Space => (length / natural_size).floor().clamp(0.0, 8192.0) as usize,
    };
    if count <= 1 && repeat == BorderImageRepeat::Space {
        return vec![(start, end, source_start, source_end)];
    }
    let tile_size = match repeat {
        BorderImageRepeat::Round => length / count as f32,
        BorderImageRepeat::Space | BorderImageRepeat::Repeat => natural_size,
        BorderImageRepeat::Stretch => length,
    };
    let gap = if repeat == BorderImageRepeat::Space && count > 1 {
        (length - count as f32 * tile_size) / (count - 1) as f32
    } else {
        0.0
    };
    (0..count)
        .filter_map(|index| {
            let offset = index as f32 * (tile_size + gap);
            if offset >= length {
                return None;
            }
            let visible_size = tile_size.min(length - offset);
            let source_visible = source_length * (visible_size / tile_size);
            Some((
                start + offset,
                start + offset + visible_size,
                source_start,
                source_start + source_visible,
            ))
        })
        .collect()
}

fn scale_opposing_slices(mut first: f32, mut second: f32, total: f32) -> (f32, f32) {
    let sum = first + second;
    if sum > total && sum > 0.0 {
        let scale = total / sum;
        first *= scale;
        second *= scale;
    }
    (first, second)
}
