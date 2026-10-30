use crate::style::{BorderLineStyle, LayoutSize};

use super::{
    PAGE_PADDING,
    borders::{draw_edge, edge_color, shade},
    overflow_clip,
    pixel::fill_rect_alpha_clipped,
};

pub(super) fn resolve_radii(size: &LayoutSize, rect: overflow_clip::ClipRect) -> [[f32; 2]; 4] {
    resolve_radii_values(
        &size.border_radius,
        size.font_size,
        size.line_height,
        size.font_relative_units,
        size.viewport_size,
        rect,
    )
}

pub(super) fn resolve_radii_values(
    values: &[[Option<String>; 2]; 4],
    font_size: f32,
    line_height: f32,
    font_relative_units: ctml_css::FontRelativeUnits,
    viewport_size: ctml_css::ViewportSize,
    rect: overflow_clip::ClipRect,
) -> [[f32; 2]; 4] {
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    let mut radii = std::array::from_fn(|corner| {
        std::array::from_fn(|axis| {
            values[corner][axis]
                .as_deref()
                .and_then(|value| {
                    super::sizing::resolve_width(
                        value,
                        if axis == 0 { width } else { height },
                        font_size,
                        line_height,
                        font_relative_units,
                        viewport_size,
                    )
                })
                .unwrap_or_default()
        })
    });
    normalize_radii(&mut radii, width, height);
    radii
}

