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

    assert_eq!(style.animation_name[0].as_deref(), Some("fade"));
    assert_eq!(style.animation_duration_ms, [2000.0]);
    assert_eq!(style.animation_delay_ms, [250.0]);
    assert_eq!(style.animation_iterations, [3.0]);
    assert_eq!(style.animation_direction, [AnimationDirection::Alternate]);
    assert_eq!(style.animation_fill_mode, [AnimationFillMode::Both]);
    assert_eq!(style.animation_play_state, [AnimationPlayState::Paused]);
    assert_eq!(style.animation_easing, [TransitionEasing::EaseIn]);
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

    assert_eq!(style.animation_name[0].as_deref(), Some("fade"));
    assert_eq!(style.animation_duration_ms, [250.0]);
    assert_eq!(style.animation_delay_ms, [-25.0]);
    assert_eq!(style.animation_iterations, [f32::INFINITY]);
    assert_eq!(
        style.animation_direction,
        [AnimationDirection::AlternateReverse]
    );
    assert_eq!(style.animation_fill_mode, [AnimationFillMode::Backwards]);
    assert_eq!(style.animation_play_state, [AnimationPlayState::Paused]);
    assert_eq!(
        style.animation_easing,
        [TransitionEasing::Steps {
            count: 4,
            position: StepPosition::JumpStart,
        }]
    );
}

#[test]
fn invalid_animation_shorthand_does_not_reset_previous_values() {
    let mut style = TextStyle::default();
    apply_animation(&mut style, "animation", "fade 2s");

    apply_animation(&mut style, "animation", "other -1s");

    assert_eq!(style.animation_name, [Some("fade".to_owned())]);
    assert_eq!(style.animation_duration_ms, [2000.0]);
}

#[test]
fn animation_lists_pair_by_index_and_repeat_shorter_longhand_lists() {
    let mut style = TextStyle::default();
    apply_animation(&mut style, "animation-name", "fade, pulse, none");
    apply_animation(&mut style, "animation-duration", "1s, 2s");
    apply_animation(&mut style, "animation-iteration-count", "2, infinite");
    apply_animation(&mut style, "animation-direction", "alternate");

    let specs = style.animation_specs();
    assert_eq!(specs.len(), 3);
    assert_eq!(specs[0].as_ref().unwrap().name, "fade");
    assert_eq!(specs[0].as_ref().unwrap().duration_ms, 1000.0);
    assert_eq!(specs[0].as_ref().unwrap().iterations, 2.0);
    assert_eq!(
        specs[0].as_ref().unwrap().direction,
        AnimationDirection::Alternate
    );
    assert_eq!(specs[1].as_ref().unwrap().name, "pulse");
    assert_eq!(specs[1].as_ref().unwrap().duration_ms, 2000.0);
    assert_eq!(specs[1].as_ref().unwrap().iterations, f32::INFINITY);
    assert_eq!(
        specs[1].as_ref().unwrap().direction,
        AnimationDirection::Alternate
    );
    assert!(specs[2].is_none());
}

#[test]
fn animation_shorthand_parses_multiple_entries_transactionally() {
    let mut style = TextStyle::default();
    apply_animation(
        &mut style,
        "animation",
        "fade 1s linear, pulse 2s ease-in 250ms 3 alternate both paused",
    );

    assert_eq!(
        style.animation_name,
        [Some("fade".to_owned()), Some("pulse".to_owned())]
    );
    assert_eq!(style.animation_duration_ms, [1000.0, 2000.0]);
    assert_eq!(style.animation_delay_ms, [0.0, 250.0]);
    assert_eq!(style.animation_iterations, [1.0, 3.0]);
    assert_eq!(
        style.animation_direction,
        [AnimationDirection::Normal, AnimationDirection::Alternate]
    );
    assert_eq!(
        style.animation_fill_mode,
        [AnimationFillMode::None, AnimationFillMode::Both]
    );
    assert_eq!(
        style.animation_play_state,
        [AnimationPlayState::Running, AnimationPlayState::Paused]
    );
    assert_eq!(
        style.animation_easing,
        [TransitionEasing::Linear, TransitionEasing::EaseIn]
    );

    apply_animation(&mut style, "animation", "fade 2s, broken -1s");
    assert_eq!(
        style.animation_name,
        [Some("fade".to_owned()), Some("pulse".to_owned())]
    );
    assert_eq!(style.animation_duration_ms, [1000.0, 2000.0]);
}
