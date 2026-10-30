use std::collections::{HashMap, HashSet};

use crate::style::{BoxShadow, TextRun};

use super::{
    PAGE_PADDING, overflow_clip, overflow_clip::BlockGeometry, pixel::fill_rect_alpha_clipped,
};

pub(super) fn draw_outer(
    runs: &[TextRun],
    pixels: &mut [u8],
    width: u32,
    height: u32,
    geometries: &HashMap<usize, BlockGeometry>,
    clips: &HashMap<usize, overflow_clip::ClipRect>,
) {
    draw(runs, pixels, width, height, geometries, clips, false);
}

pub(super) fn draw_inset(
    runs: &[TextRun],
    pixels: &mut [u8],
    width: u32,
    height: u32,
    geometries: &HashMap<usize, BlockGeometry>,
    clips: &HashMap<usize, overflow_clip::ClipRect>,
) {
    draw(runs, pixels, width, height, geometries, clips, true);
}

pub(super) fn draw_first_letter(
    runs: &[TextRun],
    buffer: &cosmic_text::Buffer,
    spans: &[super::CursorSpan],
    pixels: &mut [u8],
    width: u32,
    height: u32,
    page_writing_mode: crate::style::WritingMode,
    clips: &HashMap<usize, overflow_clip::ClipRect>,
    nested_scroll: &HashMap<usize, (f32, f32)>,
    inset: bool,
) {
    let first_letters = super::highlights::first_formatted_letters(spans);
    let vertical_origins =
        super::local_writing::vertical_block_origins(runs, buffer, page_writing_mode);
    let mut drawn = HashSet::new();
    for span in spans {
        let Some(block) = span.first_letter_block else {
            continue;
        };
        if !drawn.insert(block) || span.visibility_hidden {
            continue;
        }
        let Some((line_index, start, end)) = first_letters.get(&block).copied() else {
            continue;
        };
        let Some(style) = span.first_letter_font_style.as_ref() else {
            continue;
        };
        let Some(line) = buffer.layout_runs().find(|line| line.line_i == line_index) else {
            continue;
        };
        let mut left = f32::INFINITY;
        let mut right = f32::NEG_INFINITY;
        for glyph in line.glyphs {
            if glyph.start < end && glyph.end > start {
                left = left.min(glyph.x);
                right = right.max(glyph.x + glyph.w);
            }
        }
        if !left.is_finite() || !right.is_finite() || right <= left {
            continue;
        }
        let mut rect = overflow_clip::ClipRect {
            left: left + PAGE_PADDING,
            top: line.line_top + PAGE_PADDING,
            right: right + PAGE_PADDING,
            bottom: line.line_top + line.line_height + PAGE_PADDING,
        };
        if let Some(transform) = super::local_writing::LocalVerticalTransform::from_run(
            runs,
            buffer,
            line_index,
            page_writing_mode,
            width,
            height,
            &vertical_origins,
        ) {
            let (left, top, rect_width, rect_height) = transform.map_rect(
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
            );
            rect = overflow_clip::ClipRect {
                left,
                top,
                right: left + rect_width,
                bottom: top + rect_height,
            };
        }
        let (scroll_x, scroll_y) = overflow_clip::scroll_translation_for_path(
            &span.element_path,
            nested_scroll,
            page_writing_mode,
        );
        rect.left -= scroll_x;
        rect.right -= scroll_x;
        rect.top -= scroll_y;
        rect.bottom -= scroll_y;
        let clip = overflow_clip::clip_for_path(&span.element_path, clips);
        let opacity = span.opacity * span.first_letter_opacity.unwrap_or(1.0);
        for shadow in style
            .box_shadows
            .iter()
            .filter(|shadow| shadow.inset == inset)
            .rev()
        {
            let mut color = shadow.color.unwrap_or(style.color);
            color[3] = (f32::from(color[3]) * opacity).round().clamp(0.0, 255.0) as u8;
            paint_shadow(
                pixels,
                width,
                height,
                rect,
                [[0.0; 2]; 4],
                *shadow,
                color,
                clip,
            );
        }
    }
}

fn draw(
    runs: &[TextRun],
    pixels: &mut [u8],
    width: u32,
    height: u32,
    geometries: &HashMap<usize, BlockGeometry>,
    clips: &HashMap<usize, overflow_clip::ClipRect>,
    inset: bool,
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
            let rect = geometry.border_box;
            let radii = super::border_radius::resolve_radii(size, rect);
            let path = run
                .element_path
                .iter()
                .copied()
                .filter(|node_id| *node_id != size.node_id)
                .collect::<Vec<_>>();
            let clip = overflow_clip::clip_for_path(&path, clips);
            for shadow in size
                .box_shadows
                .iter()
                .filter(|shadow| shadow.inset == inset)
                .rev()
            {
                let mut color = shadow.color.unwrap_or(run.style.color);
                color[3] = (f32::from(color[3]) * run.style.opacity)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                paint_shadow(pixels, width, height, rect, radii, *shadow, color, clip);
            }
        }
    }
}

fn paint_shadow(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    rect: overflow_clip::ClipRect,
    radii: [[f32; 2]; 4],
    shadow: BoxShadow,
    color: [u8; 4],
    clip: Option<overflow_clip::ClipRect>,
) {
    if color[3] == 0 {
        return;
    }
    if shadow.inset {
        paint_inset_shadow(pixels, width, height, rect, radii, shadow, color, clip);
    } else {
        paint_outer_shadow(pixels, width, height, rect, radii, shadow, color, clip);
    }
}

