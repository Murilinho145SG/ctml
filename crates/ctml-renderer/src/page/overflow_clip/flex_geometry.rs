use std::collections::HashMap;

use crate::style::TextRun;

pub(super) fn nearest_flex_box_x(
    runs: &[TextRun],
    node_id: usize,
    flex_item_box_x: &HashMap<usize, f32>,
) -> f32 {
    if let Some(offset) = flex_item_box_x.get(&node_id) {
        return *offset;
    }
    runs.iter()
        .find(|run| {
            run.style
                .layout_sizes
                .iter()
                .any(|size| size.node_id == node_id)
        })
        .and_then(|run| {
            run.element_path
                .iter()
                .rev()
                .find_map(|node| flex_item_box_x.get(node).copied())
        })
        .unwrap_or_default()
}
