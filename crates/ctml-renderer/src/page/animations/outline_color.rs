use std::{collections::HashMap, time::Instant};

use crate::style::{ElementTransitionStyle, TransitionTiming};

use super::{ColorTransition, PageRasterizer, interpolate_color, start_transition};

pub(super) fn advance(page: &mut PageRasterizer, now: Instant) -> bool {
    let mut completed = Vec::new();
    for (node_id, transition) in page
        .outline_color_transitions
        .iter()
        .map(|(node_id, transition)| (*node_id, *transition))
        .collect::<Vec<_>>()
    {
        let color = interpolate_color(&transition, now);
        apply(page, node_id, color);
        page.element_outline_colors.insert(node_id, color);
        if now >= transition.starts_at + transition.duration {
            completed.push(node_id);
        }
    }
    for node_id in completed {
        page.outline_color_transitions.remove(&node_id);
    }
    !page.outline_color_transitions.is_empty()
}

pub(super) fn begin_transition(
    page: &mut PageRasterizer,
    node_id: usize,
    settings: &ElementTransitionStyle,
    timing: TransitionTiming,
    previous_colors: &HashMap<usize, [u8; 4]>,
    now: Instant,
) {
    let from = previous_colors
        .get(&node_id)
        .copied()
        .unwrap_or(settings.outline_color_value);
    let to = settings.outline_color_value;
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
    let transition = ColorTransition {
        from,
        to,
        starts_at: timing.starts_at,
        duration: timing.duration,
        easing: timing.easing,
    };
    let color = interpolate_color(&transition, now);
    apply(page, node_id, color);
    page.element_outline_colors.insert(node_id, color);
    if now < transition.starts_at + transition.duration {
        page.outline_color_transitions.insert(node_id, transition);
    }
}

pub(super) fn apply(page: &mut PageRasterizer, node_id: usize, color: [u8; 4]) {
    for size in page
        .runs
        .iter_mut()
        .flat_map(|run| run.style.layout_sizes.iter_mut())
        .filter(|size| size.node_id == node_id)
    {
        size.outline_color = Some(color);
    }
}
