use std::collections::HashMap;

use crate::style::{BorderLineStyle, LayoutSize, TextStyle};

use super::{
    border_radius::draw_rounded_border,
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
    draw_outline_with_values(
        pixels,
        width,
        height,
        geometry.border_box,
        &size.border_radius,
        size.font_size,
        size.line_height,
        size.font_relative_units,
        size.viewport_size,
        size.outline_width.as_deref(),
        size.outline_style,
        size.outline_color,
        size.outline_offset.as_deref(),
        current_color,
        opacity,
        element_path
            .iter()
            .copied()
            .filter(|node_id| *node_id != size.node_id)
            .collect::<Vec<_>>(),
        clips,
    );
}

pub(super) fn draw_inline_outline(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    style: &TextStyle,
    rect: overflow_clip::ClipRect,
    opacity: f32,
    element_path: &[usize],
    clips: &HashMap<usize, overflow_clip::ClipRect>,
) {
    draw_outline_with_values(
        pixels,
        width,
        height,
        rect,
        &style.border_radius,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
        style.outline_width.as_deref(),
        style.outline_style,
        style.outline_color,
        style.outline_offset.as_deref(),
        &style.color,
        opacity,
        element_path.to_vec(),
        clips,
    );
}

pub(super) fn draw_first_line_outlines(
    runs: &[crate::style::TextRun],
    buffer: &cosmic_text::Buffer,
    spans: &[super::CursorSpan],
    pixels: &mut [u8],
    width: u32,
    height: u32,
    page_writing_mode: crate::style::WritingMode,
    clips: &HashMap<usize, overflow_clip::ClipRect>,
    nested_scroll: &HashMap<usize, (f32, f32)>,
    flex_layout_offsets: &super::FlexLayoutOffsets,
) {
    let first_lines = super::highlights::first_formatted_line_ranges(buffer, spans);
    let vertical_origins = super::local_writing::vertical_block_origins(
        runs,
        buffer,
        page_writing_mode,
        flex_layout_offsets,
    );
    let lines = super::adjusted_layout_runs(buffer, flex_layout_offsets).collect::<Vec<_>>();
    let mut drawn = std::collections::HashSet::new();
    for span in spans {
        let Some(block) = span.first_line_block else {
            continue;
        };
        if !drawn.insert((block, span.run_index)) || span.visibility_hidden {
            continue;
        }
        let Some((line_index, line_end)) = first_lines.get(&block).copied() else {
            continue;
        };
        if span.line != line_index || span.start >= line_end {
            continue;
        }
        let Some(run) = runs.get(span.run_index) else {
            continue;
        };
        let Some((style, path)) = runs.get(span.run_index).and_then(|run| {
            run.first_line_font_style
                .as_ref()
                .map(|style| (style, run.element_path.as_slice()))
        }) else {
            continue;
        };
        let Some(line) = lines.iter().find(|line| line.line_i == line_index) else {
            continue;
        };
        let end = span.end.min(line_end);
        let (mut left, mut right) = (f32::INFINITY, f32::NEG_INFINITY);
        for glyph in line.glyphs.iter() {
            if glyph.start < end && glyph.end > span.start {
                left = left.min(glyph.x);
                right = right.max(glyph.x + glyph.w);
            }
        }
        if !left.is_finite() || !right.is_finite() || right <= left {
            continue;
        }
        let mut rect = overflow_clip::ClipRect {
            left: left + super::PAGE_PADDING,
            top: line.line_top + super::PAGE_PADDING,
            right: right + super::PAGE_PADDING,
            bottom: line.line_top + line.line_height + super::PAGE_PADDING,
        };
        let inline_available = if super::sizing::is_vertical(page_writing_mode) {
            height as f32 - super::PAGE_PADDING * 2.0
        } else {
            width as f32 - super::PAGE_PADDING * 2.0
        };
        let padding = super::layout::resolve_first_letter_padding(
            &run.style,
            style,
            block,
            inline_available.max(0.0),
            page_writing_mode,
        );
        let border: [f32; 4] =
            std::array::from_fn(|side| super::layout::resolve_border_side(style, side));
        rect.left -= padding[3] + border[3];
        rect.right += padding[1] + border[1];
        rect.top -= padding[0] + border[0];
        rect.bottom += padding[2] + border[2];
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
        let (scroll_x, scroll_y) =
            overflow_clip::scroll_translation_for_path(path, nested_scroll, page_writing_mode);
        rect.left -= scroll_x;
        rect.right -= scroll_x;
        rect.top -= scroll_y;
        rect.bottom -= scroll_y;
        draw_inline_outline(
            pixels,
            width,
            height,
            style,
            rect,
            span.opacity * runs[span.run_index].first_line_opacity.unwrap_or(1.0),
            path,
            clips,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_outline_with_values(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    rect: overflow_clip::ClipRect,
    border_radius: &[[Option<String>; 2]; 4],
    font_size: f32,
    line_height: f32,
    font_relative_units: ctml_css::FontRelativeUnits,
    viewport_size: ctml_css::ViewportSize,
    width_value: Option<&str>,
    outline_style: BorderLineStyle,
    outline_color: Option<[u8; 4]>,
    offset_value: Option<&str>,
    current_color: &[u8; 4],
    opacity: f32,
    path: Vec<usize>,
    clips: &HashMap<usize, overflow_clip::ClipRect>,
) {
    if matches!(
        outline_style,
        BorderLineStyle::None | BorderLineStyle::Hidden
    ) {
        return;
    }
    let width_value = width_value.unwrap_or("medium");
    let thickness = match width_value {
        "thin" => 1.0,
        "medium" => 3.0,
        "thick" => 5.0,
        value => super::sizing::resolve_width(
            value,
            0.0,
            font_size,
            line_height,
            font_relative_units,
            viewport_size,
        )
        .unwrap_or_default(),
    };
    if thickness <= 0.0 {
        return;
    }
    let offset = offset_value
        .and_then(|value| {
            ctml_css::parse_math_expression(value, |component| {
                ctml_css::parse_signed_font_relative_length_with_viewport(
                    component,
                    font_size,
                    line_height,
                    font_relative_units,
                    viewport_size,
                )
            })
            .or_else(|| {
                ctml_css::parse_signed_font_relative_length_with_viewport(
                    value,
                    font_size,
                    line_height,
                    font_relative_units,
                    viewport_size,
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
    let clip = overflow_clip::clip_for_path(&path, clips);
    let color = edge_color(outline_color.unwrap_or(*current_color), opacity);
    let radii = super::border_radius::resolve_radii_values(
        border_radius,
        font_size,
        line_height,
        font_relative_units,
        viewport_size,
        rect,
    )
    .map(|radius| {
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
            [outline_style; 4],
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
                outline_style,
                color,
                clip,
            );
        }
    }
}

// Rounded border rasterization lives in `border_radius.rs`.
