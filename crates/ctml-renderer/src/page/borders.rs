use std::collections::{HashMap, HashSet};

use crate::style::{BorderLineStyle, LayoutSize, TextRun, WritingMode};
use skia_safe::{
    AlphaType, ColorSpace, ColorType, Data, Image, ImageInfo, Paint, Rect,
    canvas::SrcRectConstraint, surfaces,
};

use super::{
    PAGE_PADDING,
    border_radius::{draw_rounded_border, resolve_radii},
    overflow_clip::{self, BlockGeometry},
    pixel::fill_rect_alpha_clipped,
};

pub(super) fn draw(
    runs: &[TextRun],
    pixels: &mut [u8],
    width: u32,
    height: u32,
    writing_mode: WritingMode,
    geometries: &HashMap<usize, BlockGeometry>,
    clips: &HashMap<usize, overflow_clip::ClipRect>,
) {
    let mut drawn = HashSet::new();
    for run in runs {
        if run.style.visibility_hidden || run.style.content_skipped_by_ancestor {
            continue;
        }
        for size in &run.style.layout_sizes {
            if !drawn.insert(size.node_id) {
                continue;
            }
            let Some(geometry) = geometries.get(&size.node_id) else {
                continue;
            };
            draw_block_border(
                pixels,
                width,
                height,
                size,
                geometry,
                &run.style.color,
                run.style.opacity,
                writing_mode,
                &run.element_path,
                clips,
            );
            super::outlines::draw_outline(
                pixels,
                width,
                height,
                size,
                geometry,
                &run.style.color,
                run.style.opacity,
                &run.element_path,
                clips,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_block_border(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    size: &LayoutSize,
    geometry: &BlockGeometry,
    current_color: &[u8; 4],
    opacity: f32,
    writing_mode: WritingMode,
    element_path: &[usize],
    clips: &HashMap<usize, overflow_clip::ClipRect>,
) {
    let widths = super::sizing_padding::resolve_borders(size, 0.0);
    let rect = geometry.border_box;
    if rect.right <= rect.left || rect.bottom <= rect.top {
        return;
    }
    let parent_path = element_path
        .iter()
        .copied()
        .filter(|node_id| *node_id != size.node_id)
        .collect::<Vec<_>>();
    let clip = overflow_clip::clip_for_path(&parent_path, clips);
    let transformed_sides = if writing_mode == WritingMode::HorizontalTb
        && size.writing_mode != WritingMode::HorizontalTb
    {
        [widths[1], widths[2], widths[3], widths[0]]
    } else {
        widths
    };
    let colors = size
        .border_color
        .map(|color| color.unwrap_or(*current_color));
    let styles = size.border_style;
    let radii = resolve_radii(size, rect);
    if draw_border_image(
        pixels,
        width,
        height,
        size,
        rect,
        transformed_sides,
        opacity,
        clip,
    ) {
        return;
    }
    if radii.iter().flatten().any(|radius| *radius > 0.0) {
        draw_rounded_border(
            pixels,
            width,
            height,
            rect,
            transformed_sides,
            radii,
            styles,
            colors,
            opacity,
            clip,
        );
        return;
    }
    draw_edge(
        pixels,
        width,
        height,
        rect.left,
        rect.top,
        rect.right - rect.left,
        transformed_sides[0],
        true,
        0,
        styles[0],
        edge_color(colors[0], opacity),
        clip,
    );
    draw_edge(
        pixels,
        width,
        height,
        rect.right - transformed_sides[1],
        rect.top,
        rect.bottom - rect.top,
        transformed_sides[1],
        false,
        1,
        styles[1],
        edge_color(colors[1], opacity),
        clip,
    );
    draw_edge(
        pixels,
        width,
        height,
        rect.left,
        rect.bottom - transformed_sides[2],
        rect.right - rect.left,
        transformed_sides[2],
        true,
        2,
        styles[2],
        edge_color(colors[2], opacity),
        clip,
    );
    draw_edge(
        pixels,
        width,
        height,
        rect.left,
        rect.top,
        rect.bottom - rect.top,
        transformed_sides[3],
        false,
        3,
        styles[3],
        edge_color(colors[3], opacity),
        clip,
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_border_image(
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

    let left = rect.left;
    let top = rect.top;
    let right = rect.right;
    let bottom = rect.bottom;
    let mut dest = [
        border_widths[3],
        border_widths[1],
        border_widths[0],
        border_widths[2],
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
            canvas.draw_image_rect(
                &image,
                Some((&source, SrcRectConstraint::Strict)),
                &destination,
                &paint,
            );
        }
    }
    canvas.restore();
    true
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

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_edge(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: f32,
    y: f32,
    length: f32,
    thickness: f32,
    horizontal: bool,
    side: usize,
    style: BorderLineStyle,
    color: [u8; 4],
    clip: Option<overflow_clip::ClipRect>,
) {
    if thickness <= 0.0
        || color[3] == 0
        || matches!(style, BorderLineStyle::None | BorderLineStyle::Hidden)
    {
        return;
    }
    let draw_rect = |pixels: &mut [u8], x: f32, y: f32, w: f32, h: f32, color: [u8; 4]| {
        fill_rect_alpha_clipped(
            pixels,
            width,
            height,
            x - PAGE_PADDING,
            y - PAGE_PADDING,
            w,
            h,
            color,
            clip,
        );
    };
    match style {
        BorderLineStyle::Dashed | BorderLineStyle::Dotted => {
            let unit = if style == BorderLineStyle::Dashed {
                (thickness * 3.0).max(4.0)
            } else {
                thickness.max(1.0)
            };
            let step = if style == BorderLineStyle::Dashed {
                unit * 2.0
            } else {
                unit * 2.0
            };
            let mut offset = 0.0;
            while offset < length {
                let segment = unit.min(length - offset);
                if horizontal {
                    draw_rect(pixels, x + offset, y, segment, thickness, color);
                } else {
                    draw_rect(pixels, x, y + offset, thickness, segment, color);
                }
                offset += step;
            }
        }
        BorderLineStyle::Double if thickness >= 3.0 => {
            let stripe = (thickness / 3.0).floor().max(1.0);
            if horizontal {
                draw_rect(pixels, x, y, length, stripe, color);
                draw_rect(pixels, x, y + thickness - stripe, length, stripe, color);
            } else {
                draw_rect(pixels, x, y, stripe, length, color);
                draw_rect(pixels, x + thickness - stripe, y, stripe, length, color);
            }
        }
        BorderLineStyle::Inset | BorderLineStyle::Outset => {
            let darker = if style == BorderLineStyle::Inset {
                matches!(side, 0 | 3)
            } else {
                matches!(side, 1 | 2)
            };
            draw_rect(
                pixels,
                x,
                y,
                if horizontal { length } else { thickness },
                if horizontal { thickness } else { length },
                shade(color, darker),
            );
        }
        BorderLineStyle::Groove | BorderLineStyle::Ridge => {
            let left_or_top = matches!(side, 0 | 3);
            let outer_dark = if style == BorderLineStyle::Groove {
                left_or_top
            } else {
                !left_or_top
            };
            let half = thickness / 2.0;
            if horizontal {
                draw_rect(pixels, x, y, length, half, shade(color, outer_dark));
                draw_rect(
                    pixels,
                    x,
                    y + half,
                    length,
                    thickness - half,
                    shade(color, !outer_dark),
                );
            } else {
                draw_rect(pixels, x, y, half, length, shade(color, outer_dark));
                draw_rect(
                    pixels,
                    x + half,
                    y,
                    thickness - half,
                    length,
                    shade(color, !outer_dark),
                );
            }
        }
        _ => {
            if horizontal {
                draw_rect(pixels, x, y, length, thickness, color);
            } else {
                draw_rect(pixels, x, y, thickness, length, color);
            }
        }
    }
}

pub(super) fn shade(mut color: [u8; 4], darker: bool) -> [u8; 4] {
    for channel in &mut color[..3] {
        *channel = if darker {
            (*channel as f32 * 0.55).round() as u8
        } else {
            (*channel as f32 + (255.0 - *channel as f32) * 0.45).round() as u8
        };
    }
    color
}

pub(super) fn edge_color(mut color: [u8; 4], opacity: f32) -> [u8; 4] {
    color[3] = (f32::from(color[3]) * opacity).round().clamp(0.0, 255.0) as u8;
    color
}
