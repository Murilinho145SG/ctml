use std::collections::HashMap;

use crate::style::{BorderLineStyle, LayoutSize};

use super::{
    border_radius::{draw_rounded_border, resolve_radii},
    borders::{draw_edge, edge_color},
    overflow_clip::{self, BlockGeometry},
};

pub(super) fn draw_outline(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    size: &LayoutSize,
    geometry: &BlockGeometry,
    current_color: &[u8; 4],
    opacity: f32,
    element_path: &[usize],
    clips: &HashMap<usize, overflow_clip::ClipRect>,
) {
    if matches!(
        size.outline_style,
        BorderLineStyle::None | BorderLineStyle::Hidden
    ) {
        return;
    }
    let rect = geometry.border_box;
    let width_value = size.outline_width.as_deref().unwrap_or("medium");
    let thickness = match width_value {
        "thin" => 1.0,
        "medium" => 3.0,
        "thick" => 5.0,
        value => super::sizing::resolve_width(
            value,
            0.0,
            size.font_size,
            size.line_height,
            size.font_relative_units,
            size.viewport_size,
        )
        .unwrap_or_default(),
    };
    if thickness <= 0.0 {
        return;
    }
    let offset = size
        .outline_offset
        .as_deref()
        .and_then(|value| {
            ctml_css::parse_math_expression(value, |component| {
                ctml_css::parse_signed_font_relative_length_with_viewport(
                    component,
                    size.font_size,
                    size.line_height,
                    size.font_relative_units,
                    size.viewport_size,
                )
            })
            .or_else(|| {
                ctml_css::parse_signed_font_relative_length_with_viewport(
                    value,
                    size.font_size,
                    size.line_height,
                    size.font_relative_units,
                    size.viewport_size,
                )
            })
        })
        .unwrap_or_default();
    let outward = offset + thickness;
    let expanded = overflow_clip::ClipRect {
        left: rect.left - outward,
        top: rect.top - outward,
        right: rect.right + outward,
        bottom: rect.bottom + outward,
    };
    if expanded.right <= expanded.left || expanded.bottom <= expanded.top {
        return;
    }
    let parent_path = element_path
        .iter()
        .copied()
        .filter(|node_id| *node_id != size.node_id)
        .collect::<Vec<_>>();
    let clip = overflow_clip::clip_for_path(&parent_path, clips);
    let color = edge_color(size.outline_color.unwrap_or(*current_color), opacity);
    let radii = resolve_radii(size, rect).map(|radius| {
        [
            (radius[0] + outward).max(0.0),
            (radius[1] + outward).max(0.0),
        ]
    });
    if radii.iter().flatten().any(|radius| *radius > 0.0) {
        draw_rounded_border(
            pixels,
            width,
            height,
            expanded,
            [thickness; 4],
            radii,
            [size.outline_style; 4],
            [color; 4],
            1.0,
            clip,
        );
    } else {
        for (side, x, y, length, horizontal) in [
            (
                0,
                expanded.left,
                expanded.top,
                expanded.right - expanded.left,
                true,
            ),
            (
                1,
                expanded.right - thickness,
                expanded.top,
                expanded.bottom - expanded.top,
                false,
            ),
            (
                2,
                expanded.left,
                expanded.bottom - thickness,
                expanded.right - expanded.left,
                true,
            ),
            (
                3,
                expanded.left,
                expanded.top,
                expanded.bottom - expanded.top,
                false,
            ),
        ] {
            draw_edge(
                pixels,
                width,
                height,
                x,
                y,
                length,
                thickness,
                horizontal,
                side,
                size.outline_style,
                color,
                clip,
            );
        }
    }
}

// Rounded border rasterization lives in `border_radius.rs`.
