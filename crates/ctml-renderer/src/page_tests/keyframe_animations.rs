use std::time::Duration;

use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn opacity_keyframes_sample_and_apply_fill_modes() {
    let mut page = animated_page("fade 1s linear forwards", 0.8);
    let starts_at = page.keyframe_animation_start(1).unwrap();

    assert_eq!(
        page.advance_animations(starts_at + Duration::from_millis(500)),
        Some(true)
    );
    assert!((page.opacity_groups[&1] - 0.5).abs() < 0.01);
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_secs(1)),
        Some(false)
    );
    assert_eq!(page.opacity_groups[&1], 1.0);

    let mut no_fill = animated_page("fade 1s linear", 0.8);
    let starts_at = no_fill.keyframe_animation_start(1).unwrap();
    assert_eq!(
        no_fill.advance_animations(starts_at + Duration::from_secs(1)),
        Some(false)
    );
    assert_eq!(no_fill.opacity_groups[&1], 0.8);
}

#[test]
fn alternate_direction_reverses_the_second_iteration() {
    let mut page = animated_page("fade 1s linear 2 alternate", 1.0);
    let starts_at = page.keyframe_animation_start(1).unwrap();

    assert_eq!(
        page.advance_animations(starts_at + Duration::from_millis(1250)),
        Some(true)
    );
    assert!((page.opacity_groups[&1] - 0.75).abs() < 0.01);
}

#[test]
fn backwards_fill_applies_the_first_keyframe_during_a_positive_delay() {
    let mut page = animated_page("fade 1s linear 500ms both", 0.8);
    let starts_at = page.keyframe_animation_start(1).unwrap();

    assert_eq!(
        page.advance_animations(starts_at - Duration::from_millis(100)),
        Some(true)
    );
    assert_eq!(page.opacity_groups[&1], 0.0);
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_millis(250)),
        Some(true)
    );
    assert!((page.opacity_groups[&1] - 0.25).abs() < 0.01);
}

#[test]
fn unrelated_interaction_restyles_preserve_the_animation_timeline() {
    let mut page = animated_page("fade 1s linear infinite", 1.0);
    let starts_at = page.keyframe_animation_start(1).unwrap();

    page.rebuild_interaction_styles();

    assert_eq!(page.keyframe_animation_start(1), Some(starts_at));
}

fn animated_page(animation: &str, base_opacity: f32) -> PageRasterizer {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Animated".to_owned())],
    }];
    let rule = StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "opacity".to_owned(),
                value: base_opacity.to_string(),
            },
            StyleDeclaration {
                property: "animation".to_owned(),
                value: animation.to_owned(),
            },
        ],
    };
    let keyframes = [ctml_css::CssKeyframes {
        name: "fade".to_owned(),
        frames: vec![
            ctml_css::CssKeyframe {
                offsets: vec![0.0],
                declarations: vec![StyleDeclaration {
                    property: "opacity".to_owned(),
                    value: "0".to_owned(),
                }],
            },
            ctml_css::CssKeyframe {
                offsets: vec![1.0],
                declarations: vec![StyleDeclaration {
                    property: "opacity".to_owned(),
                    value: "1".to_owned(),
                }],
            },
        ],
    }];
    PageRasterizer::new_with_keyframes(&nodes, &[rule], &keyframes)
}
