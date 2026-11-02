use std::collections::{HashMap, HashSet};

mod item_sizing;
mod vertical;

use item_sizing::{resolve_item_spacing, resolve_item_width};
use vertical::{apply_vertical, visual_justify};

use crate::{
    page::{
        PageRasterizer, local_writing,
        sizing::resolve_layout_width,
        sizing_padding::{
            inline_borders, inline_margin, inline_padding, resolve_borders, resolve_padding,
        },
    },
    style::{FlexAlignment, TextStyle, WritingMode},
};

use crate::page::FlexLayoutOffsetKey;

struct VisualLine {
    index: usize,
    top: f32,
    height: f32,
    min_x: f32,
    max_x: f32,
    flex_gap_marker: bool,
    flow_spacer: bool,
    path: Vec<usize>,
}

struct FlexItem {
    id: usize,
    lines: Vec<usize>,
    width: f32,
    margin_start: f32,
    margin_end: f32,
    flex_shrink: f32,
}

struct ColumnLayout {
    container: usize,
    depth: usize,
    left: f32,
    width: f32,
    style: TextStyle,
    items: Vec<FlexItem>,
}

pub(crate) fn apply_flex_columns(
    page: &mut PageRasterizer,
    content_width: f32,
    content_height: f32,
) {
    page.flex_layout_offsets.retain(|(_, key), _| {
        !matches!(
            key,
            FlexLayoutOffsetKey::PageLineX(_) | FlexLayoutOffsetKey::PageLineY(_)
        )
    });
    page.flex_item_box_x.clear();
    page.flex_item_box_width.clear();
    page.flex_column_extents.clear();

    let mut markers = HashMap::<usize, Vec<TextStyle>>::new();
    for run in &page.runs {
        if !run.style.flex_distributed_gap
            || !run.style.flex_items_row
            || !run.style.display_flex
            || run.style.display_grid
        {
            continue;
        }
        let Some(container) = run.element_path.last().copied() else {
            continue;
        };
        if run.text.contains('\u{200b}') {
            markers
                .entry(container)
                .or_default()
                .push(run.style.clone());
        }
    }
    if markers.is_empty() {
        return;
    }
    if !page.runs.iter().any(|run| run.style.flex_block_item) {
        return;
    }
    let original_size = page.buffer.size();
    if original_size.1.is_some() {
        page.buffer.set_size(original_size.0, None);
        page.buffer.shape_until_scroll(&mut page.font_system, false);
        super::shape_buffer::layout_lines(page, content_width, content_height);
    }

    let style_ranges = local_writing::text_style_ranges(&page.runs);
    let line_starts = local_writing::text_line_starts(&page.runs);
    let mut lines = Vec::new();
    for (visual_line, line) in page.buffer.layout_runs().enumerate() {
        let line_start = line_starts.get(line.line_i).copied().unwrap_or(0);
        let style_index = line.glyphs.first().and_then(|glyph| {
            local_writing::style_index_at(&style_ranges, line_start + glyph.start)
        });
        let path = style_index
            .map(|index| page.runs[index].element_path.clone())
            .unwrap_or_default();
        let min_x = line
            .glyphs
            .iter()
            .map(|glyph| glyph.x)
            .fold(f32::INFINITY, f32::min);
        let max_x = line
            .glyphs
            .iter()
            .map(|glyph| glyph.x + glyph.w)
            .fold(f32::NEG_INFINITY, f32::max);
        lines.push(VisualLine {
            index: visual_line,
            top: line.line_top,
            height: line.line_height,
            min_x: if min_x.is_finite() { min_x } else { 0.0 },
            max_x: if max_x.is_finite() { max_x } else { 0.0 },
            flex_gap_marker: style_index
                .is_some_and(|index| page.runs[index].style.flex_distributed_gap),
            flow_spacer: style_index.is_some_and(|index| page.runs[index].style.flow_spacer),
            path,
        });
    }

    let writing_mode = page.writing_mode();
    let mut layouts = Vec::new();
    for (container, container_markers) in markers {
        let Some(style) = container_markers.first().cloned() else {
            continue;
        };
        if style.writing_mode != WritingMode::HorizontalTb || style.flex_wrap {
            continue;
        }
        let mut item_lines = HashMap::<usize, Vec<usize>>::new();
        let mut depth = None;
        for line in &lines {
            let Some(position) = line.path.iter().position(|node| *node == container) else {
                continue;
            };
            depth.get_or_insert(position);
            let Some(item) = line.path.get(position + 1).copied() else {
                continue;
            };
            item_lines.entry(item).or_default().push(line.index);
        }
        if item_lines.len() < 2 {
            continue;
        }
        let depth = depth.unwrap_or_default();
        let mut items = item_lines
            .into_iter()
            .map(|(id, mut lines)| {
                lines.sort_unstable();
                (id, lines)
            })
            .collect::<Vec<_>>();
        items.sort_by_key(|(_, lines)| lines.first().copied().unwrap_or_default());
        let disjoint = items
            .windows(2)
            .all(|pair| pair[0].1.last() < pair[1].1.first());
        if !disjoint {
            continue;
        }

        let sizes = &style.layout_sizes;
        let (margin_start, _) = inline_margin(sizes, content_width, writing_mode);
        let (border_start, _) = inline_borders(sizes, content_width, writing_mode);
        let (padding_start, _) = inline_padding(sizes, content_width, writing_mode);
        let container_left = margin_start + border_start + padding_start;
        let container_width = resolve_layout_width(sizes, content_width, writing_mode);

        let all_lines = &lines;
        let mut bases = container_markers
            .iter()
            .map(|marker| marker.flex_basis.clone())
            .collect::<Vec<_>>();
        if bases.len() == items.len() + 1 {
            bases.remove(0);
        }
        let flex_items = items
            .into_iter()
            .enumerate()
            .map(|(index, (id, lines))| {
                let basis = bases.get(index).cloned().flatten();
                let width = resolve_item_width(
                    page,
                    id,
                    &lines,
                    &all_lines,
                    basis.as_ref(),
                    container_width,
                    writing_mode,
                );
                let (margin_start, margin_end, flex_shrink) =
                    resolve_item_spacing(page, id, container_width, writing_mode);
                FlexItem {
                    id,
                    lines,
                    width,
                    margin_start,
                    margin_end,
                    flex_shrink,
                }
            })
            .collect::<Vec<_>>();
        layouts.push(ColumnLayout {
            container,
            depth,
            left: container_left,
            width: container_width,
            style,
            items: flex_items,
        });
    }
    if layouts.is_empty() {
        return;
    }

    let mut line_x = vec![0.0; lines.len()];
    let mut line_y = vec![0.0; lines.len()];

    let mut horizontal = layouts
        .iter()
        .enumerate()
        .map(|(index, layout)| (layout.depth, index))
        .collect::<Vec<_>>();
    horizontal.sort_unstable();
    for (_, index) in horizontal {
        apply_horizontal(
            &mut layouts[index],
            &mut line_x,
            &mut page.flex_item_box_x,
            &mut page.flex_item_box_width,
            writing_mode,
        );
    }

    let mut vertical = layouts
        .iter()
        .enumerate()
        .map(|(index, layout)| (layout.depth, index))
        .collect::<Vec<_>>();
    vertical.sort_unstable_by(|left, right| right.cmp(left));
    for (_, index) in vertical {
        apply_vertical(
            &layouts[index],
            &lines,
            &mut line_y,
            content_width,
            page.layout_size.map_or(content_width, |(_, height)| {
                (height as f32 - crate::page::page_padding() * 2.0).max(0.0)
            }),
            writing_mode,
        );
    }

    let mut containers = layouts
        .iter()
        .map(|layout| (layout.depth, layout.container))
        .collect::<Vec<_>>();
    let mut seen = containers
        .iter()
        .map(|(_, container)| *container)
        .collect::<HashSet<_>>();
    for run in &page.runs {
        if !run.style.display_flex || run.style.display_grid {
            continue;
        }
        let Some(container) = run.element_path.last().copied() else {
            continue;
        };
        if seen.insert(container) {
            containers.push((run.element_path.len(), container));
        }
    }
    containers.sort_unstable_by(|left, right| right.cmp(left));
    for (_, container) in containers {
        let mut item_lines = HashMap::<usize, Vec<usize>>::new();
        for line in &lines {
            let Some(position) = line.path.iter().position(|node| *node == container) else {
                continue;
            };
            let Some(item) = line.path.get(position + 1).copied() else {
                continue;
            };
            item_lines.entry(item).or_default().push(line.index);
        }
        let mut items = item_lines
            .into_iter()
            .map(|(id, mut lines)| {
                lines.sort_unstable();
                (id, lines)
            })
            .collect::<Vec<_>>();
        items.sort_by_key(|(_, lines)| lines.first().copied().unwrap_or_default());
        if items.is_empty()
            || !items
                .windows(2)
                .all(|pair| pair[0].1.last() < pair[1].1.first())
        {
            continue;
        }
        let mut top = f32::INFINITY;
        let mut bottom = f32::NEG_INFINITY;
        for (item, item_lines) in items {
            let Some(first) = item_lines.first() else {
                continue;
            };
            let Some(last) = item_lines.last() else {
                continue;
            };
            let item_top = lines[*first].top + line_y[*first];
            let item_bottom = page
                .flex_column_extents
                .get(&item)
                .map(|(top, height)| {
                    let decoration = item_end_decoration(page, item, content_width);
                    top + height + decoration
                })
                .unwrap_or_else(|| {
                    lines[*last].top
                        + lines[*last].height
                        + line_y[*last]
                        + item_end_decoration(page, item, content_width)
                });
            top = top.min(item_top);
            bottom = bottom.max(item_bottom);
        }
        if top.is_finite() && bottom.is_finite() && bottom > top {
            page.flex_column_extents
                .insert(container, (top, bottom - top));
        }
    }

    for line in &lines {
        let x = line_x[line.index];
        let y = line_y[line.index];
        if x != 0.0 {
            page.flex_layout_offsets
                .insert((line.index, FlexLayoutOffsetKey::PageLineX(line.index)), x);
        }
        if y != 0.0 {
            page.flex_layout_offsets
                .insert((line.index, FlexLayoutOffsetKey::PageLineY(line.index)), y);
        }
    }
}

