use std::{collections::HashMap, time::Instant};

use crate::style::{TextStyle, TransitionEasing};

use super::PageRasterizer;

pub(super) struct OpacityTransition {
    pub(super) from: f32,
    pub(super) to: f32,
    pub(super) starts_at: Instant,
    pub(super) duration: std::time::Duration,
    pub(super) easing: TransitionEasing,
}

pub(super) fn advance(page: &mut PageRasterizer, now: Instant) -> Option<bool> {
    if page.root_opacity_transition.is_none() && page.opacity_transitions.is_empty() {
        return None;
    }

    let root_active = page
        .root_opacity_transition
        .as_ref()
        .is_some_and(|transition| {
            page.root_opacity = interpolate_opacity(transition, now);
            now < transition.starts_at + transition.duration
        });
    if !root_active {
        page.root_opacity_transition = None;
    }

    let mut completed = Vec::new();
    for (node_id, transition) in &page.opacity_transitions {
        if let Some(opacity) = page.opacity_groups.get_mut(node_id) {
            *opacity = interpolate_opacity(transition, now);
        }
        if now >= transition.starts_at + transition.duration {
            completed.push(*node_id);
        }
    }
    for node_id in completed {
        page.opacity_transitions.remove(&node_id);
    }
    Some(root_active || !page.opacity_transitions.is_empty())
}

pub(super) fn begin_transitions(
    page: &mut PageRasterizer,
    previous_root_opacity: f32,
    previous_opacity_groups: &HashMap<usize, f32>,
    root_style: &TextStyle,
) {
    page.root_opacity = root_style.opacity;
    page.root_opacity_transition = start_transition(
        previous_root_opacity,
        root_style.opacity,
        root_style.transition_opacity,
        root_style.transition_duration_ms,
        root_style.transition_delay_ms,
        root_style.transition_easing,
    );
    if let Some(transition) = page.root_opacity_transition.as_ref() {
        page.root_opacity = interpolate_opacity(transition, Instant::now());
    }

    page.opacity_transitions.clear();
    let now = Instant::now();
    let mut transitions_by_node = HashMap::new();
    for run in &page.runs {
        let Some(node_id) = run.element_path.last().copied() else {
            continue;
        };
        if run.style.transition_opacity {
            transitions_by_node.insert(
                node_id,
                (
                    run.style.transition_duration_ms,
                    run.style.transition_delay_ms,
                    run.style.transition_easing,
                ),
            );
        }
    }
    for (node_id, (duration_ms, delay_ms, easing)) in transitions_by_node {
        let Some(target) = page.opacity_groups.get(&node_id).copied() else {
            continue;
        };
        let from = previous_opacity_groups
            .get(&node_id)
            .copied()
            .unwrap_or(1.0);
        let Some(transition) = start_transition(from, target, true, duration_ms, delay_ms, easing)
        else {
            continue;
        };
        page.opacity_groups
            .insert(node_id, interpolate_opacity(&transition, now));
        if now < transition.starts_at + transition.duration {
            page.opacity_transitions.insert(node_id, transition);
        }
    }
}

fn start_transition(
    from: f32,
    to: f32,
    enabled: bool,
    duration_ms: f32,
    delay_ms: f32,
    easing: TransitionEasing,
) -> Option<OpacityTransition> {
    if !enabled || duration_ms <= 0.0 || (from - to).abs() <= f32::EPSILON {
        return None;
    }
    let duration = duration_from_millis(duration_ms);
    let delay = duration_from_millis(delay_ms.abs());
    let now = Instant::now();
    let starts_at = if delay_ms < 0.0 {
        now.checked_sub(delay).unwrap_or(now)
    } else {
        now.checked_add(delay).unwrap_or(now)
    };
    let transition = OpacityTransition {
        from,
        to,
        starts_at,
        duration,
        easing,
    };
    (now < starts_at + duration).then_some(transition)
}

fn duration_from_millis(milliseconds: f32) -> std::time::Duration {
    let seconds = f64::from(milliseconds.max(0.0)) / 1000.0;
    let duration = std::time::Duration::from_secs_f64(seconds.min(315_360_000.0));
    if milliseconds > 0.0 && duration.is_zero() {
        std::time::Duration::from_nanos(1)
    } else {
        duration
    }
}

fn interpolate_opacity(transition: &OpacityTransition, now: Instant) -> f32 {
    if now < transition.starts_at {
        return transition.from;
    }
    let progress = (now.duration_since(transition.starts_at).as_secs_f32()
        / transition.duration.as_secs_f32())
    .clamp(0.0, 1.0);
    let eased = ease_transition(progress, transition.easing);
    transition.from + (transition.to - transition.from) * eased
}

fn ease_transition(progress: f32, easing: TransitionEasing) -> f32 {
    if easing == TransitionEasing::Linear {
        return progress;
    }
    let (x1, y1, x2, y2) = match easing {
        TransitionEasing::Linear => return progress,
        TransitionEasing::Ease => (0.25, 0.1, 0.25, 1.0),
        TransitionEasing::EaseIn => (0.42, 0.0, 1.0, 1.0),
        TransitionEasing::EaseOut => (0.0, 0.0, 0.58, 1.0),
        TransitionEasing::EaseInOut => (0.42, 0.0, 0.58, 1.0),
    };
    let cubic = |t: f32, first: f32, second: f32| {
        3.0 * (1.0 - t).powi(2) * t * first + 3.0 * (1.0 - t) * t.powi(2) * second + t.powi(3)
    };
    let mut lower = 0.0;
    let mut upper = 1.0;
    for _ in 0..12 {
        let parameter = (lower + upper) * 0.5;
        if cubic(parameter, x1, x2) < progress {
            lower = parameter;
        } else {
            upper = parameter;
        }
    }
    cubic((lower + upper) * 0.5, y1, y2)
}
