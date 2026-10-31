use std::{collections::HashMap, time::Instant};

use crate::style::{
    ElementTransitionStyle, StepPosition, TextStyle, TransitionEasing, TransitionTiming,
};

use super::PageRasterizer;

pub(super) struct OpacityTransition {
    pub(super) from: f32,
    pub(super) to: f32,
    pub(super) starts_at: Instant,
    pub(super) duration: std::time::Duration,
    pub(super) easing: TransitionEasing,
}

#[derive(Clone, Copy)]
pub(super) struct ColorTransition {
    pub(super) from: [u8; 4],
    pub(super) to: [u8; 4],
    pub(super) starts_at: Instant,
    pub(super) duration: std::time::Duration,
    pub(super) easing: TransitionEasing,
}

pub(super) fn advance(page: &mut PageRasterizer, now: Instant) -> Option<bool> {
    if page.root_opacity_transition.is_none()
        && page.opacity_transitions.is_empty()
        && page.color_transitions.is_empty()
        && page.background_color_transitions.is_empty()
    {
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
    let mut completed_colors = Vec::new();
    for (node_id, transition) in &page.color_transitions {
        let color = interpolate_color(transition, now);
        page.element_colors.insert(*node_id, color);
        for run in &mut page.runs {
            if run.style.color_transition_owner == Some(*node_id) {
                run.style.color = color;
            }
        }
        if now >= transition.starts_at + transition.duration {
            completed_colors.push(*node_id);
        }
    }
    for node_id in completed_colors {
        page.color_transitions.remove(&node_id);
    }
    let mut completed_backgrounds = Vec::new();
    for (node_id, transition) in page
        .background_color_transitions
        .iter()
        .map(|(node_id, transition)| (*node_id, *transition))
        .collect::<Vec<_>>()
    {
        let color = interpolate_color(&transition, now);
        apply_background_color(page, node_id, color);
        page.element_background_colors.insert(node_id, color);
        if now >= transition.starts_at + transition.duration {
            completed_backgrounds.push(node_id);
        }
    }
    for node_id in completed_backgrounds {
        page.background_color_transitions.remove(&node_id);
    }
    Some(
        root_active
            || !page.opacity_transitions.is_empty()
            || !page.color_transitions.is_empty()
            || !page.background_color_transitions.is_empty(),
    )
}

pub(super) fn begin_transitions(
    page: &mut PageRasterizer,
    previous_root_opacity: f32,
    previous_opacity_groups: &HashMap<usize, f32>,
    root_style: &TextStyle,
    transition_styles: &HashMap<usize, ElementTransitionStyle>,
    previous_element_colors: &HashMap<usize, [u8; 4]>,
    previous_element_background_colors: &HashMap<usize, [u8; 4]>,
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
    page.color_transitions.clear();
    page.background_color_transitions.clear();
    let now = Instant::now();
    for (node_id, settings) in transition_styles {
        let Some(target) = page.opacity_groups.get(&node_id).copied() else {
            if let Some(timing) = settings.color {
                begin_color_transition(
                    page,
                    *node_id,
                    settings,
                    timing,
                    previous_element_colors,
                    now,
                );
            }
            if let Some(timing) = settings.background_color {
                begin_background_color_transition(
                    page,
                    *node_id,
                    settings,
                    timing,
                    previous_element_background_colors,
                    now,
                );
            }
            continue;
        };
        let from = previous_opacity_groups
            .get(&node_id)
            .copied()
            .unwrap_or(1.0);
        if let Some(timing) = settings.opacity
            && let Some(transition) = start_transition(
                from,
                target,
                true,
                timing.duration_ms,
                timing.delay_ms,
                timing.easing,
            )
        {
            page.opacity_groups
                .insert(*node_id, interpolate_opacity(&transition, now));
            if now < transition.starts_at + transition.duration {
                page.opacity_transitions.insert(*node_id, transition);
            }
        }
        if let Some(timing) = settings.color {
            begin_color_transition(
                page,
                *node_id,
                settings,
                timing,
                previous_element_colors,
                now,
            );
        }
        if let Some(timing) = settings.background_color {
            begin_background_color_transition(
                page,
                *node_id,
                settings,
                timing,
                previous_element_background_colors,
                now,
            );
        }
    }
}

fn begin_background_color_transition(
    page: &mut PageRasterizer,
    node_id: usize,
    settings: &ElementTransitionStyle,
    timing: TransitionTiming,
    previous_element_background_colors: &HashMap<usize, [u8; 4]>,
    now: Instant,
) {
    let from = previous_element_background_colors
        .get(&node_id)
        .copied()
        .unwrap_or(settings.background_color_value);
    let to = settings.background_color_value;
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
    apply_background_color(page, node_id, color);
    page.element_background_colors.insert(node_id, color);
    if now < transition.starts_at + transition.duration {
        page.background_color_transitions
            .insert(node_id, transition);
    }
}

fn apply_background_color(page: &mut PageRasterizer, node_id: usize, color: [u8; 4]) {
    for run in &mut page.runs {
        if run.element_path.last() == Some(&node_id) {
            run.style.background_color = Some(color);
            if let Some(background) = run.style.background_layers.last_mut() {
                *background = color;
            }
        }
        for size in run
            .style
            .layout_sizes
            .iter_mut()
            .filter(|size| size.node_id == node_id)
        {
            if let Some(background) = size.background_layers.last_mut() {
                *background = color;
            }
        }
    }
}

fn begin_color_transition(
    page: &mut PageRasterizer,
    node_id: usize,
    settings: &ElementTransitionStyle,
    timing: TransitionTiming,
    previous_element_colors: &HashMap<usize, [u8; 4]>,
    now: Instant,
) {
    let from = previous_element_colors
        .get(&node_id)
        .copied()
        .unwrap_or(settings.color_value);
    let to = settings.color_value;
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
    page.element_colors.insert(node_id, color);
    for run in &mut page.runs {
        if run.style.color_transition_owner == Some(node_id) {
            run.style.color = color;
        }
    }
    if now < transition.starts_at + transition.duration {
        page.color_transitions.insert(node_id, transition);
    }
}

fn interpolate_color(transition: &ColorTransition, now: Instant) -> [u8; 4] {
    let progress = if now < transition.starts_at {
        0.0
    } else {
        (now.duration_since(transition.starts_at).as_secs_f32() / transition.duration.as_secs_f32())
            .clamp(0.0, 1.0)
    };
    let progress = ease_transition(progress, transition.easing);
    let from_alpha = f32::from(transition.from[3]) / 255.0;
    let to_alpha = f32::from(transition.to[3]) / 255.0;
    let alpha = from_alpha + (to_alpha - from_alpha) * progress;
    let mut color = [0; 4];
    color[3] = (alpha * 255.0).round().clamp(0.0, 255.0) as u8;
    for (index, channel) in color.iter_mut().take(3).enumerate() {
        let from = f32::from(transition.from[index]) / 255.0 * from_alpha;
        let to = f32::from(transition.to[index]) / 255.0 * to_alpha;
        *channel = if alpha <= f32::EPSILON {
            0
        } else {
            ((from + (to - from) * progress) / alpha * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8
        };
    }
    color
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
    let (x1, y1, x2, y2) = match easing {
        TransitionEasing::Linear => return progress,
        TransitionEasing::Ease => (0.25, 0.1, 0.25, 1.0),
        TransitionEasing::EaseIn => (0.42, 0.0, 1.0, 1.0),
        TransitionEasing::EaseOut => (0.0, 0.0, 0.58, 1.0),
        TransitionEasing::EaseInOut => (0.42, 0.0, 0.58, 1.0),
        TransitionEasing::CubicBezier(values) => {
            let [x1, y1, x2, y2] = values.map(f32::from_bits);
            return cubic_bezier(progress, x1, y1, x2, y2);
        }
        TransitionEasing::Steps { count, position } => return steps(progress, count, position),
    };
    cubic_bezier(progress, x1, y1, x2, y2)
}

fn cubic_bezier(progress: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    if progress <= 0.0 {
        return 0.0;
    }
    if progress >= 1.0 {
        return 1.0;
    }
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

fn steps(progress: f32, count: u32, position: StepPosition) -> f32 {
    let count = count as f32;
    if progress >= 1.0 {
        return 1.0;
    }
    let step = (progress * count).floor();
    match position {
        StepPosition::JumpStart => ((step + 1.0) / count).min(1.0),
        StepPosition::JumpEnd => step / count,
        StepPosition::JumpNone => step / (count - 1.0),
        StepPosition::JumpBoth => (step + 1.0) / (count + 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_cubic_bezier_preserves_endpoints_and_midpoint() {
        let easing = TransitionEasing::CubicBezier([
            0.0f32.to_bits(),
            0.0f32.to_bits(),
            1.0f32.to_bits(),
            1.0f32.to_bits(),
        ]);
        assert_eq!(ease_transition(0.0, easing), 0.0);
        assert!((ease_transition(0.5, easing) - 0.5).abs() < 0.001);
        assert_eq!(ease_transition(1.0, easing), 1.0);
    }

    #[test]
    fn step_easing_uses_requested_jump_position() {
        assert_eq!(steps(0.0, 4, StepPosition::JumpStart), 0.25);
        assert_eq!(steps(0.0, 4, StepPosition::JumpEnd), 0.0);
        assert_eq!(steps(0.5, 4, StepPosition::JumpNone), 2.0 / 3.0);
        assert_eq!(steps(0.0, 4, StepPosition::JumpBoth), 0.2);
        assert_eq!(steps(1.0, 4, StepPosition::JumpEnd), 1.0);
    }
}
