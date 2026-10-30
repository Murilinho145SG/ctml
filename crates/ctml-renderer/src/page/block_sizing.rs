use cosmic_text::Buffer;
use std::collections::HashMap;

use crate::style::{LayoutSize, TextRun, WritingMode};

use super::sizing_height::resolve_block_box_extent;

pub(super) fn block_spacer_heights(
    runs: &[TextRun],
    line_sizes: &[Vec<LayoutSize>],
    buffer: &Buffer,
    available_height: f32,
    available_width: f32,
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
        let containing_height = containing_block_height(
            node_id,
            &parents,
            &block_sizes,
            &content_heights,
            available_height,
            available_width,
            page_writing_mode,
        );
        let containing_width = containing_block_width(
            node_id,
            &parents,
            &block_sizes,
            available_width,
            page_writing_mode,
        );
        let sizing_content_height = if size.size_contained {
            0.0
        } else {
            content_height
        };
        let requested_height = resolve_block_box_extent(
            size,
            sizing_content_height,
            containing_height,
            containing_width,
            page_writing_mode,
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

fn containing_block_height(
    node_id: usize,
    parents: &HashMap<usize, usize>,
    block_sizes: &HashMap<usize, LayoutSize>,
    content_heights: &HashMap<usize, f32>,
    available_height: f32,
    available_width: f32,
    page_writing_mode: WritingMode,
) -> f32 {
    let mut ancestors = Vec::new();
    let mut ancestor = parents.get(&node_id).copied();
    while let Some(ancestor_id) = ancestor {
        ancestors.push(ancestor_id);
        ancestor = parents.get(&ancestor_id).copied();
    }
    ancestors.reverse();

    let mut containing_height = available_height;
    for ancestor_id in ancestors {
        let Some(size) = block_sizes.get(&ancestor_id) else {
            continue;
        };
        let (height, min_height, max_height) =
            super::sizing::block_size_constraints(size, page_writing_mode);
        if height.is_none()
            && min_height.is_none()
            && max_height.is_none()
            && size.aspect_ratio.is_none()
        {
            continue;
        }
        let containing_width = containing_block_width(
            ancestor_id,
            parents,
            block_sizes,
            available_width,
            page_writing_mode,
        );
        let sizing_content_height = if size.size_contained {
            0.0
        } else {
            content_heights
                .get(&ancestor_id)
                .copied()
                .unwrap_or_default()
        };
        containing_height = resolve_block_box_extent(
            size,
            sizing_content_height,
            containing_height,
            containing_width,
            page_writing_mode,
        );
    }
    containing_height
}

fn containing_block_width(
    node_id: usize,
    parents: &HashMap<usize, usize>,
    block_sizes: &HashMap<usize, LayoutSize>,
    available_width: f32,
    page_writing_mode: WritingMode,
) -> f32 {
    let mut ancestors = Vec::new();
    let mut ancestor = parents.get(&node_id).copied();
    while let Some(ancestor_id) = ancestor {
        ancestors.push(ancestor_id);
        ancestor = parents.get(&ancestor_id).copied();
    }
    ancestors.reverse();
    let sizes = ancestors
        .iter()
        .filter_map(|ancestor_id| block_sizes.get(ancestor_id).cloned())
        .collect::<Vec<_>>();
    super::sizing::resolve_layout_width(&sizes, available_width, page_writing_mode)
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
