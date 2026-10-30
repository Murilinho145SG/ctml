use std::collections::{HashMap, HashSet};

use cosmic_text::Buffer;

use crate::style::{LayoutSize, TextRun};

use super::overflow_clip::{BlockGeometry, ClipRect};

const VIEWPORT_PREPARATION_MARGIN: f32 = 0.5;

pub(super) fn offscreen_auto_nodes(
    runs: &[TextRun],
    geometries: &HashMap<usize, BlockGeometry>,
    width: u32,
    height: u32,
    scroll_offset: f32,
    has_selection: bool,
    focused_path: &[usize],
) -> HashSet<usize> {
    if has_selection {
        return HashSet::new();
    }

    let viewport = ClipRect {
        left: -(width as f32 * VIEWPORT_PREPARATION_MARGIN),
        top: scroll_offset - height as f32 * VIEWPORT_PREPARATION_MARGIN,
        right: width as f32 * (1.0 + VIEWPORT_PREPARATION_MARGIN),
        bottom: scroll_offset + height as f32 * (1.0 + VIEWPORT_PREPARATION_MARGIN),
    };
    let auto_nodes = runs
        .iter()
        .filter(|run| run.style.content_visibility_auto)
        .flat_map(|run| {
            run.style
                .layout_sizes
                .iter()
                .filter(|size| run.element_path.last() == Some(&size.node_id))
                .map(|size| size.node_id)
        })
        .collect::<HashSet<_>>();

    auto_nodes
        .into_iter()
        .filter(|node_id| {
            is_offscreen_auto_node(*node_id, geometries, viewport, has_selection, focused_path)
        })
        .collect()
}

pub(super) fn vertical_scroll_offset(buffer: &Buffer) -> f32 {
    let scroll = buffer.scroll();
    let metrics = buffer.metrics();
    let mut offset = scroll.vertical;
    for buffer_line in buffer.lines.iter().take(scroll.line) {
        offset += buffer_line
            .layout_runs(None, metrics.line_height)
            .map(|line| line.line_height)
            .sum::<f32>();
    }
    offset
}

fn is_offscreen_auto_node(
    node_id: usize,
    geometries: &HashMap<usize, BlockGeometry>,
    viewport: ClipRect,
    has_selection: bool,
    focused_path: &[usize],
) -> bool {
    !has_selection
        && !focused_path.contains(&node_id)
        && geometries
            .get(&node_id)
            .is_some_and(|geometry| !intersects(geometry.border_box, viewport))
}

pub(super) fn skip_auto_contents(runs: &mut [TextRun], node_ids: &HashSet<usize>) {
    for run in runs {
        for node_id in node_ids {
            let Some(path_index) = run
                .element_path
                .iter()
                .position(|candidate| candidate == node_id)
            else {
                continue;
            };
            run.style.content_skipped = true;
            if path_index + 1 < run.element_path.len() {
                run.style.content_skipped_by_ancestor = true;
            }
            for size in &mut run.style.layout_sizes {
                if size.node_id == *node_id {
                    size_contain_for_skipped_auto(size);
                }
            }
            for size in &mut run.style.layout_end_sizes {
                if size.node_id == *node_id {
                    size_contain_for_skipped_auto(size);
                }
            }
            if run.element_path.last() == Some(node_id)
                && run.style.content_visibility_auto
                && run.style.empty_block_height.is_some()
            {
                run.style
                    .empty_block_height
                    .as_mut()
                    .expect("checked empty block height")
                    .size_contained = true;
            }
        }
    }
}

fn size_contain_for_skipped_auto(size: &mut LayoutSize) {
    size.size_contained = true;
    size.inline_size_contained = true;
}

fn intersects(first: ClipRect, second: ClipRect) -> bool {
    first.left < second.right
        && first.right > second.left
        && first.top < second.bottom
        && first.bottom > second.top
}

#[cfg(test)]
mod tests {
    use super::{intersects, is_offscreen_auto_node};
    use crate::page::overflow_clip::{BlockGeometry, ClipRect};
    use std::collections::HashMap;

    fn geometry(left: f32, top: f32, right: f32, bottom: f32) -> BlockGeometry {
        let rect = ClipRect {
            left,
            top,
            right,
            bottom,
        };
        BlockGeometry {
            clip: rect,
            border_box: rect,
            content_block_extent: bottom - top,
            content_inline_extent: right - left,
        }
    }

    #[test]
    fn viewport_margin_keeps_nearby_auto_content_relevant() {
        assert!(intersects(
            geometry(10.0, 140.0, 40.0, 180.0).border_box,
            ClipRect {
                left: 0.0,
                top: -50.0,
                right: 150.0,
                bottom: 150.0,
            }
        ));
        assert!(!intersects(
            geometry(10.0, 151.0, 40.0, 180.0).border_box,
            ClipRect {
                left: 0.0,
                top: -50.0,
                right: 150.0,
                bottom: 150.0,
            }
        ));
    }

    #[test]
    fn focused_or_selected_auto_content_remains_relevant() {
        let geometries = HashMap::from([(7, geometry(10.0, 400.0, 40.0, 440.0))]);
        let viewport = ClipRect {
            left: -50.0,
            top: -50.0,
            right: 150.0,
            bottom: 150.0,
        };

        assert!(is_offscreen_auto_node(7, &geometries, viewport, false, &[]));
        assert!(!is_offscreen_auto_node(
            7,
            &geometries,
            viewport,
            false,
            &[7]
        ));
        assert!(!is_offscreen_auto_node(7, &geometries, viewport, true, &[]));
    }
}
