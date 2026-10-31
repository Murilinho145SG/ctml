use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::style::{
    AnimationDirection, AnimationFillMode, AnimationPlayState, AnimationSpec,
    ElementTransitionStyle, TransitionEasing,
};

use super::{PageRasterizer, ease_transition};

#[derive(Clone)]
pub(in crate::page) struct KeyframeAnimationState {
    base_opacity: f32,
    base_visibility_hidden: bool,
    opacity_frames: Option<Vec<(f32, f32)>>,
    visibility_frames: Option<Vec<(f32, bool)>>,
    pub(in crate::page) starts_at: Instant,
    animation: AnimationSpec,
    duration: Duration,
    iterations: f32,
    direction: AnimationDirection,
    fill_mode: AnimationFillMode,
    play_state: AnimationPlayState,
    easing: TransitionEasing,
}

pub(in crate::page) fn build_states(
    styles: &HashMap<usize, ElementTransitionStyle>,
    keyframes: &[ctml_css::CssKeyframes],
    opacity_groups: &HashMap<usize, f32>,
    root_opacity: f32,
    now: Instant,
    previous: &HashMap<usize, KeyframeAnimationState>,
) -> HashMap<usize, KeyframeAnimationState> {
    styles
        .iter()
        .filter_map(|(node_id, style)| {
            let animation = style.animation.as_ref()?;
            let rule = keyframes
                .iter()
                .rev()
                .find(|rule| rule.name == animation.name)?;
            let base_opacity = if *node_id == 0 {
                root_opacity
            } else {
                *opacity_groups.get(node_id)?
            };
            let base_visibility_hidden = style.visibility_hidden;
            let opacity_frames = collect_opacity_frames(rule, base_opacity);
            let visibility_frames = collect_visibility_frames(rule, base_visibility_hidden);
            if opacity_frames.is_none() && visibility_frames.is_none() {
                return None;
            }
            let mut state = KeyframeAnimationState {
                base_opacity,
                base_visibility_hidden,
                opacity_frames,
                visibility_frames,
                starts_at: animation_start(animation, now),
                animation: animation.clone(),
                duration: duration_from_millis(animation.duration_ms),
                iterations: animation.iterations,
                direction: animation.direction,
                fill_mode: animation.fill_mode,
                play_state: animation.play_state,
                easing: animation.easing,
            };
            if let Some(previous) = previous
                .get(node_id)
                .filter(|previous| previous.animation == *animation)
            {
                state.starts_at = previous.starts_at;
            }
            Some((*node_id, state))
        })
        .collect()
}

pub(super) fn advance(page: &mut PageRasterizer, now: Instant) -> bool {
    let mut finished = Vec::new();
    let mut active = false;
    for (node_id, state) in page
        .keyframe_animations
        .iter()
        .map(|(node_id, state)| (*node_id, state.clone()))
        .collect::<Vec<_>>()
    {
        let elapsed = now.saturating_duration_since(state.starts_at);
        let before_start = now < state.starts_at;
        let total_duration = state.duration.as_secs_f64() * f64::from(state.iterations);
        let elapsed_seconds = elapsed.as_secs_f64();
        let total_seconds = total_duration;
        let infinite = state.iterations.is_infinite();
        let is_finished = !before_start
            && !infinite
            && (state.duration.is_zero() || elapsed_seconds >= total_seconds);

        let progress = if before_start || state.play_state == AnimationPlayState::Paused {
            0.0
        } else if state.duration.is_zero() {
            1.0
        } else if is_finished {
            1.0
        } else {
            let position = elapsed_seconds / state.duration.as_secs_f64();
            (position - position.floor()) as f32
        };
        let iteration = if state.duration.is_zero() || before_start {
            0u64
        } else if is_finished {
            state.iterations.ceil().max(1.0) as u64 - 1
        } else {
            (elapsed_seconds / state.duration.as_secs_f64()).floor() as u64
        };
        let directed_progress = direct_progress(progress, iteration, state.direction);
        let has_backwards_fill = matches!(
            state.fill_mode,
            AnimationFillMode::Backwards | AnimationFillMode::Both
        );
        let has_forwards_fill = matches!(
            state.fill_mode,
            AnimationFillMode::Forwards | AnimationFillMode::Both
        );
        let effect_applies = if before_start {
            has_backwards_fill
        } else if is_finished {
            has_forwards_fill
        } else {
            true
        };
        if let Some(frames) = &state.opacity_frames {
            let opacity = if effect_applies {
                sample_opacity(frames, directed_progress, state.easing)
            } else {
                state.base_opacity
            };
            apply_opacity(page, node_id, opacity);
        }
        if let Some(frames) = &state.visibility_frames {
            let hidden = if effect_applies {
                sample_visibility(frames, directed_progress, state.easing)
            } else {
                state.base_visibility_hidden
            };
            super::visibility::apply_visibility(page, node_id, hidden);
            page.element_visibility.insert(node_id, hidden);
        }

        if is_finished {
            if !has_forwards_fill {
                finished.push(node_id);
            }
        } else if state.play_state == AnimationPlayState::Running {
            active = true;
        }
    }
    for node_id in finished {
        page.keyframe_animations.remove(&node_id);
    }
    active
}

