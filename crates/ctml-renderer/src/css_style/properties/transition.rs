use crate::style::{StepPosition, TextStyle, TransitionEasing};

pub(super) fn apply_transition(style: &mut TextStyle, property: &str, value: &str) {
    match property {
        "transition-property" => {
            let Some(properties) = parse_transition_properties(value) else {
                return;
            };
            style.transition_properties = properties;
            update_opacity_transition(style);
        }
        "transition-duration" => {
            if let Some(durations) = parse_time_list(value, parse_nonnegative_time) {
                style.transition_durations_ms = durations;
                update_opacity_transition(style);
            }
        }
        "transition-delay" => {
            if let Some(delays) = parse_time_list(value, parse_time) {
                style.transition_delays_ms = delays;
                update_opacity_transition(style);
            }
        }
        "transition-timing-function" => {
            if let Some(easings) = split_transition_entries(value)
                .into_iter()
                .map(parse_easing)
                .collect::<Option<Vec<_>>>()
            {
                style.transition_easings = easings;
                update_opacity_transition(style);
            }
        }
        "transition-behavior" => {
            if let Some(behaviors) = parse_transition_behaviors(value) {
                style.transition_behaviors = behaviors;
                update_opacity_transition(style);
            }
        }
        "transition" => apply_shorthand(style, value),
        _ => {}
    }
}

fn apply_shorthand(style: &mut TextStyle, value: &str) {
    let entries = split_transition_entries(value)
        .into_iter()
        .map(parse_transition_entry)
        .collect::<Option<Vec<_>>>();
    let Some(entries) = entries else {
        return;
    };
    if entries.is_empty() {
        return;
    }
    if entries.iter().any(|entry| entry.0 == "none") {
        if entries.len() != 1 {
            return;
        }
        style.transition_properties = vec!["none".to_owned()];
        style.transition_durations_ms = vec![0.0];
        style.transition_delays_ms = vec![0.0];
        style.transition_easings = vec![TransitionEasing::default()];
        style.transition_behaviors = vec![false];
        update_opacity_transition(style);
        return;
    }
    let selected = entries
        .iter()
        .map(|entry| entry.0.clone())
        .collect::<Vec<_>>();
    style.transition_properties = selected;
    style.transition_durations_ms = entries.iter().map(|entry| entry.1).collect();
    style.transition_delays_ms = entries.iter().map(|entry| entry.2).collect();
    style.transition_easings = entries.iter().map(|entry| entry.3).collect();
    style.transition_behaviors = entries.iter().map(|entry| entry.4).collect();
    update_opacity_transition(style);
}

pub(super) fn reset_transition(style: &mut TextStyle, property: &str) {
    match property {
        "transition" => {
            style.transition_properties = vec!["all".to_owned()];
            style.transition_durations_ms = vec![0.0];
            style.transition_delays_ms = vec![0.0];
            style.transition_easings = vec![TransitionEasing::default()];
            style.transition_behaviors = vec![false];
        }
        "transition-property" => style.transition_properties = vec!["all".to_owned()],
        "transition-duration" => style.transition_durations_ms = vec![0.0],
        "transition-delay" => style.transition_delays_ms = vec![0.0],
        "transition-timing-function" => {
            style.transition_easings = vec![TransitionEasing::default()]
        }
        "transition-behavior" => style.transition_behaviors = vec![false],
        _ => return,
    }
    update_opacity_transition(style);
}