fn normalize_radii(radii: &mut [[f32; 2]; 4], width: f32, height: f32) {
    let mut scale: f32 = 1.0;
    for (available, requested) in [
        (width, radii[0][0] + radii[1][0]),
        (width, radii[3][0] + radii[2][0]),
        (height, radii[0][1] + radii[3][1]),
        (height, radii[1][1] + radii[2][1]),
    ] {
        if requested > 0.0 {
            scale = scale.min(available / requested);
        }
    }
    for radius in radii.iter_mut().flatten() {
        *radius *= scale;
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_rounded_border(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    rect: overflow_clip::ClipRect,
    border: [f32; 4],
    radii: [[f32; 2]; 4],
    styles: [BorderLineStyle; 4],
    colors: [[u8; 4]; 4],
    opacity: f32,
    clip: Option<overflow_clip::ClipRect>,
) {
    draw_edge(
        pixels,
        width,
        height,
        rect.left + radii[0][0],
        rect.top,
        (rect.right - radii[1][0]) - (rect.left + radii[0][0]),
        border[0],
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
        rect.right - border[1],
        rect.top + radii[1][1],
        (rect.bottom - radii[2][1]) - (rect.top + radii[1][1]),
        border[1],
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
        rect.left + radii[3][0],
        rect.bottom - border[2],
        (rect.right - radii[2][0]) - (rect.left + radii[3][0]),
        border[2],
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
        rect.top + radii[0][1],
        (rect.bottom - radii[3][1]) - (rect.top + radii[0][1]),
        border[3],
        false,
        3,
        styles[3],
        edge_color(colors[3], opacity),
        clip,
    );

    let inner = overflow_clip::ClipRect {
        left: rect.left + border[3],
        top: rect.top + border[0],
        right: (rect.right - border[1]).max(rect.left + border[3]),
        bottom: (rect.bottom - border[2]).max(rect.top + border[0]),
    };
    let mut inner_radii = [
        [
            (radii[0][0] - border[3]).max(0.0),
            (radii[0][1] - border[0]).max(0.0),
        ],
        [
            (radii[1][0] - border[1]).max(0.0),
            (radii[1][1] - border[0]).max(0.0),
        ],
        [
            (radii[2][0] - border[1]).max(0.0),
            (radii[2][1] - border[2]).max(0.0),
        ],
        [
            (radii[3][0] - border[3]).max(0.0),
            (radii[3][1] - border[2]).max(0.0),
        ],
    ];
    normalize_radii(
        &mut inner_radii,
        inner.right - inner.left,
        inner.bottom - inner.top,
    );
    for corner in 0..4 {
        draw_rounded_corner(
            pixels,
            width,
            height,
            rect,
            inner,
            radii,
            inner_radii,
            border,
            styles,
            colors,
            opacity,
            clip,
            corner,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_rounded_corner(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    outer: overflow_clip::ClipRect,
    inner: overflow_clip::ClipRect,
    radii: [[f32; 2]; 4],
    inner_radii: [[f32; 2]; 4],
    border: [f32; 4],
    styles: [BorderLineStyle; 4],
    colors: [[u8; 4]; 4],
    opacity: f32,
    clip: Option<overflow_clip::ClipRect>,
    corner: usize,
) {
    let (left, top, right, bottom) = match corner {
        0 => (
            outer.left,
            outer.top,
            outer.left + radii[0][0],
            outer.top + radii[0][1],
        ),
        1 => (
            outer.right - radii[1][0],
            outer.top,
            outer.right,
            outer.top + radii[1][1],
        ),
        2 => (
            outer.right - radii[2][0],
            outer.bottom - radii[2][1],
            outer.right,
            outer.bottom,
        ),
        _ => (
            outer.left,
            outer.bottom - radii[3][1],
            outer.left + radii[3][0],
            outer.bottom,
        ),
    };
    if right <= left || bottom <= top {
        return;
    }
    let horizontal_side = if corner < 2 { 0 } else { 2 };
    let vertical_side = if corner == 0 || corner == 3 { 3 } else { 1 };
    for y in top.floor() as i32..bottom.ceil() as i32 {
        for x in left.floor() as i32..right.ceil() as i32 {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            if !inside_rounded_rect(px, py, outer, radii)
                || inside_rounded_rect(px, py, inner, inner_radii)
            {
                continue;
            }
            let near_horizontal = match corner {
                0 => py - outer.top <= px - outer.left,
                1 => py - outer.top <= outer.right - px,
                2 => outer.bottom - py <= outer.right - px,
                _ => outer.bottom - py <= px - outer.left,
            };
            let side = if near_horizontal {
                horizontal_side
            } else {
                vertical_side
            };
            if !border_pixel_on_style(px, py, outer, border[side], side, styles[side]) {
                continue;
            }
            fill_rect_alpha_clipped(
                pixels,
                width,
                height,
                x as f32 - PAGE_PADDING,
                y as f32 - PAGE_PADDING,
                1.0,
                1.0,
                edge_color(shade_for_side(colors[side], side, styles[side]), opacity),
                clip,
            );
        }
    }
}

fn inside_rounded_rect(
    x: f32,
    y: f32,
    rect: overflow_clip::ClipRect,
    radii: [[f32; 2]; 4],
) -> bool {
    let corner = if x < rect.left + radii[0][0] && y < rect.top + radii[0][1] {
        Some(0)
    } else if x > rect.right - radii[1][0] && y < rect.top + radii[1][1] {
        Some(1)
    } else if x > rect.right - radii[2][0] && y > rect.bottom - radii[2][1] {
        Some(2)
    } else if x < rect.left + radii[3][0] && y > rect.bottom - radii[3][1] {
        Some(3)
    } else {
        None
    };
    let Some(corner) = corner else {
        return x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom;
    };
    let (rx, ry) = (radii[corner][0], radii[corner][1]);
    if rx <= 0.0 || ry <= 0.0 {
        return true;
    }
    let (cx, cy) = match corner {
        0 => (rect.left + rx, rect.top + ry),
        1 => (rect.right - rx, rect.top + ry),
        2 => (rect.right - rx, rect.bottom - ry),
        _ => (rect.left + rx, rect.bottom - ry),
    };
    ((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2) <= 1.0
}

fn border_pixel_on_style(
    x: f32,
    y: f32,
    rect: overflow_clip::ClipRect,
    thickness: f32,
    side: usize,
    style: BorderLineStyle,
) -> bool {
    if thickness <= 0.0 || matches!(style, BorderLineStyle::None | BorderLineStyle::Hidden) {
        return false;
    }
    let along = if side == 0 || side == 2 {
        x - rect.left
    } else {
        y - rect.top
    };
    let across = match side {
        0 => y - rect.top,
        1 => rect.right - x,
        2 => rect.bottom - y,
        _ => x - rect.left,
    };
    match style {
        BorderLineStyle::Dashed => {
            (along % (thickness * 6.0).max(8.0)) < (thickness * 3.0).max(4.0)
        }
        BorderLineStyle::Dotted => (along % (thickness * 2.0).max(2.0)) < thickness.max(1.0),
        BorderLineStyle::Double if thickness >= 3.0 => {
            across < thickness / 3.0 || across > thickness * 2.0 / 3.0
        }
        _ => true,
    }
}

fn shade_for_side(color: [u8; 4], side: usize, style: BorderLineStyle) -> [u8; 4] {
    match style {
        BorderLineStyle::Inset if matches!(side, 0 | 3) => shade(color, true),
        BorderLineStyle::Outset if matches!(side, 1 | 2) => shade(color, true),
        BorderLineStyle::Groove if matches!(side, 0 | 3) => shade(color, true),
        BorderLineStyle::Ridge if matches!(side, 1 | 2) => shade(color, true),
        BorderLineStyle::Inset
        | BorderLineStyle::Outset
        | BorderLineStyle::Groove
        | BorderLineStyle::Ridge => shade(color, false),
        _ => color,
    }
}
