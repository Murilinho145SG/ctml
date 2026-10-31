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
fn parses_transition_lists_and_uses_the_last_matching_opacity_entry() {
    let mut style = TextStyle::default();
    apply_transition(
        &mut style,
        "transition",
        "opacity 100ms linear, color 300ms ease-in, all 500ms ease-out 25ms",
    );
    assert!(style.transition_opacity);
    assert_eq!(style.transition_duration_ms, 500.0);
    assert_eq!(style.transition_delay_ms, 25.0);
    assert_eq!(style.transition_easing, TransitionEasing::EaseOut);

    apply_transition(
        &mut style,
        "transition",
        "opacity 100ms linear, color 300ms ease-in",
    );
    assert_eq!(style.transition_duration_ms, 100.0);
    assert_eq!(style.transition_easing, TransitionEasing::Linear);
}

#[test]
fn transition_allows_outline_color_with_its_own_list_timing() {
    let mut style = TextStyle::default();
    apply_transition(
        &mut style,
        "transition",
        "color 100ms linear, outline-color 300ms ease-in",
    );

    assert!(style.transition_outline_color);
    assert_eq!(style.transition_outline_color_duration_ms, 300.0);
    assert_eq!(
        style.transition_outline_color_easing,
        TransitionEasing::EaseIn
    );
}

#[test]
fn invalid_transition_list_does_not_replace_the_previous_value() {
    let mut style = TextStyle::default();
    apply_transition(&mut style, "transition", "opacity 200ms ease-in");
    apply_transition(&mut style, "transition", "opacity 100ms, color invalid");
    assert!(style.transition_opacity);
    assert_eq!(style.transition_duration_ms, 200.0);
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
fn transition_longhand_lists_match_the_opacity_entry_by_index() {
    let mut style = TextStyle::default();
    apply_transition(&mut style, "transition-duration", "100ms, 250ms, 400ms");
    apply_transition(&mut style, "transition-timing-function", "linear, ease-in");
    apply_transition(&mut style, "transition-delay", "-10ms, 25ms");
    apply_transition(
        &mut style,
        "transition-property",
        "color, opacity, transform",
    );
    assert!(style.transition_opacity);
    assert_eq!(style.transition_duration_ms, 250.0);
    assert_eq!(style.transition_delay_ms, 25.0);
    assert_eq!(style.transition_easing, TransitionEasing::EaseIn);
    assert!(style.transition_color);
    assert_eq!(style.transition_color_duration_ms, 100.0);
    assert_eq!(style.transition_color_delay_ms, -10.0);
    assert_eq!(style.transition_color_easing, TransitionEasing::Linear);

    apply_transition(&mut style, "transition-property", "opacity, color, opacity");
    assert_eq!(style.transition_duration_ms, 400.0);
    assert_eq!(style.transition_delay_ms, -10.0);
    assert_eq!(style.transition_easing, TransitionEasing::Linear);
    assert_eq!(style.transition_color_duration_ms, 250.0);
    assert_eq!(style.transition_color_delay_ms, 25.0);
    assert_eq!(style.transition_color_easing, TransitionEasing::EaseIn);
}

#[test]
fn other_transition_properties_do_not_enable_opacity_animation() {
    let mut style = TextStyle::default();
    apply_transition(&mut style, "transition-property", "transform");
    assert!(!style.transition_opacity);
}

#[test]
fn transition_behavior_enables_discrete_visibility_transitions() {
    let mut style = TextStyle::default();
    apply_transition(
        &mut style,
        "transition",
        "visibility 250ms linear allow-discrete",
    );
    assert_eq!(
        style.transition_visibility,
        Some(crate::style::TransitionTiming {
            duration_ms: 250.0,
            delay_ms: 0.0,
            easing: TransitionEasing::Linear,
        })
    );

    apply_transition(&mut style, "transition-property", "opacity, visibility");
    apply_transition(&mut style, "transition-duration", "100ms, 400ms");
    apply_transition(&mut style, "transition-behavior", "normal, allow-discrete");
    assert_eq!(
        style.transition_visibility,
        Some(crate::style::TransitionTiming {
            duration_ms: 400.0,
            delay_ms: 0.0,
            easing: TransitionEasing::Linear,
        })
    );

    apply_transition(&mut style, "transition-behavior", "normal");
    assert_eq!(style.transition_visibility, None);
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