fn item_end_decoration(page: &PageRasterizer, item: usize, content_width: f32) -> f32 {
    let Some(size) = page.runs.iter().find_map(|run| {
        run.style
            .layout_sizes
            .iter()
            .find(|size| size.node_id == item)
    }) else {
        return 0.0;
    };
    let padding = resolve_padding(size, content_width);
    let border = resolve_borders(size, content_width);
    let margin = crate::page::sizing_padding::resolve_margin(size, content_width);
    padding[2] + border[2] + margin[2]
}

#[allow(clippy::too_many_arguments)]
fn apply_horizontal(
    layout: &mut ColumnLayout,
    line_x: &mut [f32],
    item_box_x: &mut HashMap<usize, f32>,
    item_box_width: &mut HashMap<usize, f32>,
    writing_mode: WritingMode,
) {
    let total = layout
        .items
        .iter()
        .map(|item| item.width + item.margin_start + item.margin_end)
        .sum::<f32>();
    let deficit = (total - layout.width).max(0.0);
    let shrink_weight = layout
        .items
        .iter()
        .map(|item| item.width * item.flex_shrink)
        .sum::<f32>();
    let did_shrink = deficit > 0.0 && shrink_weight > 0.0;
    if did_shrink {
        for item in &mut layout.items {
            let weight = item.width * item.flex_shrink;
            item.width = (item.width - deficit * weight / shrink_weight).max(0.0);
        }
    }
    let total = layout
        .items
        .iter()
        .map(|item| item.width + item.margin_start + item.margin_end)
        .sum::<f32>();
    let free_space = (layout.width - total).max(0.0);
    let count = layout.items.len();
    let justify = visual_justify(
        layout.style.justify_content,
        layout.style.flex_direction_reverse,
    );
    let (mut cursor, gap) = match justify {
        FlexAlignment::Center => (free_space / 2.0, 0.0),
        FlexAlignment::FlexEnd | FlexAlignment::End => (free_space, 0.0),
        FlexAlignment::SpaceBetween => (
            0.0,
            if count > 1 {
                free_space / (count - 1) as f32
            } else {
                0.0
            },
        ),
        FlexAlignment::SpaceAround => {
            let gap = free_space / count as f32;
            (gap / 2.0, gap)
        }
        FlexAlignment::SpaceEvenly => {
            let gap = free_space / (count + 1) as f32;
            (gap, gap)
        }
        _ => (0.0, 0.0),
    };
    let accumulated = layout
        .items
        .first()
        .and_then(|item| item.lines.first())
        .map(|line| line_x[*line])
        .unwrap_or_default();
    item_box_x.insert(layout.container, accumulated + layout.left);
    for item in &layout.items {
        let local = cursor;
        for line in &item.lines {
            line_x[*line] += local;
        }
        item_box_x.insert(
            item.id,
            layout.left + accumulated + local + item.margin_start,
        );
        if did_shrink && writing_mode == WritingMode::HorizontalTb {
            item_box_width.insert(item.id, item.width);
        }
        cursor += item.margin_start + item.width + item.margin_end + gap;
    }
}
