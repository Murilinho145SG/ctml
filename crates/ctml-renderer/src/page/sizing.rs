use cosmic_text::{Buffer, Cursor};
use ctml_css::{FontRelativeUnits, parse_font_relative_length};
use std::collections::HashMap;

use crate::style::{LayoutSize, TextRun};

pub(super) fn resolve_width(
    value: &str,
    available_width: f32,
    font_size: f32,
    line_height: f32,
    font_relative_units: FontRelativeUnits,
) -> Option<f32> {
    let value = value.trim();
    if value == "0" {
        return Some(0.0);
    }
    if let Some(percentage) = value.strip_suffix('%') {
        return percentage
            .trim()
            .parse::<f32>()
            .ok()
            .map(|percentage| available_width * percentage / 100.0)
            .filter(|width| width.is_finite() && *width >= 0.0);
    }
    parse_font_relative_length(value, font_size, line_height, font_relative_units)
}

pub(super) fn resolve_layout_width(sizes: &[LayoutSize], available_width: f32) -> f32 {
    let mut content_width = available_width;
    for size in sizes {
        let min_width = size.min_width.as_deref().and_then(|value| {
            resolve_width(
                value,
                content_width,
                size.font_size,
                size.line_height,
                size.font_relative_units,
            )
        });
        let max_width = size.max_width.as_deref().and_then(|value| {
            resolve_width(
                value,
                content_width,
                size.font_size,
                size.line_height,
                size.font_relative_units,
            )
        });
        let specified_width = size
            .width
            .as_deref()
            .and_then(|value| {
                resolve_width(
                    value,
                    content_width,
                    size.font_size,
                    size.line_height,
                    size.font_relative_units,
                )
            })
            .unwrap_or(content_width);
        content_width = match (min_width, max_width) {
            (Some(min), Some(max)) if min > max => specified_width.max(min),
            (Some(min), Some(max)) => specified_width.max(min).min(max),
            (Some(min), None) => specified_width.max(min),
            (None, Some(max)) => specified_width.min(max),
            (None, None) => specified_width,
        };
    }
    content_width.max(1.0)
}

#[cfg(test)]
mod tests {
    use super::resolve_width;
    use ctml_css::FontRelativeUnits;

    #[test]
    fn resolves_font_metric_units_for_box_dimensions() {
        let units = FontRelativeUnits {
            ex: 0.4,
            ch: 0.55,
            ic: 1.0,
            cap: 0.7,
        };
        assert_eq!(resolve_width("2ch", 200.0, 20.0, 24.0, units), Some(22.0));
        assert_eq!(resolve_width("0.5lh", 200.0, 20.0, 24.0, units), Some(12.0));
        assert_eq!(resolve_width("25%", 200.0, 20.0, 24.0, units), Some(50.0));
    }
}

pub(super) fn resolve_layout_height(
    height: Option<&str>,
    min_height: Option<&str>,
    max_height: Option<&str>,
    available_height: f32,
    font_size: f32,
    line_height: f32,
    font_relative_units: FontRelativeUnits,
) -> f32 {
    let min_height = min_height.and_then(|value| {
        resolve_width(
            value,
            available_height,
            font_size,
            line_height,
            font_relative_units,
        )
    });
    let max_height = max_height.and_then(|value| {
        resolve_width(
            value,
            available_height,
            font_size,
            line_height,
            font_relative_units,
        )
    });
    let specified_height = height
        .and_then(|value| {
            resolve_width(
                value,
                available_height,
                font_size,
                line_height,
                font_relative_units,
            )
        })
        .unwrap_or(0.0);
    match (min_height, max_height) {
        (Some(min), Some(max)) if min > max => specified_height.max(min),
        (Some(min), Some(max)) => specified_height.max(min).min(max),
        (Some(min), None) => specified_height.max(min),
        (None, Some(max)) => specified_height.min(max),
        (None, None) => specified_height,
    }
}

pub(super) fn line_layout_sizes(runs: &[TextRun], line_count: usize) -> Vec<Vec<LayoutSize>> {
    let mut sizes = vec![None; line_count];
    let mut line = 0;
    let mut line_has_text = false;
    for run in runs {
        for character in run.text.chars() {
            if character == '\n' {
                line += 1;
                line_has_text = false;
            } else if !line_has_text {
                if let Some(line_sizes) = sizes.get_mut(line) {
                    *line_sizes = Some(run.style.layout_sizes.clone());
                }
                line_has_text = true;
            }
        }
    }
    sizes.into_iter().map(Option::unwrap_or_default).collect()
}

pub(super) fn flow_spacer_lines(runs: &[TextRun], line_count: usize) -> Vec<bool> {
    let mut spacer_lines = vec![false; line_count];
    let mut line = 0;
    for run in runs {
        for character in run.text.chars() {
            if character == '\n' {
                line += 1;
            } else if run.style.flow_spacer
                && let Some(is_spacer) = spacer_lines.get_mut(line)
            {
                *is_spacer = true;
            }
        }
    }
    spacer_lines
}

pub(super) fn cursor_without_flow_spacers(cursor: Cursor, spacer_lines: &[bool]) -> Cursor {
    let preceding_spacers = spacer_lines
        .iter()
        .take(cursor.line)
        .filter(|is_spacer| **is_spacer)
        .count();
    let on_spacer = spacer_lines.get(cursor.line).copied().unwrap_or_default();
    Cursor::new_with_affinity(
        cursor.line.saturating_sub(preceding_spacers),
        if on_spacer { 0 } else { cursor.index },
        cursor.affinity,
    )
}

pub(super) fn cursor_with_flow_spacers(cursor: Cursor, spacer_lines: &[bool]) -> Cursor {
    let mut source_line = 0;
    for (line, is_spacer) in spacer_lines.iter().copied().enumerate() {
        if is_spacer {
            continue;
        }
        if source_line == cursor.line {
            return Cursor::new_with_affinity(line, cursor.index, cursor.affinity);
        }
        source_line += 1;
    }
    Cursor::new_with_affinity(
        spacer_lines.len().saturating_sub(1),
        cursor.index,
        cursor.affinity,
    )
}

pub(super) fn block_spacer_heights(
    runs: &[TextRun],
    line_sizes: &[Vec<LayoutSize>],
    buffer: &Buffer,
    available_height: f32,
    _font_size: f32,
    default_line_height: f32,
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
        let requested_height = resolve_layout_height(
            size.height.as_deref(),
            size.min_height.as_deref(),
            size.max_height.as_deref(),
            available_height,
            size.font_size,
            size.line_height,
            size.font_relative_units,
        );
        let extra_height = (requested_height - content_height).max(0.0);
        if extra_height <= 0.0 {
            continue;
        }

        spacers.insert(node_id, extra_height);
        let mut ancestor = parents.get(&node_id).copied();
        while let Some(ancestor_id) = ancestor {
            *content_heights.entry(ancestor_id).or_default() += extra_height;
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
            if height <= 0.0 {
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
                style,
                element_path: Vec::new(),
                selection_background: None,
                selection_foreground: None,
                first_line_block: None,
                first_line_foreground: None,
                first_line_background: None,
                first_line_decoration: None,
                first_line_font_style: None,
                first_letter_block: None,
                first_letter_foreground: None,
                first_letter_background: None,
                first_letter_decoration: None,
                first_letter_font_style: None,
            });
            inserted = true;
        }
    }
    inserted.then_some(output)
}