pub(super) fn update_opacity_transition(style: &mut TextStyle) {
    if let Some(index) = style
        .transition_properties
        .iter()
        .rposition(|property| matches!(property.as_str(), "opacity" | "all"))
    {
        style.transition_opacity = true;
        style.transition_duration_ms = list_item(&style.transition_durations_ms, index);
        style.transition_delay_ms = list_item(&style.transition_delays_ms, index);
        style.transition_easing = list_item(&style.transition_easings, index);
    } else {
        style.transition_opacity = false;
    }
    if let Some(index) = style
        .transition_properties
        .iter()
        .rposition(|property| matches!(property.as_str(), "color" | "all"))
    {
        style.transition_color = true;
        style.transition_color_duration_ms = list_item(&style.transition_durations_ms, index);
        style.transition_color_delay_ms = list_item(&style.transition_delays_ms, index);
        style.transition_color_easing = list_item(&style.transition_easings, index);
    } else {
        style.transition_color = false;
    }
    if let Some(index) = style
        .transition_properties
        .iter()
        .rposition(|property| matches!(property.as_str(), "background-color" | "all"))
    {
        style.transition_background_color = true;
        style.transition_background_color_duration_ms =
            list_item(&style.transition_durations_ms, index);
        style.transition_background_color_delay_ms = list_item(&style.transition_delays_ms, index);
        style.transition_background_color_easing = list_item(&style.transition_easings, index);
    } else {
        style.transition_background_color = false;
    }
    if let Some(index) = style
        .transition_properties
        .iter()
        .rposition(|property| matches!(property.as_str(), "outline-color" | "all"))
    {
        style.transition_outline_color = true;
        style.transition_outline_color_duration_ms =
            list_item(&style.transition_durations_ms, index);
        style.transition_outline_color_delay_ms = list_item(&style.transition_delays_ms, index);
        style.transition_outline_color_easing = list_item(&style.transition_easings, index);
    } else {
        style.transition_outline_color = false;
    }
    if let Some(index) = style
        .transition_properties
        .iter()
        .rposition(|property| matches!(property.as_str(), "border-color" | "all"))
    {
        style.transition_border_color = true;
        style.transition_border_color_duration_ms =
            list_item(&style.transition_durations_ms, index);
        style.transition_border_color_delay_ms = list_item(&style.transition_delays_ms, index);
        style.transition_border_color_easing = list_item(&style.transition_easings, index);
    } else {
        style.transition_border_color = false;
    }
    if let Some(index) = style
        .transition_properties
        .iter()
        .rposition(|property| matches!(property.as_str(), "visibility" | "all"))
        && list_item(&style.transition_behaviors, index)
    {
        style.transition_visibility = Some(crate::style::TransitionTiming {
            duration_ms: list_item(&style.transition_durations_ms, index),
            delay_ms: list_item(&style.transition_delays_ms, index),
            easing: list_item(&style.transition_easings, index),
        });
    } else {
        style.transition_visibility = None;
    }
}

fn list_item<T: Copy>(values: &[T], index: usize) -> T {
    values[index % values.len()]
}

fn parse_transition_properties(value: &str) -> Option<Vec<String>> {
    let properties = split_transition_entries(value)
        .into_iter()
        .map(|property| {
            let property = property.to_ascii_lowercase();
            is_transition_property(&property).then_some(property)
        })
        .collect::<Option<Vec<_>>>()?;
    if properties.is_empty() || (properties.contains(&"none".to_owned()) && properties.len() != 1) {
        return None;
    }
    Some(properties)
}

fn parse_time_list(value: &str, parse: fn(&str) -> Option<f32>) -> Option<Vec<f32>> {
    split_transition_entries(value)
        .into_iter()
        .map(parse)
        .collect()
}

fn parse_transition_behaviors(value: &str) -> Option<Vec<bool>> {
    split_transition_entries(value)
        .into_iter()
        .map(|behavior| match behavior.to_ascii_lowercase().as_str() {
            "normal" => Some(false),
            "allow-discrete" => Some(true),
            _ => None,
        })
        .collect()
}

fn parse_transition_entry(value: &str) -> Option<(String, f32, f32, TransitionEasing, bool)> {
    let mut property = None::<String>;
    let mut duration = None;
    let mut delay = None;
    let mut easing = None;
    let mut behavior = None;
    let tokens = split_function_tokens(value);
    if tokens.is_empty() {
        return None;
    }
    for token in &tokens {
        let keyword = token.to_ascii_lowercase();
        if let Some(time) = parse_time(token) {
            if time < 0.0 {
                if delay.replace(time).is_some() {
                    return None;
                }
            } else if duration.is_none() {
                duration = Some(time);
            } else if delay.is_none() {
                delay = Some(time);
            } else {
                return None;
            }
        } else if let Some(value) = parse_easing(&keyword) {
            if easing.replace(value).is_some() {
                return None;
            }
        } else if matches!(keyword.as_str(), "normal" | "allow-discrete") {
            if behavior.replace(keyword == "allow-discrete").is_some() {
                return None;
            }
        } else if is_transition_property(&keyword) {
            if property.replace(keyword).is_some() {
                return None;
            }
        } else {
            return None;
        }
    }
    if property.as_deref() == Some("none")
        && (duration.is_some() || delay.is_some() || easing.is_some() || behavior.is_some())
    {
        return None;
    }
    if property.as_deref() == Some("none") && tokens.len() != 1 {
        return None;
    }
    Some((
        property.unwrap_or_else(|| "all".to_owned()),
        duration.unwrap_or_default(),
        delay.unwrap_or_default(),
        easing.unwrap_or_default(),
        behavior.unwrap_or_default(),
    ))
}

