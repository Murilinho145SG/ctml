use std::{collections::HashMap, time::Instant};

use crate::{
    page::PageRasterizer,
    style::{ElementTransitionStyle, TransitionTiming},
};

pub(in crate::page) struct VisibilityTransition {
    pub(in crate::page) from_hidden: bool,
    pub(in crate::page) to_hidden: bool,
    pub(in crate::page) starts_at: Instant,
    pub(in crate::page) duration: std::time::Duration,
}

pub(super) fn advance(page: &mut PageRasterizer, now: Instant) -> bool {
    let mut completed = Vec::new();
    for (node_id, transition) in page
        .visibility_transitions
        .iter()
        .map(|(node_id, transition)| {
            (
                *node_id,
                VisibilityTransition {
                    from_hidden: transition.from_hidden,
                    to_hidden: transition.to_hidden,
                    starts_at: transition.starts_at,
                    duration: transition.duration,
                },
            )
        })
        .collect::<Vec<_>>()
    {
        let hidden = visibility_value(&transition, now);
        apply_visibility(page, node_id, hidden);
        page.element_visibility.insert(node_id, hidden);
        if now >= transition.starts_at + transition.duration {
            completed.push(node_id);
        }
    }
    for node_id in completed {
        page.visibility_transitions.remove(&node_id);
    }
    !page.visibility_transitions.is_empty()
}

pub(super) fn begin_transition(
    page: &mut PageRasterizer,
    node_id: usize,
    settings: &ElementTransitionStyle,
    timing: TransitionTiming,
    previous_element_visibility: &HashMap<usize, bool>,
    now: Instant,
) {
    let from_hidden = previous_element_visibility
        .get(&node_id)
        .copied()
        .unwrap_or(settings.visibility_hidden);
    let to_hidden = settings.visibility_hidden;
    if from_hidden == to_hidden {
        return;
    }
    let Some(timing) = super::start_transition(
        0.0,
        1.0,
        true,
        timing.duration_ms,
        timing.delay_ms,
        timing.easing,
    ) else {
        return;
    };
    let transition = VisibilityTransition {
        from_hidden,
        to_hidden,
        starts_at: timing.starts_at,
        duration: timing.duration,
    };
    let hidden = visibility_value(&transition, now);
    apply_visibility(page, node_id, hidden);
    page.element_visibility.insert(node_id, hidden);
    if now < transition.starts_at + transition.duration {
        page.visibility_transitions.insert(node_id, transition);
    }
}

fn visibility_value(transition: &VisibilityTransition, now: Instant) -> bool {
    if now < transition.starts_at + transition.duration / 2 {
        transition.from_hidden
    } else {
        transition.to_hidden
    }
}

fn apply_visibility(page: &mut PageRasterizer, node_id: usize, hidden: bool) {
    for run in &mut page.runs {
        if run.style.visibility_owner == Some(node_id) {
            run.style.visibility_hidden = hidden;
        }
        for size in run
            .style
            .layout_sizes
            .iter_mut()
            .filter(|size| size.node_id == node_id)
        {
            size.visibility_hidden = hidden;
        }
    }
}
