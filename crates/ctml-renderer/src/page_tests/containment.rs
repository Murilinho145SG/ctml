use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn layout_containment_is_recorded_for_contain_and_content_visibility() {
    let nodes = [
        UiNode::Element {
            tag: "div".to_owned(),
            class: Some("layout".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("Layout contained".to_owned())],
        },
        UiNode::Element {
            tag: "div".to_owned(),
            class: Some("hidden".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("Hidden content".to_owned())],
        },
        UiNode::Element {
            tag: "div".to_owned(),
            class: Some("auto".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("Visible content".to_owned())],
        },
    ];
    let stylesheet = [
        (".layout", "contain", "layout"),
        (".hidden", "content-visibility", "hidden"),
        (".auto", "content-visibility", "auto"),
    ]
    .into_iter()
    .map(|(selector, property, value)| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    })
    .collect::<Vec<_>>();
    let page = PageRasterizer::new(&nodes, &stylesheet);

    for initial in ["L", "H", "V"] {
        assert!(
            page.runs
                .iter()
                .any(|run| { run.text.starts_with(initial) && run.style.layout_contained }),
            "text starting with {initial} should carry layout-containment state"
        );
    }
}