fn is_transition_property(value: &str) -> bool {
    matches!(value, "opacity" | "all" | "none")
        || value.strip_prefix("--").is_some_and(is_valid_property_name)
        || is_valid_property_name(value)
}

fn is_valid_property_name(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

fn split_transition_entries(value: &str) -> Vec<&str> {
    let mut entries = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                entries.push(value[start..index].trim());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    entries.push(value[start..].trim());
    entries
}

fn parse_time(value: &str) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    let milliseconds = value
        .strip_suffix("ms")
        .and_then(|number| number.trim().parse::<f32>().ok())
        .or_else(|| {
            value
                .strip_suffix('s')
                .and_then(|number| number.trim().parse::<f32>().ok())
                .map(|seconds| seconds * 1000.0)
        })?;
    milliseconds.is_finite().then_some(milliseconds)
}

fn parse_nonnegative_time(value: &str) -> Option<f32> {
    parse_time(value).filter(|duration| *duration >= 0.0)
}

fn parse_easing(value: &str) -> Option<TransitionEasing> {
    let value = value.trim().to_ascii_lowercase();
    Some(match value.as_str() {
        "linear" => TransitionEasing::Linear,
        "ease" => TransitionEasing::Ease,
        "ease-in" => TransitionEasing::EaseIn,
        "ease-out" => TransitionEasing::EaseOut,
        "ease-in-out" => TransitionEasing::EaseInOut,
        "step-start" => TransitionEasing::Steps {
            count: 1,
            position: StepPosition::JumpStart,
        },
        "step-end" => TransitionEasing::Steps {
            count: 1,
            position: StepPosition::JumpEnd,
        },
        _ if value.starts_with("cubic-bezier(") && value.ends_with(')') => {
            let args = &value["cubic-bezier(".len()..value.len() - 1];
            let values = args
                .split(',')
                .map(|part| part.trim().parse::<f32>().ok())
                .collect::<Option<Vec<_>>>()?;
            if values.len() != 4
                || values.iter().any(|v| !v.is_finite())
                || !(0.0..=1.0).contains(&values[0])
                || !(0.0..=1.0).contains(&values[2])
            {
                return None;
            }
            TransitionEasing::CubicBezier(
                values
                    .into_iter()
                    .map(f32::to_bits)
                    .collect::<Vec<_>>()
                    .try_into()
                    .ok()?,
            )
        }
        _ if value.starts_with("steps(") && value.ends_with(')') => {
            let args = &value["steps(".len()..value.len() - 1];
            let mut parts = args.split(',').map(str::trim);
            let count = parts.next()?.parse::<u32>().ok()?;
            if count == 0 {
                return None;
            }
            let position = match parts.next().unwrap_or("jump-end") {
                "start" | "jump-start" => StepPosition::JumpStart,
                "end" | "jump-end" => StepPosition::JumpEnd,
                "jump-none" => StepPosition::JumpNone,
                "jump-both" => StepPosition::JumpBoth,
                _ => return None,
            };
            if parts.next().is_some() || (position == StepPosition::JumpNone && count < 2) {
                return None;
            }
            TransitionEasing::Steps { count, position }
        }
        _ => return None,
    })
}

fn split_function_tokens(value: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (index, ch) in value.char_indices() {
        if ch.is_ascii_whitespace() && depth == 0 {
            if let Some(begin) = start.take() {
                tokens.push(&value[begin..index]);
            }
        } else {
            start.get_or_insert(index);
            match ch {
                '(' => depth += 1,
                ')' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
    }
    if let Some(begin) = start {
        tokens.push(&value[begin..]);
    }
    tokens
}

#[cfg(test)]
#[path = "transition_tests.rs"]
mod tests;
