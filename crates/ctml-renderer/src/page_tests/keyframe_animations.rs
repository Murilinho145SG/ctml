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

#[test]
fn paused_keyframes_freeze_and_resume_their_timeline() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Animated".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["div".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "animation".to_owned(),
                value: "fade 1s linear infinite".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["div:hover".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "animation-play-state".to_owned(),
                value: "paused".to_owned(),
            }],
        },
    ];
    let keyframes = [opacity_keyframes("fade", 0.0, 1.0)];
    let mut page = PageRasterizer::new_with_keyframes(&nodes, &stylesheet, &keyframes);
    let starts_at = page.keyframe_animation_start(1).unwrap();

    page.advance_animations(starts_at + Duration::from_millis(500));
    assert!((page.opacity_groups[&1] - 0.5).abs() < 0.01);

    let paused_at = starts_at + Duration::from_millis(500);
    page.hovered_path = vec![1];
    page.rebuild_interaction_styles_at(paused_at);
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_millis(900)),
        Some(false)
    );
    assert!((page.opacity_groups[&1] - 0.5).abs() < 0.01);

    page.hovered_path.clear();
    let resumed_at = starts_at + Duration::from_millis(900);
    page.rebuild_interaction_styles_at(resumed_at);
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_secs(1)),
        Some(true)
    );
    assert!((page.opacity_groups[&1] - 0.6).abs() < 0.01);
}

#[test]
fn disabling_animations_cancels_keyframes_and_stops_animation_ticks() {
    let mut page = animated_page("fade 1s linear infinite", 1.0);
    page.animations_enabled = false;

    page.rebuild_interaction_styles();

    assert!(page.keyframe_animations.is_empty());
    assert!(page.advance_animations(std::time::Instant::now()).is_none());
}

#[test]
fn visibility_keyframes_switch_discretely_and_respect_fill_mode() {
    let mut page = visibility_animated_page("fade 1s linear forwards");
    let starts_at = page.keyframe_animation_start(1).unwrap();

    assert_eq!(
        page.advance_animations(starts_at + Duration::from_millis(250)),
        Some(true)
    );
    assert_eq!(page.element_visibility[&1], false);
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_millis(750)),
        Some(true)
    );
    assert_eq!(page.element_visibility[&1], true);
    assert!(page.runs.iter().any(|run| run.style.visibility_hidden));
    assert!(
        page.cursor_spans
            .iter()
            .filter(|span| span.element_path.contains(&1))
            .all(|span| span.visibility_hidden)
    );
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_secs(1)),
        Some(false)
    );
    assert_eq!(page.element_visibility[&1], true);

    let mut no_fill = visibility_animated_page("fade 1s linear");
    let starts_at = no_fill.keyframe_animation_start(1).unwrap();
    no_fill.advance_animations(starts_at + Duration::from_millis(750));
    assert_eq!(no_fill.element_visibility[&1], true);
    no_fill.advance_animations(starts_at + Duration::from_secs(1));
    assert_eq!(no_fill.element_visibility[&1], false);
}

#[test]
fn animated_parent_visibility_preserves_an_explicitly_visible_child() {
    let nodes = [UiNode::Element {
        tag: "section".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "span".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("Visible".to_owned())],
        }],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["section".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "animation".to_owned(),
                value: "fade 1s linear forwards".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["span".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "visibility".to_owned(),
                value: "visible".to_owned(),
            }],
        },
    ];
    let keyframes = [ctml_css::CssKeyframes {
        name: "fade".to_owned(),
        frames: vec![
            ctml_css::CssKeyframe {
                offsets: vec![0.0],
                declarations: vec![StyleDeclaration {
                    property: "visibility".to_owned(),
                    value: "visible".to_owned(),
                }],
            },
            ctml_css::CssKeyframe {
                offsets: vec![1.0],
                declarations: vec![StyleDeclaration {
                    property: "visibility".to_owned(),
                    value: "hidden".to_owned(),
                }],
            },
        ],
    }];
    let mut page = PageRasterizer::new_with_keyframes(&nodes, &stylesheet, &keyframes);
    let starts_at = page.keyframe_animation_start(1).unwrap();

    page.advance_animations(starts_at + Duration::from_millis(750));

    assert_eq!(page.element_visibility[&1], true);
    assert!(
        page.runs
            .iter()
            .any(|run| { run.element_path.contains(&2) && !run.style.visibility_hidden })
    );
}

#[test]
fn multiple_animations_apply_distinct_properties_and_later_values_win() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Animated".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "animation".to_owned(),
                value: "fade 1s linear forwards, hide 1s linear forwards".to_owned(),
            },
            StyleDeclaration {
                property: "opacity".to_owned(),
                value: "1".to_owned(),
            },
        ],
    }];
    let keyframes = [
        opacity_keyframes("fade", 0.0, 0.4),
        visibility_keyframes("hide"),
    ];
    let mut page = PageRasterizer::new_with_keyframes(&nodes, &stylesheet, &keyframes);
    let starts_at = page.keyframe_animation_start(1).unwrap();
    assert_eq!(page.keyframe_animations[&1].len(), 2);

    page.advance_animations(starts_at + Duration::from_millis(750));

    assert!((page.opacity_groups[&1] - 0.3).abs() < 0.01);
    assert!(page.element_visibility[&1]);
}

#[test]
fn later_animation_overrides_the_same_property_at_equal_progress() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Animated".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "animation".to_owned(),
                value: "fade 1s linear forwards, pulse 1s linear forwards".to_owned(),
            },
            StyleDeclaration {
                property: "opacity".to_owned(),
                value: "1".to_owned(),
            },
        ],
    }];
    let keyframes = [
        opacity_keyframes("fade", 0.0, 0.4),
        opacity_keyframes("pulse", 0.6, 1.0),
    ];
    let mut page = PageRasterizer::new_with_keyframes(&nodes, &stylesheet, &keyframes);
    let starts_at = page.keyframe_animation_start(1).unwrap();

    page.advance_animations(starts_at + Duration::from_millis(500));

    assert!((page.opacity_groups[&1] - 0.8).abs() < 0.01);
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

fn visibility_animated_page(animation: &str) -> PageRasterizer {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Animated".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "animation".to_owned(),
            value: animation.to_owned(),
        }],
    }];
    let keyframes = [visibility_keyframes("fade")];
    PageRasterizer::new_with_keyframes(&nodes, &stylesheet, &keyframes)
}

fn opacity_keyframes(name: &str, from: f32, to: f32) -> ctml_css::CssKeyframes {
    ctml_css::CssKeyframes {
        name: name.to_owned(),
        frames: [from, to]
            .into_iter()
            .enumerate()
            .map(|(index, opacity)| ctml_css::CssKeyframe {
                offsets: vec![index as f32],
                declarations: vec![StyleDeclaration {
                    property: "opacity".to_owned(),
                    value: opacity.to_string(),
                }],
            })
            .collect(),
    }
}

fn visibility_keyframes(name: &str) -> ctml_css::CssKeyframes {
    ctml_css::CssKeyframes {
        name: name.to_owned(),
        frames: ["visible", "hidden"]
            .into_iter()
            .enumerate()
            .map(|(index, visibility)| ctml_css::CssKeyframe {
                offsets: vec![index as f32],
                declarations: vec![StyleDeclaration {
                    property: "visibility".to_owned(),
                    value: visibility.to_owned(),
                }],
            })
            .collect(),
    }
}
