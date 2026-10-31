use std::time::Duration;

use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn opacity_keyframes_compose_with_the_underlying_value() {
    for composition in ["add", "accumulate"] {
        let nodes = [UiNode::Element {
            tag: "div".to_owned(),
            class: None,
            id: None,
            attributes: vec![("style".to_owned(), Some("opacity: 0.2".to_owned()))],
            children: vec![UiNode::Text("Composed opacity".to_owned())],
        }];
        let stylesheet = [StyleRule {
            selectors: vec!["div".to_owned()],
            layer: None,
            declarations: [
                ("animation-name", "replace-value, additive-value"),
                ("animation-duration", "0s, 0s"),
                ("animation-fill-mode", "forwards, forwards"),
                ("animation-composition", &format!("replace, {composition}")),
            ]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
        }];
        let keyframes = [
            opacity_keyframes("replace-value", 0.3),
            opacity_keyframes("additive-value", 0.4),
        ];
        let mut page = PageRasterizer::new_with_keyframes(&nodes, &stylesheet, &keyframes);
        let starts_at = page.keyframe_animation_start(1).unwrap();

        page.advance_animations(starts_at + Duration::from_millis(1));

        assert!(
            (page.opacity_groups[&1] - 0.7).abs() < 0.01,
            "{composition} should combine the 0.4 effect value with the underlying 0.3 value"
        );
    }
}

fn opacity_keyframes(name: &str, opacity: f32) -> ctml_css::CssKeyframes {
    ctml_css::CssKeyframes {
        name: name.to_owned(),
        frames: vec![ctml_css::CssKeyframe {
            offsets: vec![1.0],
            declarations: vec![StyleDeclaration {
                property: "opacity".to_owned(),
                value: opacity.to_string(),
            }],
        }],
    }
}
