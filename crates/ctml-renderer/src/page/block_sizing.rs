use cosmic_text::Buffer;
use std::collections::HashMap;

use crate::style::{LayoutSize, TextRun, WritingMode};

use super::{
    sizing::{is_vertical, select_axis_size},
    sizing_height::resolve_content_block_height,
};

pub(super) fn block_spacer_heights(
    runs: &[TextRun],
    line_sizes: &[Vec<LayoutSize>],
    buffer: &Buffer,
    available_height: f32,
    _font_size: f32,
    default_line_height: f32,
    page_writing_mode: WritingMode,
) -> HashMap<usize, f32> {
    let mut content_heights = HashMap::<usize, f32>::new();
    let mut block_sizes = HashMap::<usize, LayoutSize>::new();
    let mut block_depths = HashMap::<usize, usize>::new();
    let mut parents = HashMap::<usize, usize>::new();

    for (line_index, sizes) in line_sizes.iter().enumerate() {
        let Some(line) = buffer.lines.get(line_index) else {
            continue;
        };
        let Some(layouts) = line.layout_opt() else {
            continue;
        };
        let content_height = layouts
            .iter()
            .map(|layout| layout.line_height_opt.unwrap_or(default_line_height))
            .sum::<f32>();
        for (depth, size) in sizes.iter().enumerate() {
            *content_heights.entry(size.node_id).or_default() += content_height;
            block_depths
                .entry(size.node_id)
                .and_modify(|current| *current = (*current).max(depth))
                .or_insert(depth);
            if let Some(parent) = depth.checked_sub(1).and_then(|index| sizes.get(index)) {
                parents.entry(size.node_id).or_insert(parent.node_id);
            }
        }
    }

    for run in runs {
        for size in &run.style.layout_end_sizes {
            block_sizes.insert(size.node_id, size.clone());
        }
    }

    let mut block_ids = block_sizes.keys().copied().collect::<Vec<_>>();
    block_ids.sort_by_key(|node_id| {
        std::cmp::Reverse(block_depths.get(node_id).copied().unwrap_or_default())
    });

    let mut spacers = HashMap::new();
    for node_id in block_ids {
        let size = &block_sizes[&node_id];
        let content_height = content_heights.get(&node_id).copied().unwrap_or_default();
        let same_axis_orientation =
            is_vertical(size.writing_mode) == is_vertical(page_writing_mode);
        let (
            logical_size,
            logical_min,
            logical_max,
            logical_name,
            logical_min_name,
            logical_max_name,
        ) = if same_axis_orientation {
            (
                &size.block_size,
                &size.min_block_size,
                &size.max_block_size,
                "block-size",
                "min-block-size",
                "max-block-size",
            )
        } else {
            (
                &size.inline_size,
                &size.min_inline_size,
                &size.max_inline_size,
                "inline-size",
                "min-inline-size",
                "max-inline-size",
            )
        };
        let (
            physical_size,
            physical_min,
            physical_max,
            physical_name,
            physical_min_name,
            physical_max_name,
        ) = if is_vertical(page_writing_mode) {
            (
                &size.width,
                &size.min_width,
                &size.max_width,
                "width",
                "min-width",
                "max-width",
            )
        } else {
            (
                &size.height,
                &size.min_height,
                &size.max_height,
                "height",
                "min-height",
                "max-height",
            )
        };
        let block_size = select_axis_size(
            logical_size.as_ref(),
            physical_size.as_ref(),
            &size.size_property_orders,
            logical_name,
            physical_name,
        )
        .flatten();
        let min_block_size = select_axis_size(
            logical_min.as_ref(),
            physical_min.as_ref(),
            &size.size_property_orders,
            logical_min_name,
            physical_min_name,
        )
        .flatten();
        let max_block_size = select_axis_size(
            logical_max.as_ref(),
            physical_max.as_ref(),
            &size.size_property_orders,
            logical_max_name,
            physical_max_name,
        )
        .flatten();
        let requested_height = resolve_content_block_height(
            block_size,
            min_block_size,
            max_block_size,
            content_height,
            available_height,
            size.font_size,
            size.line_height,
            size.font_relative_units,
            size.viewport_size,
        );
        let flow_adjustment = requested_height - content_height;
        if flow_adjustment.abs() < f32::EPSILON {
            continue;
        }

        spacers.insert(node_id, flow_adjustment);
        let mut ancestor = parents.get(&node_id).copied();
        while let Some(ancestor_id) = ancestor {
            let content_height = content_heights.entry(ancestor_id).or_default();
            *content_height = (*content_height + flow_adjustment).max(0.0);
            ancestor = parents.get(&ancestor_id).copied();
        }
    }
    spacers
}

pub(super) fn insert_block_spacers(
    runs: &[TextRun],
    spacer_heights: &HashMap<usize, f32>,
) -> Option<Vec<TextRun>> {
    let mut output = Vec::with_capacity(runs.len());
    let mut inserted = false;
    for run in runs {
        output.push((*run).clone());
        for size in &run.style.layout_end_sizes {
            let Some(height) = spacer_heights.get(&size.node_id).copied() else {
                continue;
            };
            if height == 0.0 {
                continue;
            }
            let mut style = run.style.clone();
            style.line_height = height;
            style.line_height_normal = false;
            style.layout_sizes.clear();
            style.layout_end_sizes.clear();
            style.empty_block_height = None;
            style.flow_spacer = true;
            output.push(TextRun {
                text: "\u{200b}\n".to_owned(),
                source_first_letter_character: None,
                style,
                element_path: Vec::new(),
                selection_background: None,
                selection_foreground: None,
                first_line_block: None,
                first_line_foreground: None,
                first_line_background: None,
                first_line_decoration: None,
                first_line_opacity: None,
                first_line_font_style: None,
                first_letter_block: None,
                first_letter_foreground: None,
                first_letter_background: None,
                first_letter_decoration: None,
                first_letter_opacity: None,
                first_letter_font_style: None,
            });
            inserted = true;
        }
    }
    inserted.then_some(output)
}
