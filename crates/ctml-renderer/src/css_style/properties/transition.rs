use crate::style::{StepPosition, TextStyle, TransitionEasing};

pub(super) fn apply_transition(style: &mut TextStyle, property: &str, value: &str) {
    match property {
        "transition-property" => {
            let Some(property) = value.split(',').next().map(str::trim) else {
                return;
            };
            style.transition_opacity =
                matches!(property.to_ascii_lowercase().as_str(), "opacity" | "all");
        }
        "transition-duration" => {
            if let Some(duration) = value.split(',').next().and_then(parse_nonnegative_time) {
                style.transition_duration_ms = duration;
            }
        }
        "transition-delay" => {
            if let Some(delay) = value.split(',').next().and_then(parse_time) {
                style.transition_delay_ms = delay;
            }
        }
        "transition-timing-function" => {
            if let Some(easing) = first_top_level_item(value).and_then(parse_easing) {
                style.transition_easing = easing;
            }
        }
        "transition" => apply_shorthand(style, value),
        _ => {}
    }
}

fn apply_shorthand(style: &mut TextStyle, value: &str) {
    let mut property = None::<String>;
    let mut duration = None;
    let mut delay = None;
    let mut easing = None;
    for token in split_function_tokens(value) {
        let keyword = token.to_ascii_lowercase();
        if let Some(time) = parse_time(token) {
            if time < 0.0 {
                if delay.replace(time).is_some() {
                    return;
                }
            } else if duration.is_none() {
                duration = Some(time);
            } else if delay.is_none() {
                delay = Some(time);
            } else {
                return;
            }
        } else if let Some(value) = parse_easing(&keyword) {
            if easing.replace(value).is_some() {
                return;
            }
        } else if matches!(keyword.as_str(), "opacity" | "all" | "none") {
            if property.replace(keyword).is_some() {
                return;
            }
        } else {
            return;
        }
    }
    style.transition_opacity = matches!(property.as_deref().unwrap_or("all"), "opacity" | "all");
    style.transition_duration_ms = duration.unwrap_or_default();
    style.transition_delay_ms = delay.unwrap_or_default();
    style.transition_easing = easing.unwrap_or_default();
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

fn first_top_level_item(value: &str) -> Option<&str> {
    let mut depth = 0usize;
    let end = value
        .char_indices()
        .find_map(|(index, ch)| match ch {
            '(' => {
                depth += 1;
                None
            }
            ')' => {
                depth = depth.saturating_sub(1);
                None
            }
            ',' if depth == 0 => Some(index),
            _ => None,
        })
        .unwrap_or(value.len());
    let item = value[..end].trim();
    (!item.is_empty()).then_some(item)
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
mod tests {
    use super::*;

    #[test]
    fn parses_opacity_transition_shorthand() {
        let mut style = TextStyle::default();
        apply_transition(&mut style, "transition", "opacity 250ms ease-in 50ms");
        assert!(style.transition_opacity);
        assert_eq!(style.transition_duration_ms, 250.0);
        assert_eq!(style.transition_delay_ms, 50.0);
        assert_eq!(style.transition_easing, TransitionEasing::EaseIn);
    }

    #[test]
    fn parses_transition_longhands_and_rejects_invalid_values() {
        let mut style = TextStyle::default();
        apply_transition(&mut style, "transition-property", "opacity, color");
        apply_transition(&mut style, "transition-duration", "0.2s");
        apply_transition(&mut style, "transition-delay", "-25ms");
        apply_transition(&mut style, "transition-timing-function", "ease-out");
        assert!(style.transition_opacity);
        assert_eq!(style.transition_duration_ms, 200.0);
        assert_eq!(style.transition_delay_ms, -25.0);
        assert_eq!(style.transition_easing, TransitionEasing::EaseOut);

        apply_transition(&mut style, "transition-duration", "-1ms");
        assert_eq!(style.transition_duration_ms, 200.0);
    }

    #[test]
    fn other_transition_properties_do_not_enable_opacity_animation() {
        let mut style = TextStyle::default();
        apply_transition(&mut style, "transition-property", "transform");
        assert!(!style.transition_opacity);
    }

    #[test]
    fn transition_keywords_are_ascii_case_insensitive() {
        let mut style = TextStyle::default();
        apply_transition(&mut style, "transition", "OPACITY 0.25S EASE-IN");
        assert!(style.transition_opacity);
        assert_eq!(style.transition_duration_ms, 250.0);
        assert_eq!(style.transition_easing, TransitionEasing::EaseIn);
    }

    #[test]
    fn parses_custom_cubic_bezier_with_internal_whitespace() {
        let mut style = TextStyle::default();
        apply_transition(
            &mut style,
            "transition",
            "opacity 250ms cubic-bezier(0.1, 0.2, 0.3, 0.9)",
        );
        assert_eq!(
            style.transition_easing,
            TransitionEasing::CubicBezier([
                0.1f32.to_bits(),
                0.2f32.to_bits(),
                0.3f32.to_bits(),
                0.9f32.to_bits()
            ])
        );
        apply_transition(
            &mut style,
            "transition-timing-function",
            "cubic-bezier(0, 0.4, 1, 0.8), ease",
        );
        assert_eq!(
            style.transition_easing,
            TransitionEasing::CubicBezier([
                0.0f32.to_bits(),
                0.4f32.to_bits(),
                1.0f32.to_bits(),
                0.8f32.to_bits()
            ])
        );
    }

    #[test]
    fn parses_step_easing_and_rejects_invalid_curves() {
        let mut style = TextStyle::default();
        apply_transition(
            &mut style,
            "transition-timing-function",
            "steps(4, jump-start)",
        );
        assert_eq!(
            style.transition_easing,
            TransitionEasing::Steps {
                count: 4,
                position: StepPosition::JumpStart
            }
        );
        apply_transition(&mut style, "transition", "opacity steps(3, jump-none)");
        assert_eq!(
            style.transition_easing,
            TransitionEasing::Steps {
                count: 3,
                position: StepPosition::JumpNone
            }
        );
        let previous = style.transition_easing;
        apply_transition(
            &mut style,
            "transition-timing-function",
            "cubic-bezier(2, 0, 0, 1)",
        );
        assert_eq!(style.transition_easing, previous);
        apply_transition(
            &mut style,
            "transition-timing-function",
            "steps(1, jump-none)",
        );
        assert_eq!(style.transition_easing, previous);
    }
}
