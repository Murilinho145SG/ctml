use crate::style::{
    AnimationDirection, AnimationFillMode, AnimationPlayState, TextStyle, TransitionEasing,
};

const ANIMATION_PROPERTIES: [&str; 8] = [
    "animation",
    "animation-name",
    "animation-duration",
    "animation-delay",
    "animation-iteration-count",
    "animation-direction",
    "animation-fill-mode",
    "animation-play-state",
];

pub(super) fn is_animation_property(property: &str) -> bool {
    ANIMATION_PROPERTIES.contains(&property)
        || property.eq_ignore_ascii_case("animation-timing-function")
}

pub(super) fn apply_animation(style: &mut TextStyle, property: &str, value: &str) {
    let value = value.trim();
    match property {
        "animation" => apply_shorthand(style, value),
        "animation-name" => {
            if let Some(name) = parse_name(value) {
                style.animation_name = name;
            }
        }
        "animation-duration" => {
            if let Some(duration) = super::transition::parse_time(value).filter(|time| *time >= 0.0)
            {
                style.animation_duration_ms = duration;
            }
        }
        "animation-delay" => {
            if let Some(delay) = super::transition::parse_time(value) {
                style.animation_delay_ms = delay;
            }
        }
        "animation-iteration-count" => {
            if value.eq_ignore_ascii_case("infinite") {
                style.animation_iterations = f32::INFINITY;
            } else if let Some(iterations) = ctml_css::parse_css_number(value)
                && iterations.is_finite()
                && iterations >= 0.0
            {
                style.animation_iterations = iterations;
            }
        }
        "animation-direction" => {
            if let Some(direction) = parse_direction(value) {
                style.animation_direction = direction;
            }
        }
        "animation-fill-mode" => {
            if let Some(fill_mode) = parse_fill_mode(value) {
                style.animation_fill_mode = fill_mode;
            }
        }
        "animation-play-state" => {
            if let Some(play_state) = parse_play_state(value) {
                style.animation_play_state = play_state;
            }
        }
        "animation-timing-function" => {
            if let Some(easing) = super::transition::parse_easing(value) {
                style.animation_easing = easing;
            }
        }
        _ => {}
    }
}

pub(super) fn reset_animation(style: &mut TextStyle, property: &str) {
    match property {
        "animation" => {
            style.animation_name = None;
            style.animation_duration_ms = 0.0;
            style.animation_delay_ms = 0.0;
            style.animation_iterations = 1.0;
            style.animation_direction = AnimationDirection::Normal;
            style.animation_fill_mode = AnimationFillMode::None;
            style.animation_play_state = AnimationPlayState::Running;
            style.animation_easing = TransitionEasing::Ease;
        }
        "animation-name" => style.animation_name = None,
        "animation-duration" => style.animation_duration_ms = 0.0,
        "animation-delay" => style.animation_delay_ms = 0.0,
        "animation-iteration-count" => style.animation_iterations = 1.0,
        "animation-direction" => style.animation_direction = AnimationDirection::Normal,
        "animation-fill-mode" => style.animation_fill_mode = AnimationFillMode::None,
        "animation-play-state" => style.animation_play_state = AnimationPlayState::Running,
        "animation-timing-function" => style.animation_easing = TransitionEasing::Ease,
        _ => {}
    }
}

pub(super) fn inherit_animation(style: &mut TextStyle, inherited: &TextStyle, property: &str) {
    match property {
        "animation" => {
            style.animation_name.clone_from(&inherited.animation_name);
            style.animation_duration_ms = inherited.animation_duration_ms;
            style.animation_delay_ms = inherited.animation_delay_ms;
            style.animation_iterations = inherited.animation_iterations;
            style.animation_direction = inherited.animation_direction;
            style.animation_fill_mode = inherited.animation_fill_mode;
            style.animation_play_state = inherited.animation_play_state;
            style.animation_easing = inherited.animation_easing;
        }
        "animation-name" => style.animation_name.clone_from(&inherited.animation_name),
        "animation-duration" => style.animation_duration_ms = inherited.animation_duration_ms,
        "animation-delay" => style.animation_delay_ms = inherited.animation_delay_ms,
        "animation-iteration-count" => style.animation_iterations = inherited.animation_iterations,
        "animation-direction" => style.animation_direction = inherited.animation_direction,
        "animation-fill-mode" => style.animation_fill_mode = inherited.animation_fill_mode,
        "animation-play-state" => style.animation_play_state = inherited.animation_play_state,
        "animation-timing-function" => style.animation_easing = inherited.animation_easing,
        _ => {}
    }
}

