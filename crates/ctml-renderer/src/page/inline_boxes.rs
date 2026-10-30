use std::collections::HashMap;

use cosmic_text::Buffer;

use crate::style::{BorderLineStyle, BoxDecorationBreak, TextRun, WritingMode};

use super::{
    PAGE_PADDING,
    borders::{draw_edge, edge_color},
    local_writing,
    overflow_clip::{self, ClipRect},
    pixel::fill_rect_alpha_clipped,
    sizing::resolve_width,
};

#[derive(Clone, Copy)]
struct Fragment {
    node_id: usize,
    style_index: usize,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    source_start: usize,
    source_end: usize,
}

pub(super) fn draw(
    runs: &[TextRun],
    buffer: &Buffer,
    pixels: &mut [u8],
    width: u32,
    height: u32,
    page_writing_mode: WritingMode,
    clips: &HashMap<usize, ClipRect>,
    nested_scroll: &HashMap<usize, (f32, f32)>,
    image_cache: &mut super::background_images::ImageCache,
) {
    let style_ranges = local_writing::text_style_ranges(runs);
    let line_starts = local_writing::text_line_starts(runs);
    let mut fragments = HashMap::<usize, Vec<Fragment>>::new();
    for line in buffer.layout_runs() {
        let line_start = line_starts.get(line.line_i).copied().unwrap_or(0);
        let mut line_fragments = HashMap::<usize, Fragment>::new();
        for glyph in line.glyphs {
            let Some(style_index) =
                local_writing::style_index_at(&style_ranges, line_start + glyph.start)
            else {
                continue;
            };
            let Some(run) = runs.get(style_index) else {
                continue;
            };
            if !run.style.inline_box || run.style.visibility_hidden || run.style.content_skipped {
                continue;
            }
            let Some(node_id) = run.element_path.last().copied() else {
                continue;
            };
            let has_border = run
                .style
                .border_style
                .iter()
                .any(|style| *style != BorderLineStyle::None);
            let has_outline = run.first_letter_block.is_some()
                && !matches!(
                    run.style.outline_style,
                    BorderLineStyle::None | BorderLineStyle::Hidden
                );
            let has_background = has_inline_background(run);
            if !has_border && !has_outline && !has_background {
                continue;
            }
            let entry = line_fragments.entry(node_id).or_insert(Fragment {
                node_id,
                style_index,
                left: glyph.x,
                right: glyph.x + glyph.w,
                top: line.line_top,
                bottom: line.line_top + line.line_height,
                source_start: glyph.start,
                source_end: glyph.end,
            });
            entry.left = entry.left.min(glyph.x);
            entry.right = entry.right.max(glyph.x + glyph.w);
            entry.source_start = entry.source_start.min(glyph.start);
            entry.source_end = entry.source_end.max(glyph.end);
        }
        for fragment in line_fragments.into_values() {
            fragments
                .entry(fragment.node_id)
                .or_default()
                .push(fragment);
        }
    }

    for node_fragments in fragments.values_mut() {
        node_fragments.sort_by_key(|fragment| fragment.source_start);
        let first_start = node_fragments.first().map(|fragment| fragment.source_start);
        let last_end = node_fragments
            .iter()
            .map(|fragment| fragment.source_end)
            .max();
        for fragment in node_fragments.iter().copied() {
            let Some(run) = runs.get(fragment.style_index) else {
                continue;
            };
            let style = &run.style;
            let has_outline = run.first_letter_block.is_some()
                && !matches!(
                    style.outline_style,
                    BorderLineStyle::None | BorderLineStyle::Hidden
                );
            let has_background = has_inline_background(run);
            let widths: [f32; 4] = std::array::from_fn(|side| {
                if style.border_style[side] == BorderLineStyle::None {
                    return 0.0;
                }
                match style.border_width[side].as_deref().unwrap_or("medium") {
                    "thin" => 1.0,
                    "medium" => 3.0,
                    "thick" => 5.0,
                    value => super::sizing::resolve_width(
                        value,
                        0.0,
                        style.font_size,
                        style.line_height,
                        style.font_relative_units,
                        style.viewport_size,
                    )
                    .unwrap_or_default(),
                }
            });
            let colors = style.border_color.map(|color| color.unwrap_or(style.color));
            let parent_path = run
                .element_path
                .iter()
                .copied()
                .filter(|node_id| *node_id != fragment.node_id)
                .collect::<Vec<_>>();
            let clip = overflow_clip::clip_for_path(&parent_path, clips);
            let sides = if style.box_decoration_break == BoxDecorationBreak::Clone {
                [true; 4]
            } else {
                let first = fragment.source_start == first_start.unwrap_or_default();
                let last = fragment.source_end == last_end.unwrap_or_default();
                let (start_side, end_side) = if style.direction == crate::style::Direction::Ltr {
                    (3, 1)
                } else {
                    (1, 3)
                };
                [
                    true,
                    (end_side == 1 && last) || (start_side == 1 && first),
                    true,
                    (end_side == 3 && last) || (start_side == 3 && first),
                ]
            };
            let left = fragment.left + PAGE_PADDING;
            let length = (fragment.right - fragment.left).max(0.0);
            let fragment_height = (fragment.bottom - fragment.top).max(0.0);
            let block_padding = if let Some(block) = run.first_letter_block {
                run.first_letter_font_style
                    .as_ref()
                    .map_or([0.0; 4], |pseudo_style| {
                        let inline_available = if super::sizing::is_vertical(page_writing_mode) {
                            height as f32 - PAGE_PADDING * 2.0
                        } else {
                            width as f32 - PAGE_PADDING * 2.0
                        };
                        super::layout::resolve_first_letter_padding(
                            style,
                            pseudo_style,
                            block,
                            inline_available.max(0.0),
                            page_writing_mode,
                        )
                    })
            } else {
                [0.0; 4]
            };
            let top = fragment.top + PAGE_PADDING - block_padding[0];
            let fragment_height = fragment_height + block_padding[0] + block_padding[2];
            let (scroll_x, scroll_y) = overflow_clip::scroll_translation_for_path(
                &run.element_path,
                nested_scroll,
                page_writing_mode,
            );
            let element_scroll = run.element_path.last().map_or((0.0, 0.0), |node_id| {
                overflow_clip::scroll_translation_for_path(
                    &[*node_id],
                    nested_scroll,
                    page_writing_mode,
                )
            });
            if has_background {
                let inline_available = if super::sizing::is_vertical(page_writing_mode) {
                    height as f32 - PAGE_PADDING * 2.0
                } else {
                    width as f32 - PAGE_PADDING * 2.0
                };
                let background_padding = resolve_inline_padding(style, inline_available.max(0.0));
                let border_start = if sides[3] { widths[3] } else { 0.0 };
                let border_end = if sides[1] { widths[1] } else { 0.0 };
                let padding_start = if sides[3] { background_padding[3] } else { 0.0 };
                let padding_end = if sides[1] { background_padding[1] } else { 0.0 };
                let background_left = left - border_start - padding_start;
                let background_top = top - widths[0] - background_padding[0];
                let background_width =
                    length + border_start + border_end + padding_start + padding_end;
                let background_height = fragment_height
                    + widths[0]
                    + widths[2]
                    + background_padding[0]
                    + background_padding[2];
                let background_boxes = overflow_clip::BackgroundBoxes {
                    border: ClipRect {
                        left: background_left,
                        top: background_top,
                        right: background_left + background_width,
                        bottom: background_top + background_height,
                    },
                    padding: ClipRect {
                        left: background_left + border_start,
                        top: background_top + widths[0],
                        right: background_left + background_width - border_end,
                        bottom: background_top + background_height - widths[2],
                    },
                    content: ClipRect {
                        left: background_left + border_start + background_padding[3],
                        top: background_top + widths[0] + background_padding[0],
                        right: background_left + background_width
                            - border_end
                            - background_padding[1],
                        bottom: background_top + background_height
                            - widths[2]
                            - background_padding[2],
                    },
                }
                .offset(-scroll_x, -scroll_y);
                let color_clip_box = style.background_clips.last().copied().unwrap_or_default();
                let color_clip_box = background_boxes
                    .get(color_clip_box)
                    .with_offset((PAGE_PADDING, PAGE_PADDING));
                let color_clip =
                    Some(clip.map_or(color_clip_box, |outer| outer.intersect(color_clip_box)));
                for layer in &style.background_layers {
                    let mut color = *layer;
                    color[3] = (f32::from(color[3]) * style.opacity)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                    fill_rect_alpha_clipped(
                        pixels,
                        width,
                        height,
                        background_left,
                        background_top,
                        background_width,
                        background_height,
                        color,
                        color_clip,
                    );
                }
                super::background_images::draw_background_image_layers(
                    &style.background_images,
                    &style.background_repeats,
                    &style.background_positions,
                    &style.background_sizes,
                    &style.background_attachments,
                    &style.background_origins,
                    &style.background_clips,
                    (scroll_x, scroll_y),
                    element_scroll,
                    style.opacity * run.first_letter_opacity.unwrap_or(1.0),
                    pixels,
                    width,
                    height,
                    background_boxes,
                    clip,
                    None,
                    image_cache,
                );
            }
            if has_outline {
                super::outlines::draw_inline_outline(
                    pixels,
                    width,
                    height,
                    style,
                    overflow_clip::ClipRect {
                        left,
                        top,
                        right: left + length,
                        bottom: top + fragment_height,
                    },
                    style.opacity * run.first_letter_opacity.unwrap_or(1.0),
                    &parent_path,
                    clips,
                );
            }
            if run.first_letter_block.is_some()
                && style.border_radius.iter().flatten().any(Option::is_some)
            {
                let rect = overflow_clip::ClipRect {
                    left,
                    top,
                    right: left + length,
                    bottom: top + fragment_height,
                };
                let radii = super::border_radius::resolve_radii_values(
                    &style.border_radius,
                    style.font_size,
                    style.line_height,
                    style.font_relative_units,
                    style.viewport_size,
                    rect,
                );
                super::border_radius::draw_rounded_border(
                    pixels,
                    width,
                    height,
                    rect,
                    widths,
                    radii,
                    style.border_style,
                    colors,
                    style.opacity,
                    clip,
                );
                continue;
            }
            for side in 0..4 {
                if !sides[side] {
                    continue;
                }
                let (x, y, edge_length, thickness, horizontal) = match side {
                    0 => (left, top, length, widths[0], true),
                    1 => (
                        left + length - widths[1],
                        top,
                        fragment_height,
                        widths[1],
                        false,
                    ),
                    2 => (
                        left,
                        top + fragment_height - widths[2],
                        length,
                        widths[2],
                        true,
                    ),
                    _ => (left, top, fragment_height, widths[3], false),
                };
                draw_edge(
                    pixels,
                    width,
                    height,
                    x,
                    y,
                    edge_length,
                    thickness,
                    horizontal,
                    side,
                    style.border_style[side],
                    edge_color(colors[side], style.opacity),
                    clip,
                );
            }
        }
    }
}

fn has_inline_background(run: &TextRun) -> bool {
    if run.first_letter_block.is_some() {
        run.style
            .background_images
            .iter()
            .any(|image| !matches!(image, crate::style::BackgroundImage::None))
    } else {
        !run.style.background_layers.is_empty() || !run.style.background_images.is_empty()
    }
}

fn resolve_inline_padding(style: &crate::style::TextStyle, containing_width: f32) -> [f32; 4] {
    std::array::from_fn(|side| {
        style.padding[side]
            .as_deref()
            .and_then(|value| {
                resolve_width(
                    value,
                    containing_width,
                    style.font_size,
                    style.line_height,
                    style.font_relative_units,
                    style.viewport_size,
                )
            })
            .unwrap_or_default()
    })
}
