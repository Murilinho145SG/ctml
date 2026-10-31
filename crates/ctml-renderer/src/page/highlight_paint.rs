use cosmic_text::Cursor;

use super::{
    local_writing::LocalVerticalTransform,
    overflow_clip::{BackgroundBoxes, ClipRect},
    pixel::fill_rect_alpha_clipped,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_highlight_background_images(
    style: &crate::style::TextStyle,
    opacity: f32,
    line: usize,
    line_top: f32,
    line_height: f32,
    start: usize,
    end: usize,
    run: &super::PageLayoutRun<'_>,
    local_transform: Option<LocalVerticalTransform>,
    scroll_translation: (f32, f32),
    element_scroll: (f32, f32),
    clip: Option<ClipRect>,
    pixels: &mut [u8],
    width: u32,
    height: u32,
    image_cache: &mut super::background_images::ImageCache,
) {
    if style.background_images.is_empty() {
        return;
    }
    let line_top = line_top + run.offset_for_range(start, end);
    let start = Cursor {
        line,
        index: start,
        affinity: cosmic_text::Affinity::Before,
    };
    let end = Cursor {
        line,
        index: end,
        affinity: cosmic_text::Affinity::After,
    };
    for (left, span_width) in run.highlight(start, end) {
        let (left, top, span_width, span_height) =
            local_transform.map_or((left, line_top, span_width, line_height), |transform| {
                transform.map_rect(
                    left + super::PAGE_PADDING,
                    line_top + super::PAGE_PADDING,
                    span_width,
                    line_height,
                )
            });
        let rect = ClipRect {
            left: left - scroll_translation.0,
            top: top - scroll_translation.1,
            right: left + span_width - scroll_translation.0,
            bottom: top + span_height - scroll_translation.1,
        };
        let boxes = BackgroundBoxes {
            border: rect,
            padding: rect,
            content: rect,
        };
        let radii = super::border_radius::resolve_radii_values(
            &style.border_radius,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
            rect,
        );
        super::background_images::draw_background_image_layers(
            &style.background_images,
            &style.background_blend_modes,
            &style.background_repeats,
            &style.background_positions,
            &style.background_sizes,
            &style.background_attachments,
            &style.background_origins,
            &style.background_clips,
            scroll_translation,
            element_scroll,
            opacity,
            pixels,
            width,
            height,
            boxes,
            clip,
            Some(radii),
            image_cache,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn fill_highlight(
    line: usize,
    line_top: f32,
    line_height: f32,
    start: usize,
    end: usize,
    color: [u8; 4],
    run: &super::PageLayoutRun<'_>,
    pixels: &mut [u8],
    width: u32,
    height: u32,
    local_transform: Option<LocalVerticalTransform>,
    clip: Option<ClipRect>,
    translate_x: f32,
    translate_y: f32,
    border_radius: Option<&crate::style::TextStyle>,
) {
    let line_top = line_top + run.offset_for_range(start, end);
    let start = Cursor {
        line,
        index: start,
        affinity: cosmic_text::Affinity::Before,
    };
    let end = Cursor {
        line,
        index: end,
        affinity: cosmic_text::Affinity::After,
    };
    for (left, span_width) in run.highlight(start, end) {
        let (left, top, span_width, line_height) =
            local_transform.map_or((left, line_top, span_width, line_height), |transform| {
                transform.map_rect(
                    left + super::PAGE_PADDING,
                    line_top + super::PAGE_PADDING,
                    span_width,
                    line_height,
                )
            });
        let rect = ClipRect {
            left: left - translate_x,
            top: top - translate_y,
            right: left + span_width - translate_x,
            bottom: top + line_height - translate_y,
        };
        let radii = border_radius.map(|style| {
            super::border_radius::resolve_radii_values(
                &style.border_radius,
                style.font_size,
                style.line_height,
                style.font_relative_units,
                style.viewport_size,
                rect,
            )
        });
        if let Some(radii) =
            radii.filter(|radii| radii.iter().flatten().any(|radius| *radius > 0.0))
        {
            super::pixel::fill_rounded_rect_alpha_clipped(
                pixels, width, height, rect, radii, color, clip,
            );
        } else {
            fill_rect_alpha_clipped(
                pixels,
                width,
                height,
                rect.left - super::PAGE_PADDING,
                rect.top - super::PAGE_PADDING,
                span_width,
                line_height,
                color,
                clip,
            );
        }
    }
}
