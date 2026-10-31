use std::time::Duration;

use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule};

fn rule(selector: &str, declarations: &[(&str, &str)]) -> StyleRule {
    StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: declarations
            .iter()
            .map(|(property, value)| StyleDeclaration {
                property: (*property).to_owned(),
                value: (*value).to_owned(),
            })
            .collect(),
    }
}

#[test]
fn root_opacity_transition_interpolates_and_finishes_at_target() {
    let stylesheet = [
        rule("body", &[("transition", "opacity 1000ms linear")]),
        rule("body:hover", &[("opacity", "0")]),
    ];
    let mut page = PageRasterizer::new(&[], &stylesheet);
    page.hovered_path.push(1);
    page.rebuild_interaction_styles();

    let transition = page.root_opacity_transition.as_ref().unwrap();
    let starts_at = transition.starts_at;
    assert!((page.root_opacity - 1.0).abs() < 0.001);
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_millis(500)),
        Some(true)
    );
    assert!((page.root_opacity - 0.5).abs() < 0.01);
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_secs(1)),
        Some(false)
    );
    assert_eq!(page.root_opacity, 0.0);
    assert!(page.root_opacity_transition.is_none());
}

#[test]
fn zero_duration_keeps_interaction_opacity_discrete() {
    let stylesheet = [rule("body:hover", &[("opacity", "0")])];
    let mut page = PageRasterizer::new(&[], &stylesheet);
    page.hovered_path.push(1);
    page.rebuild_interaction_styles();
    assert_eq!(page.root_opacity, 0.0);
    assert!(page.root_opacity_transition.is_none());
}

#[test]
fn text_element_opacity_transitions_on_hover() {
    let nodes = [crate::UiNode::Element {
        tag: "button".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![crate::UiNode::Text("Fade".to_owned())],
    }];
    let stylesheet = [
        rule("button", &[("transition", "opacity 1000ms linear")]),
        rule("button:hover", &[("opacity", "0")]),
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.hovered_path.push(1);
    page.rebuild_interaction_styles();

    let transition = page.opacity_transitions.get(&1).unwrap();
    let starts_at = transition.starts_at;
    assert_eq!(page.opacity_groups.get(&1), Some(&1.0));
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_millis(500)),
        Some(true)
    );
    assert!((page.opacity_groups[&1] - 0.5).abs() < 0.01);
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_secs(1)),
        Some(false)
    );
    assert_eq!(page.opacity_groups.get(&1), Some(&0.0));
}

#[test]
fn wrapper_opacity_transitions_when_only_descendants_render_text() {
    let nodes = [crate::UiNode::Element {
        tag: "section".to_owned(),
        class: Some("wrapper".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![crate::UiNode::Element {
            tag: "button".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![crate::UiNode::Text("Nested".to_owned())],
        }],
    }];
    let stylesheet = [
        rule(".wrapper", &[("transition", "opacity 1000ms linear")]),
        rule(".wrapper:hover", &[("opacity", "0")]),
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.hovered_path.push(1);
    page.rebuild_interaction_styles();

    let transition = page.opacity_transitions.get(&1).unwrap();
    let starts_at = transition.starts_at;
    assert_eq!(page.opacity_groups.get(&1), Some(&1.0));
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_millis(500)),
        Some(true)
    );
    assert!((page.opacity_groups[&1] - 0.5).abs() < 0.01);
}

#[test]
fn text_color_transitions_interpolate_between_rgba_values() {
    let nodes = [crate::UiNode::Element {
        tag: "button".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![crate::UiNode::Text("Color".to_owned())],
    }];
    let stylesheet = [
        rule(
            "button",
            &[("color", "blue"), ("transition", "color 1000ms linear")],
        ),
        rule("button:hover", &[("color", "red")]),
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    assert_eq!(page.runs[0].style.color, [0, 0, 255, 255]);
    page.hovered_path.push(1);
    page.rebuild_interaction_styles();

    let transition = page.color_transitions.get(&1).unwrap();
    let starts_at = transition.starts_at;
    assert_eq!(page.runs[0].style.color, [0, 0, 255, 255]);
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_millis(500)),
        Some(true)
    );
    assert_eq!(page.runs[0].style.color, [128, 0, 128, 255]);
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_secs(1)),
        Some(false)
    );
    assert_eq!(page.runs[0].style.color, [255, 0, 0, 255]);
}

#[test]
fn wrapper_color_transitions_inherited_text_without_overriding_explicit_descendant_colors() {
    let nodes = [crate::UiNode::Element {
        tag: "section".to_owned(),
        class: Some("wrapper".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: [("inherited", "Inherited"), ("explicit", "Explicit")]
            .into_iter()
            .map(|(class, text)| crate::UiNode::Element {
                tag: "p".to_owned(),
                class: Some(class.to_owned()),
                id: None,
                attributes: Vec::new(),
                children: vec![crate::UiNode::Text(text.to_owned())],
            })
            .collect(),
    }];
    let stylesheet = [
        rule(
            ".wrapper",
            &[("color", "blue"), ("transition", "color 1000ms linear")],
        ),
        rule(".wrapper:hover", &[("color", "red")]),
        rule(".explicit", &[("color", "green")]),
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.hovered_path.push(1);
    page.rebuild_interaction_styles();

    let transition = page.color_transitions.get(&1).unwrap();
    let starts_at = transition.starts_at;
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_millis(500)),
        Some(true)
    );
    let inherited = page
        .runs
        .iter()
        .find(|run| run.text.contains('I'))
        .expect("inherited descendant should render text");
    let explicit = page
        .runs
        .iter()
        .find(|run| run.text.contains('E'))
        .expect("explicit descendant should render text");
    assert_eq!(inherited.style.color, [128, 0, 128, 255]);
    assert_eq!(explicit.style.color, [0, 128, 0, 255]);
}

#[test]
fn background_color_transitions_interpolate_for_text_elements() {
    let nodes = [crate::UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![crate::UiNode::Text("Background".to_owned())],
    }];
    let stylesheet = [
        rule(
            "div",
            &[
                ("display", "block"),
                ("width", "80px"),
                ("background-color", "blue"),
                ("transition", "background-color 1000ms linear"),
            ],
        ),
        rule("div:hover", &[("background-color", "red")]),
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.hovered_path.push(1);
    page.rebuild_interaction_styles();

    let transition = page.background_color_transitions.get(&1).unwrap();
    let starts_at = transition.starts_at;
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_millis(500)),
        Some(true)
    );
    assert_eq!(
        page.runs[0].style.background_color,
        Some([128, 0, 128, 255])
    );
    assert_eq!(
        page.runs[0].style.background_layers,
        vec![[128, 0, 128, 255]]
    );
    let block_background = page.runs[0]
        .style
        .layout_sizes
        .iter()
        .find(|size| size.node_id == 1)
        .expect("block background should have layout geometry");
    assert_eq!(block_background.background_layers, vec![[128, 0, 128, 255]]);
    assert_eq!(
        page.advance_animations(starts_at + Duration::from_secs(1)),
        Some(false)
    );
    assert_eq!(page.runs[0].style.background_color, Some([255, 0, 0, 255]));
}