fn apply_shorthand(target: &mut TextStyle, value: &str) {
    let entries = super::transition::split_transition_entries(value);
    if entries.len() != 1 {
        return;
    }
    let mut style = target.clone();
    reset_animation(&mut style, "animation");
    let mut name = None;
    let mut time_count = 0;
    for token in super::transition::split_function_tokens(entries[0]) {
        if let Some(time) = super::transition::parse_time(&token) {
            if time_count == 0 {
                if time < 0.0 {
                    return;
                }
                style.animation_duration_ms = time;
            } else if time_count == 1 {
                style.animation_delay_ms = time;
            } else {
                return;
            }
            time_count += 1;
        } else if let Some(easing) = super::transition::parse_easing(&token) {
            style.animation_easing = easing;
        } else if token.eq_ignore_ascii_case("infinite") {
            style.animation_iterations = f32::INFINITY;
        } else if let Some(iterations) = ctml_css::parse_css_number(&token) {
            if !iterations.is_finite() || iterations < 0.0 {
                return;
            }
            style.animation_iterations = iterations;
        } else if let Some(direction) = parse_direction(&token) {
            style.animation_direction = direction;
        } else if let Some(fill_mode) = parse_fill_mode(&token) {
            style.animation_fill_mode = fill_mode;
        } else if let Some(play_state) = parse_play_state(&token) {
            style.animation_play_state = play_state;
        } else if token.eq_ignore_ascii_case("none") {
            name = None;
        } else if let Some(parsed_name) = parse_custom_name(&token) {
            if name.replace(parsed_name).is_some() {
                return;
            }
        } else {
            return;
        }
    }
    style.animation_name = name;
    target.animation_name = style.animation_name;
    target.animation_duration_ms = style.animation_duration_ms;
    target.animation_delay_ms = style.animation_delay_ms;
    target.animation_iterations = style.animation_iterations;
    target.animation_direction = style.animation_direction;
    target.animation_fill_mode = style.animation_fill_mode;
    target.animation_play_state = style.animation_play_state;
    target.animation_easing = style.animation_easing;
}

fn parse_name(value: &str) -> Option<Option<String>> {
    if value.eq_ignore_ascii_case("none") {
        return Some(None);
    }
    parse_custom_name(value).map(Some)
}

fn parse_custom_name(value: &str) -> Option<String> {
    let name = ctml_css::parse_css_string(value).unwrap_or_else(|| value.to_owned());
    let mut characters = name.chars();
    let first = characters.next()?;
    let valid_first =
        first == '_' || first == '-' || first.is_ascii_alphabetic() || !first.is_ascii();
    let valid_rest = characters.all(|character| {
        character == '_'
            || character == '-'
            || character.is_ascii_alphanumeric()
            || !character.is_ascii()
    });
    (valid_first && valid_rest && !is_reserved_custom_ident(&name)).then_some(name)
}

fn is_reserved_custom_ident(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "initial" | "inherit" | "unset" | "revert" | "revert-layer" | "default"
    )
}

fn parse_direction(value: &str) -> Option<AnimationDirection> {
    match value.to_ascii_lowercase().as_str() {
        "normal" => Some(AnimationDirection::Normal),
        "reverse" => Some(AnimationDirection::Reverse),
        "alternate" => Some(AnimationDirection::Alternate),
        "alternate-reverse" => Some(AnimationDirection::AlternateReverse),
        _ => None,
    }
}

fn parse_fill_mode(value: &str) -> Option<AnimationFillMode> {
    match value.to_ascii_lowercase().as_str() {
        "none" => Some(AnimationFillMode::None),
        "forwards" => Some(AnimationFillMode::Forwards),
        "backwards" => Some(AnimationFillMode::Backwards),
        "both" => Some(AnimationFillMode::Both),
        _ => None,
    }
}

fn parse_play_state(value: &str) -> Option<AnimationPlayState> {
    match value.to_ascii_lowercase().as_str() {
        "running" => Some(AnimationPlayState::Running),
        "paused" => Some(AnimationPlayState::Paused),
        _ => None,
    }
}

#[cfg(test)]
#[path = "animation_tests.rs"]
mod tests;