fn collect_opacity_frames(
    rule: &ctml_css::CssKeyframes,
    base_opacity: f32,
) -> Option<Vec<(f32, f32)>> {
    let mut frames = rule
        .frames
        .iter()
        .filter_map(|frame| {
            let value = frame
                .declarations
                .iter()
                .rev()
                .find(|declaration| declaration.property == "opacity")
                .and_then(|declaration| {
                    ctml_css::parse_css_number(&declaration.value).or_else(|| {
                        ctml_css::parse_css_percentage(&declaration.value)
                            .map(|percentage| percentage / 100.0)
                    })
                })?
                .clamp(0.0, 1.0);
            Some(frame.offsets.iter().map(move |offset| (*offset, value)))
        })
        .flatten()
        .collect::<Vec<_>>();
    if frames.is_empty() {
        return None;
    }
    frames.sort_by(|left, right| left.0.total_cmp(&right.0));
    let mut unique = Vec::<(f32, f32)>::new();
    for frame in frames {
        if unique.last().is_some_and(|last| last.0 == frame.0) {
            *unique.last_mut().expect("last frame exists") = frame;
        } else {
            unique.push(frame);
        }
    }
    if unique[0].0 > 0.0 {
        unique.insert(0, (0.0, base_opacity));
    }
    if unique.last().is_some_and(|last| last.0 < 1.0) {
        unique.push((1.0, base_opacity));
    }
    Some(unique)
}

fn collect_visibility_frames(
    rule: &ctml_css::CssKeyframes,
    base_hidden: bool,
) -> Option<Vec<(f32, bool)>> {
    let mut frames = rule
        .frames
        .iter()
        .filter_map(|frame| {
            let hidden = frame
                .declarations
                .iter()
                .rev()
                .find(|declaration| declaration.property == "visibility")
                .and_then(|declaration| {
                    match declaration.value.trim().to_ascii_lowercase().as_str() {
                        "visible" => Some(false),
                        "hidden" | "collapse" => Some(true),
                        _ => None,
                    }
                })?;
            Some(frame.offsets.iter().map(move |offset| (*offset, hidden)))
        })
        .flatten()
        .collect::<Vec<_>>();
    if frames.is_empty() {
        return None;
    }
    frames.sort_by(|left, right| left.0.total_cmp(&right.0));
    let mut unique = Vec::<(f32, bool)>::new();
    for frame in frames {
        if unique.last().is_some_and(|last| last.0 == frame.0) {
            *unique.last_mut().expect("last frame exists") = frame;
        } else {
            unique.push(frame);
        }
    }
    if unique[0].0 > 0.0 {
        unique.insert(0, (0.0, base_hidden));
    }
    if unique.last().is_some_and(|last| last.0 < 1.0) {
        unique.push((1.0, base_hidden));
    }
    Some(unique)
}

fn sample_opacity(frames: &[(f32, f32)], progress: f32, easing: TransitionEasing) -> f32 {
    if progress <= frames[0].0 {
        return frames[0].1;
    }
    for pair in frames.windows(2) {
        let [(start_offset, start_opacity), (end_offset, end_opacity)] = pair else {
            continue;
        };
        if progress <= *end_offset {
            let segment = (progress - start_offset) / (end_offset - start_offset).max(f32::EPSILON);
            let segment = ease_transition(segment, easing);
            return start_opacity + (end_opacity - start_opacity) * segment;
        }
    }
    frames.last().expect("animation has keyframe values").1
}

fn sample_visibility(frames: &[(f32, bool)], progress: f32, easing: TransitionEasing) -> bool {
    if progress <= frames[0].0 {
        return frames[0].1;
    }
    for pair in frames.windows(2) {
        let [(start_offset, start_hidden), (end_offset, end_hidden)] = pair else {
            continue;
        };
        if progress <= *end_offset {
            let segment = (progress - start_offset) / (end_offset - start_offset).max(f32::EPSILON);
            return if ease_transition(segment, easing) < 0.5 {
                *start_hidden
            } else {
                *end_hidden
            };
        }
    }
    frames.last().expect("animation has visibility values").1
}

fn direct_progress(progress: f32, iteration: u64, direction: AnimationDirection) -> f32 {
    let reverse = match direction {
        AnimationDirection::Normal => false,
        AnimationDirection::Reverse => true,
        AnimationDirection::Alternate => iteration % 2 == 1,
        AnimationDirection::AlternateReverse => iteration % 2 == 0,
    };
    if reverse { 1.0 - progress } else { progress }
}

fn apply_opacity(page: &mut PageRasterizer, node_id: usize, opacity: f32) {
    if node_id == 0 {
        page.root_opacity = opacity;
    } else if let Some(group) = page.opacity_groups.get_mut(&node_id) {
        *group = opacity;
    }
}

fn animation_start(animation: &AnimationSpec, now: Instant) -> Instant {
    let delay = duration_from_millis(animation.delay_ms.abs());
    if animation.delay_ms < 0.0 {
        now.checked_sub(delay).unwrap_or(now)
    } else {
        now.checked_add(delay).unwrap_or(now)
    }
}

fn duration_from_millis(milliseconds: f32) -> Duration {
    let seconds = f64::from(milliseconds.max(0.0)) / 1000.0;
    Duration::from_secs_f64(seconds.min(315_360_000.0))
}
