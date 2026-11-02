use std::collections::HashSet;

use crate::{
    page::{
        PageRasterizer,
        sizing_height::resolve_block_box_extent,
        sizing_padding::{resolve_borders, resolve_margin, resolve_padding},
    },
    style::{TextRun, WritingMode},
};

use super::VisualLine;

pub(super) fn apply_auto_main_margins(
    page: &PageRasterizer,
    lines: &[VisualLine],
    line_y: &mut [f32],
    available_width: f32,
) {
    if page.writing_mode() != WritingMode::HorizontalTb {
        return;
    }
    let mut containers = HashSet::new();
    for run in &page.runs {
        if !run.style.display_flex || !run.style.flex_direction_column {
            continue;
        }
        let Some(container) = run.element_path.last().copied() else {
            continue;
        };
        if !containers.insert(container) {
            continue;
        }
        let Some(container_size) = find_layout_size(&page.runs, container) else {
            continue;
        };
        let available_height = page
            .layout_size
            .map_or(container_size.viewport_size.height, |(_, height)| {
                height as f32
            });
        let container_extent = resolve_block_box_extent(
            container_size,
            0.0,
            available_height,
            available_width,
            WritingMode::HorizontalTb,
        );
        let container_padding = resolve_padding(container_size, available_width);
        let container_border = resolve_borders(container_size, available_width);
        let content_bottom = container_extent - container_padding[2] - container_border[2];

        let mut candidates = Vec::new();
        for line in lines {
            let Some(position) = line.path.iter().position(|node| *node == container) else {
                continue;
            };
            let Some(item) = line.path.get(position + 1).copied() else {
                continue;
            };
            let Some(size) = find_layout_size(&page.runs, item) else {
                continue;
            };
            if !size.margin[0]
                .as_deref()
                .is_some_and(|margin| margin.eq_ignore_ascii_case("auto"))
            {
                continue;
            }
            if let Some(candidate) = candidates
                .iter_mut()
                .find(|candidate: &&mut Candidate| candidate.item == item)
            {
                candidate.first_line = candidate.first_line.min(line.index);
                candidate.last_bottom = candidate
                    .last_bottom
                    .max(line.top + line.height + line_y[line.index]);
            } else {
                let padding = resolve_padding(size, available_width);
                let border = resolve_borders(size, available_width);
                let margin = resolve_margin(size, available_width);
                candidates.push(Candidate {
                    item,
                    first_line: line.index,
                    last_bottom: line.top + line.height + line_y[line.index],
                    end_decoration: padding[2] + border[2] + margin[2],
                });
            }
        }

        let Some(candidate) = candidates
            .into_iter()
            .min_by_key(|candidate| candidate.first_line)
        else {
            continue;
        };
        let shift = content_bottom - candidate.end_decoration - candidate.last_bottom;
        if shift > 0.0 {
            for offset in line_y.iter_mut().skip(candidate.first_line) {
                *offset += shift;
            }
        }
    }
}

struct Candidate {
    item: usize,
    first_line: usize,
    last_bottom: f32,
    end_decoration: f32,
}

fn find_layout_size(runs: &[TextRun], node_id: usize) -> Option<&crate::style::LayoutSize> {
    runs.iter().find_map(|run| {
        run.style
            .layout_sizes
            .iter()
            .chain(&run.style.layout_end_sizes)
            .find(|size| size.node_id == node_id)
    })
}