fn paint_outer_shadow(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    rect: overflow_clip::ClipRect,
    radii: [[f32; 2]; 4],
    shadow: BoxShadow,
    color: [u8; 4],
    clip: Option<overflow_clip::ClipRect>,
) {
    let shape = overflow_clip::ClipRect {
        left: rect.left + shadow.offset_x - shadow.spread,
        top: rect.top + shadow.offset_y - shadow.spread,
        right: rect.right + shadow.offset_x + shadow.spread,
        bottom: rect.bottom + shadow.offset_y + shadow.spread,
    };
    if shape.right <= shape.left || shape.bottom <= shape.top {
        return;
    }
    let sigma = shadow.blur * 0.5;
    let shadow_radii = radii.map(|radius| {
        [
            (radius[0] + shadow.spread).max(0.0),
            (radius[1] + shadow.spread).max(0.0),
        ]
    });
    let extent = (sigma * 3.0)
        .ceil()
        .max(if sigma == 0.0 { 1.0 } else { 0.0 });
    let top = ((shape.top - extent).floor() as i32).max(0);
    let bottom = ((shape.bottom + extent).ceil() as i32).min(height as i32);
    let left = ((shape.left - extent).floor() as i32).max(0);
    let right = ((shape.right + extent).ceil() as i32).min(width as i32);
    for y in top..bottom {
        for x in left..right {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            if rounded_rect_distance(px, py, rect, radii) <= 0.0 {
                continue;
            }
            let distance = rounded_rect_distance(px, py, shape, shadow_radii);
            let coverage = if distance <= 0.0 {
                1.0
            } else if sigma == 0.0 {
                0.0
            } else {
                (-distance.powi(2) / (2.0 * sigma.powi(2))).exp()
            };
            paint_pixel(pixels, width, height, x, y, color, coverage, clip);
        }
    }
}

fn paint_inset_shadow(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    rect: overflow_clip::ClipRect,
    radii: [[f32; 2]; 4],
    shadow: BoxShadow,
    color: [u8; 4],
    clip: Option<overflow_clip::ClipRect>,
) {
    let sigma = (shadow.blur * 0.5 + shadow.spread.max(0.0)).max(0.5);
    let top = (rect.top.floor() as i32).max(0);
    let bottom = (rect.bottom.ceil() as i32).min(height as i32);
    let left = (rect.left.floor() as i32).max(0);
    let right = (rect.right.ceil() as i32).min(width as i32);
    for y in top..bottom {
        for x in left..right {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            if rounded_rect_distance(px, py, rect, radii) > 0.0 {
                continue;
            }
            let distance = (px - rect.left - shadow.offset_x)
                .min(rect.right - px + shadow.offset_x)
                .min(py - rect.top - shadow.offset_y)
                .min(rect.bottom - py + shadow.offset_y)
                .max(0.0);
            let coverage = (-distance.powi(2) / (2.0 * sigma.powi(2))).exp();
            paint_pixel(pixels, width, height, x, y, color, coverage, clip);
        }
    }
}

fn paint_pixel(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    mut color: [u8; 4],
    coverage: f32,
    clip: Option<overflow_clip::ClipRect>,
) {
    if coverage <= 0.003 {
        return;
    }
    color[3] = (f32::from(color[3]) * coverage).round().clamp(0.0, 255.0) as u8;
    fill_rect_alpha_clipped(
        pixels,
        width,
        height,
        x as f32 - PAGE_PADDING,
        y as f32 - PAGE_PADDING,
        1.0,
        1.0,
        color,
        clip,
    );
}

fn rounded_rect_distance(
    x: f32,
    y: f32,
    rect: overflow_clip::ClipRect,
    radii: [[f32; 2]; 4],
) -> f32 {
    let corner = if x < rect.left + radii[0][0] && y < rect.top + radii[0][1] {
        Some((0, rect.left + radii[0][0], rect.top + radii[0][1]))
    } else if x > rect.right - radii[1][0] && y < rect.top + radii[1][1] {
        Some((1, rect.right - radii[1][0], rect.top + radii[1][1]))
    } else if x > rect.right - radii[2][0] && y > rect.bottom - radii[2][1] {
        Some((2, rect.right - radii[2][0], rect.bottom - radii[2][1]))
    } else if x < rect.left + radii[3][0] && y > rect.bottom - radii[3][1] {
        Some((3, rect.left + radii[3][0], rect.bottom - radii[3][1]))
    } else {
        None
    };
    if let Some((corner, center_x, center_y)) = corner {
        let (radius_x, radius_y) = (radii[corner][0], radii[corner][1]);
        if radius_x > 0.0 && radius_y > 0.0 {
            let normalized = ((x - center_x) / radius_x).hypot((y - center_y) / radius_y);
            return (normalized - 1.0) * radius_x.min(radius_y);
        }
    }
    let dx = (rect.left - x).max(0.0).max(x - rect.right);
    let dy = (rect.top - y).max(0.0).max(y - rect.bottom);
    if dx > 0.0 || dy > 0.0 {
        return dx.hypot(dy);
    }
    -((x - rect.left)
        .min(rect.right - x)
        .min(y - rect.top)
        .min(rect.bottom - y))
}
