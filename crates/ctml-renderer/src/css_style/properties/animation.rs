use crate::style::{
    AnimationComposition, AnimationDirection, AnimationFillMode, AnimationPlayState, TextStyle,
    TransitionEasing,
};

const ANIMATION_PROPERTIES: [&str; 9] = [
    "animation",
    "animation-name",
    "animation-duration",
    "animation-delay",
    "animation-iteration-count",
    "animation-direction",
    "animation-fill-mode",
    "animation-play-state",
    "animation-composition",
];

#[derive(Clone, Copy)]
struct AnimationEntry<'a> {
    name: Option<&'a str>,
    duration_ms: f32,
    delay_ms: f32,
    iterations: f32,
    direction: AnimationDirection,
    fill_mode: AnimationFillMode,
    play_state: AnimationPlayState,
    easing: TransitionEasing,
}

impl Default for AnimationEntry<'_> {
    fn default() -> Self {
        Self {
            name: None,
            duration_ms: 0.0,
            delay_ms: 0.0,
            iterations: 1.0,
            direction: AnimationDirection::Normal,
            fill_mode: AnimationFillMode::None,
            play_state: AnimationPlayState::Running,
            easing: TransitionEasing::Ease,
        }
    }
}

pub(super) fn is_animation_property(property: &str) -> bool {
    ANIMATION_PROPERTIES.contains(&property)
        || property.eq_ignore_ascii_case("animation-timing-function")
}

pub(super) fn apply_animation(style: &mut TextStyle, property: &str, value: &str) {
    match property {
        "animation" => apply_shorthand(style, value.trim()),
        "animation-name" => apply_list(style, value, parse_name, |style, values| {
            style.animation_name = values;
        }),
        "animation-duration" => apply_list(
            style,
            value,
            |entry| super::transition::parse_time(entry).filter(|time| *time >= 0.0),
            |style, values| style.animation_duration_ms = values,
        ),
        "animation-delay" => apply_list(
            style,
            value,
            super::transition::parse_time,
            |style, values| style.animation_delay_ms = values,
        ),
        "animation-iteration-count" => {
            apply_list(style, value, parse_iterations, |style, values| {
                style.animation_iterations = values;
            })
        }
        "animation-direction" => apply_list(style, value, parse_direction, |style, values| {
            style.animation_direction = values;
        }),
        "animation-fill-mode" => apply_list(style, value, parse_fill_mode, |style, values| {
            style.animation_fill_mode = values;
        }),
        "animation-play-state" => apply_list(style, value, parse_play_state, |style, values| {
            style.animation_play_state = values;
        }),
        "animation-composition" => apply_list(style, value, parse_composition, |style, values| {
            style.animation_composition = values;
        }),
        "animation-timing-function" => apply_list(
            style,
            value,
            super::transition::parse_easing,
            |style, values| style.animation_easing = values,
        ),
        _ => {}
    }
}

pub(super) fn reset_animation(style: &mut TextStyle, property: &str) {
    match property {
        "animation" => {
            style.animation_name = vec![None];
            style.animation_duration_ms = vec![0.0];
            style.animation_delay_ms = vec![0.0];
            style.animation_iterations = vec![1.0];
            style.animation_direction = vec![AnimationDirection::Normal];
            style.animation_fill_mode = vec![AnimationFillMode::None];
            style.animation_play_state = vec![AnimationPlayState::Running];
            style.animation_composition = vec![AnimationComposition::Replace];
            style.animation_easing = vec![TransitionEasing::Ease];
        }
        "animation-name" => style.animation_name = vec![None],
        "animation-duration" => style.animation_duration_ms = vec![0.0],
        "animation-delay" => style.animation_delay_ms = vec![0.0],
        "animation-iteration-count" => style.animation_iterations = vec![1.0],
        "animation-direction" => style.animation_direction = vec![AnimationDirection::Normal],
        "animation-fill-mode" => style.animation_fill_mode = vec![AnimationFillMode::None],
        "animation-play-state" => style.animation_play_state = vec![AnimationPlayState::Running],
        "animation-composition" => {
            style.animation_composition = vec![AnimationComposition::Replace]
        }
        "animation-timing-function" => style.animation_easing = vec![TransitionEasing::Ease],
        _ => {}
    }
}

pub(super) fn inherit_animation(style: &mut TextStyle, inherited: &TextStyle, property: &str) {
    match property {
        "animation" => {
            style.animation_name.clone_from(&inherited.animation_name);
            style
                .animation_duration_ms
                .clone_from(&inherited.animation_duration_ms);
            style
                .animation_delay_ms
                .clone_from(&inherited.animation_delay_ms);
            style
                .animation_iterations
                .clone_from(&inherited.animation_iterations);
            style
                .animation_direction
                .clone_from(&inherited.animation_direction);
            style
                .animation_fill_mode
                .clone_from(&inherited.animation_fill_mode);
            style
                .animation_play_state
                .clone_from(&inherited.animation_play_state);
            style
                .animation_composition
                .clone_from(&inherited.animation_composition);
            style
                .animation_easing
                .clone_from(&inherited.animation_easing);
        }
        "animation-name" => style.animation_name.clone_from(&inherited.animation_name),
        "animation-duration" => style
            .animation_duration_ms
            .clone_from(&inherited.animation_duration_ms),
        "animation-delay" => style
            .animation_delay_ms
            .clone_from(&inherited.animation_delay_ms),
        "animation-iteration-count" => style
            .animation_iterations
            .clone_from(&inherited.animation_iterations),
        "animation-direction" => style
            .animation_direction
            .clone_from(&inherited.animation_direction),
        "animation-fill-mode" => style
            .animation_fill_mode
            .clone_from(&inherited.animation_fill_mode),
        "animation-play-state" => style
            .animation_play_state
            .clone_from(&inherited.animation_play_state),
        "animation-composition" => style
            .animation_composition
            .clone_from(&inherited.animation_composition),
        "animation-timing-function" => style
            .animation_easing
            .clone_from(&inherited.animation_easing),
        _ => {}
    }
}

