use std::{collections::HashMap, time::Instant};

use crate::style::{ElementTransitionStyle, TransitionTiming};

use super::{ColorTransition, PageRasterizer, interpolate_color, start_transition};

pub(super) fn advance(page: &mut PageRasterizer, now: Instant) -> bool {
    let mut completed = Vec::new();
    for (node_id, transitions) in page
        .border_color_transitions
        .iter()
        .map(|(node_id, transitions)| (*node_id, *transitions))
        .collect::<Vec<_>>()
    {
        let colors = std::array::from_fn(|side| interpolate_color(&transitions[side], now));
        apply(page, node_id, colors);
        page.element_border_colors.insert(node_id, colors);
        if now >= transitions[0].starts_at + transitions[0].duration {
            completed.push(node_id);
        }
    }
    for node_id in completed {
        page.border_color_transitions.remove(&node_id);
    }
    !page.border_color_transitions.is_empty()
}

pub(super) fn begin_transition(
    page: &mut PageRasterizer,
    node_id: usize,
    settings: &ElementTransitionStyle,
    timing: TransitionTiming,
    previous_colors: &HashMap<usize, [[u8; 4]; 4]>,
    now: Instant,
) {
    let from = previous_colors
        .get(&node_id)
        .copied()
        .unwrap_or(settings.border_color_value);
    let to = settings.border_color_value;
    let Some(timing) = start_transition(
        0.0,
        1.0,
        from != to,
        timing.duration_ms,
        timing.delay_ms,
        timing.easing,
    ) else {
        return;
    };
    let transitions = std::array::from_fn(|side| ColorTransition {
        from: from[side],
        to: to[side],
        starts_at: timing.starts_at,
        duration: timing.duration,
        easing: timing.easing,
    });
    let colors = std::array::from_fn(|side| interpolate_color(&transitions[side], now));
    apply(page, node_id, colors);
    page.element_border_colors.insert(node_id, colors);
    if now < timing.starts_at + timing.duration {
        page.border_color_transitions.insert(node_id, transitions);
    }
}

fn apply(page: &mut PageRasterizer, node_id: usize, colors: [[u8; 4]; 4]) {
    if node_id == 0 {
        page.root_style.border_color = colors.map(Some);
        return;
    }
    for size in page
        .runs
        .iter_mut()
        .flat_map(|run| run.style.layout_sizes.iter_mut())
        .filter(|size| size.node_id == node_id)
    {
        size.border_color = colors.map(Some);
    }
}
