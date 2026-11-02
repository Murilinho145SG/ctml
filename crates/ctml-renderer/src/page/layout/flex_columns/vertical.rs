use crate::{
    page::{
        sizing::resolve_layout_width,
        sizing_padding::{resolve_borders, resolve_margin, resolve_padding},
    },
    style::{FlexAlignment, WritingMode},
};

use super::{ColumnLayout, VisualLine};

pub(super) fn apply_vertical(
    layout: &ColumnLayout,
    lines: &[VisualLine],
    line_y: &mut [f32],
    available_width: f32,
    available_height: f32,
    writing_mode: WritingMode,
) {
    let mut tops = Vec::with_capacity(layout.items.len());
    let mut heights = Vec::with_capacity(layout.items.len());
    let mut last_lines = Vec::with_capacity(layout.items.len());
    for item in &layout.items {
        let mut top = f32::INFINITY;
        let mut bottom = f32::NEG_INFINITY;
        let mut last_line = None;
        for line in &item.lines {
            let visual = &lines[*line];
            // Empty inline markers inherit line height but have no cross-axis content.
            if visual.max_x <= visual.min_x && !visual.flow_spacer {
                continue;
            }
            let shifted = visual.top + line_y[*line];
            top = top.min(shifted);
            bottom = bottom.max(shifted + visual.height);
            last_line = Some(*line);
        }
        if last_line.is_none() {
            for line in &item.lines {
                let visual = &lines[*line];
                let shifted = visual.top + line_y[*line];
                top = top.min(shifted);
                bottom = bottom.max(shifted + visual.height);
                last_line = Some(*line);
            }
        }
        if !top.is_finite() || !bottom.is_finite() {
            return;
        }
        tops.push(top);
        heights.push(bottom - top);
        last_lines.push(last_line.unwrap());
    }
    let container_top = tops.iter().copied().fold(f32::INFINITY, f32::min);
    let item_cross_size = heights.iter().copied().fold(0.0, f32::max);
    let cross_size = flex_container_cross_size(
        layout,
        item_cross_size,
        available_width,
        available_height,
        writing_mode,
    );
    let alignment = layout.style.align_items.unwrap_or(FlexAlignment::Stretch);
    // Distributed gaps are zero-width lines; place them on the row's cross-axis start.
    for line in lines
        .iter()
        .filter(|line| line.flex_gap_marker && line.path.last() == Some(&layout.container))
    {
        line_y[line.index] += container_top - (line.top + line_y[line.index]);
    }
    for (index, item) in layout.items.iter().enumerate() {
        let target = match alignment {
            FlexAlignment::Center => container_top + (cross_size - heights[index]) / 2.0,
            FlexAlignment::FlexEnd | FlexAlignment::End => {
                container_top + cross_size - heights[index]
            }
            FlexAlignment::Stretch => {
                let stretch = cross_size - heights[index];
                if stretch > 0.0 && index < last_lines.len() {
                    let last = last_lines[index];
                    line_y[last] += stretch;
                }
                container_top
            }
            _ => container_top,
        };
        let delta = target - tops[index];
        if delta != 0.0 {
            for line in &item.lines {
                line_y[*line] += delta;
            }
        }
    }
    // Include container-owned marker lines before moving the next block into flow.
    let last_line = lines
        .iter()
        .filter(|line| line.path.contains(&layout.container))
        .map(|line| line.index)
        .max();
    if let Some(last_line) = last_line {
        if let Some(next) = lines.get(last_line + 1) {
            let desired_bottom = container_top
                + cross_size
                + container_block_end_decoration(layout, available_width, writing_mode)
                + first_content_line_height(layout, lines);
            let next_top = next.top + line_y[next.index];
            let shift = desired_bottom - next_top;
            if shift != 0.0 {
                for offset in line_y.iter_mut().skip(last_line + 1) {
                    *offset += shift;
                }
            }
        }
    }
}

fn container_block_end_decoration(
    layout: &ColumnLayout,
    available_width: f32,
    writing_mode: WritingMode,
) -> f32 {
    let Some((_, size)) = layout
        .style
        .layout_sizes
        .iter()
        .enumerate()
        .find(|(_, size)| size.node_id == layout.container)
    else {
        return 0.0;
    };
    let padding = resolve_padding(size, available_width);
    let borders = resolve_borders(size, available_width);
    let margins = resolve_margin(size, available_width);
    match writing_mode {
        WritingMode::HorizontalTb => padding[2] + borders[2] + margins[2],
        WritingMode::VerticalRl => padding[3] + borders[3] + margins[3],
        WritingMode::VerticalLr => padding[1] + borders[1] + margins[1],
    }
}

fn first_content_line_height(layout: &ColumnLayout, lines: &[VisualLine]) -> f32 {
    layout
        .items
        .iter()
        .flat_map(|item| &item.lines)
        .filter_map(|index| {
            let line = &lines[*index];
            (line.max_x > line.min_x).then_some((line.index, line.height))
        })
        .min_by_key(|(index, _)| *index)
        .map_or(layout.style.line_height, |(_, height)| height)
}

fn flex_container_cross_size(
    layout: &ColumnLayout,
    item_cross_size: f32,
    available_width: f32,
    available_height: f32,
    writing_mode: WritingMode,
) -> f32 {
    let Some((index, size)) = layout
        .style
        .layout_sizes
        .iter()
        .enumerate()
        .find(|(_, size)| size.node_id == layout.container)
    else {
        return item_cross_size;
    };
    let containing_width = resolve_layout_width(
        &layout.style.layout_sizes[..index],
        available_width,
        writing_mode,
    );
    let box_extent = crate::page::sizing_height::resolve_block_box_extent(
        size,
        item_cross_size,
        available_height,
        containing_width,
        writing_mode,
    );
    let padding = resolve_padding(size, containing_width);
    let borders = resolve_borders(size, containing_width);
    let margins = resolve_margin(size, containing_width);
    let (padding_extent, border_extent, margin_extent) = match writing_mode {
        WritingMode::HorizontalTb => (
            padding[0] + padding[2],
            borders[0] + borders[2],
            margins[0] + margins[2],
        ),
        WritingMode::VerticalRl => (
            padding[1] + padding[3],
            borders[1] + borders[3],
            margins[1] + margins[3],
        ),
        WritingMode::VerticalLr => (
            padding[3] + padding[1],
            borders[3] + borders[1],
            margins[3] + margins[1],
        ),
    };
    item_cross_size.max(box_extent - padding_extent - border_extent - margin_extent)
}

pub(super) fn visual_justify(alignment: FlexAlignment, reverse: bool) -> FlexAlignment {
    if !reverse {
        return alignment;
    }
    match alignment {
        FlexAlignment::FlexStart => FlexAlignment::FlexEnd,
        FlexAlignment::FlexEnd => FlexAlignment::FlexStart,
        _ => alignment,
    }
}