fn apply_list<T>(
    style: &mut TextStyle,
    value: &str,
    mut parse: impl FnMut(&str) -> Option<T>,
    mut set: impl FnMut(&mut TextStyle, Vec<T>),
) {
    let entries = super::transition::split_transition_entries(value);
    if entries.is_empty() {
        return;
    }
    let Some(values) = entries.iter().map(|entry| parse(entry.trim())).collect() else {
        return;
    };
    set(style, values);
}

fn apply_shorthand(style: &mut TextStyle, value: &str) {
    let entries = super::transition::split_transition_entries(value);
    if entries.is_empty() {
        return;
    }
    let Some(entries) = entries
        .into_iter()
        .map(parse_shorthand_entry)
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };
    style.animation_name = entries
        .iter()
        .map(|entry| entry.name.and_then(parse_custom_name))
        .collect();
    style.animation_duration_ms = entries.iter().map(|entry| entry.duration_ms).collect();
    style.animation_delay_ms = entries.iter().map(|entry| entry.delay_ms).collect();
    style.animation_iterations = entries.iter().map(|entry| entry.iterations).collect();
    style.animation_direction = entries.iter().map(|entry| entry.direction).collect();
    style.animation_fill_mode = entries.iter().map(|entry| entry.fill_mode).collect();
    style.animation_play_state = entries.iter().map(|entry| entry.play_state).collect();
    style.animation_composition = vec![AnimationComposition::Replace; entries.len()];
    style.animation_easing = entries.iter().map(|entry| entry.easing).collect();
}

fn parse_shorthand_entry(value: &str) -> Option<AnimationEntry<'_>> {
    let mut entry = AnimationEntry::default();
    let mut name_seen = false;
    let mut easing_seen = false;
    let mut iteration_seen = false;
    let mut direction_seen = false;
    let mut fill_seen = false;
    let mut play_seen = false;
    let mut time_count = 0;
    for token in super::transition::split_function_tokens(value) {
        if let Some(time) = super::transition::parse_time(token) {
            if time_count == 0 {
                if time < 0.0 {
                    return None;
                }
                entry.duration_ms = time;
            } else if time_count == 1 {
                entry.delay_ms = time;
            } else {
                return None;
            }
            time_count += 1;
        } else if let Some(easing) = super::transition::parse_easing(token) {
            if easing_seen {
                return None;
            }
            entry.easing = easing;
            easing_seen = true;
        } else if let Some(iterations) = parse_iterations(token) {
            if iteration_seen {
                return None;
            }
            entry.iterations = iterations;
            iteration_seen = true;
        } else if let Some(direction) = parse_direction(token) {
            if direction_seen {
                return None;
            }
            entry.direction = direction;
            direction_seen = true;
        } else if let Some(fill_mode) = parse_fill_mode(token) {
            if fill_seen {
                return None;
            }
            entry.fill_mode = fill_mode;
            fill_seen = true;
        } else if let Some(play_state) = parse_play_state(token) {
            if play_seen {
                return None;
            }
            entry.play_state = play_state;
            play_seen = true;
        } else if token.eq_ignore_ascii_case("none") && !name_seen {
            entry.name = Some("none");
            name_seen = true;
        } else if parse_custom_name(token).is_some() && !name_seen {
            entry.name = Some(token);
            name_seen = true;
        } else {
            return None;
        }
    }
    Some(entry)
}

fn parse_name(value: &str) -> Option<Option<String>> {
    if value.eq_ignore_ascii_case("none") {
        Some(None)
    } else {
        parse_custom_name(value).map(Some)
    }
}

fn parse_custom_name(value: &str) -> Option<String> {
    if value.eq_ignore_ascii_case("none") {
        return None;
    }
    if let Some(name) = ctml_css::parse_css_string(value) {
        return Some(name);
    }
    let name = value.to_owned();
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

fn parse_iterations(value: &str) -> Option<f32> {
    if value.eq_ignore_ascii_case("infinite") {
        return Some(f32::INFINITY);
    }
    ctml_css::parse_css_number(value).filter(|count| count.is_finite() && *count >= 0.0)
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

fn parse_composition(value: &str) -> Option<AnimationComposition> {
    match value.trim().to_ascii_lowercase().as_str() {
        "replace" => Some(AnimationComposition::Replace),
        "add" => Some(AnimationComposition::Add),
        "accumulate" => Some(AnimationComposition::Accumulate),
        _ => None,
    }
}

#[cfg(test)]
#[path = "animation_tests.rs"]
mod tests;
