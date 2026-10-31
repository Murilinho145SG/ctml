use super::*;
use crate::style::StepPosition;

#[test]
fn parses_animation_shorthand_components_in_any_order() {
    let mut style = TextStyle::default();
    apply_animation(
        &mut style,
        "animation",
        "fade 2s ease-in 250ms 3 alternate both paused",
    );

    assert_eq!(style.animation_name.as_deref(), Some("fade"));
    assert_eq!(style.animation_duration_ms, 2000.0);
    assert_eq!(style.animation_delay_ms, 250.0);
    assert_eq!(style.animation_iterations, 3.0);
    assert_eq!(style.animation_direction, AnimationDirection::Alternate);
    assert_eq!(style.animation_fill_mode, AnimationFillMode::Both);
    assert_eq!(style.animation_play_state, AnimationPlayState::Paused);
    assert_eq!(style.animation_easing, TransitionEasing::EaseIn);
}

#[test]
fn animation_longhands_parse_values_and_ignore_invalid_replacements() {
    let mut style = TextStyle::default();
    apply_animation(&mut style, "animation-name", "fade");
    apply_animation(&mut style, "animation-duration", "250ms");
    apply_animation(&mut style, "animation-delay", "-25ms");
    apply_animation(&mut style, "animation-iteration-count", "infinite");
    apply_animation(&mut style, "animation-direction", "alternate-reverse");
    apply_animation(&mut style, "animation-fill-mode", "backwards");
    apply_animation(&mut style, "animation-play-state", "paused");
    apply_animation(
        &mut style,
        "animation-timing-function",
        "steps(4, jump-start)",
    );

    apply_animation(&mut style, "animation-duration", "-1ms");
    apply_animation(&mut style, "animation-direction", "sideways");

    assert_eq!(style.animation_name.as_deref(), Some("fade"));
    assert_eq!(style.animation_duration_ms, 250.0);
    assert_eq!(style.animation_delay_ms, -25.0);
    assert_eq!(style.animation_iterations, f32::INFINITY);
    assert_eq!(
        style.animation_direction,
        AnimationDirection::AlternateReverse
    );
    assert_eq!(style.animation_fill_mode, AnimationFillMode::Backwards);
    assert_eq!(style.animation_play_state, AnimationPlayState::Paused);
    assert_eq!(
        style.animation_easing,
        TransitionEasing::Steps {
            count: 4,
            position: StepPosition::JumpStart,
        }
    );
}

#[test]
fn invalid_animation_shorthand_does_not_reset_previous_values() {
    let mut style = TextStyle::default();
    apply_animation(&mut style, "animation", "fade 2s");

    apply_animation(&mut style, "animation", "other -1s");

    assert_eq!(style.animation_name.as_deref(), Some("fade"));
    assert_eq!(style.animation_duration_ms, 2000.0);
}
