use crate::style::{TextStyle, TransitionEasing};

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
            if let Some(easing) = value.split(',').next().and_then(parse_easing) {
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
    for token in value.split_ascii_whitespace() {
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
    Some(match value {
        "linear" => TransitionEasing::Linear,
        "ease" => TransitionEasing::Ease,
        "ease-in" => TransitionEasing::EaseIn,
        "ease-out" => TransitionEasing::EaseOut,
        "ease-in-out" => TransitionEasing::EaseInOut,
        _ => return None,
    })
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
}
