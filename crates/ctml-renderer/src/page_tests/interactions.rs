use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn pointer_hover_and_active_states_restyle_text_runs() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Hover".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p:hover".to_owned()],
            declarations: vec![StyleDeclaration {
                property: "color".to_owned(),
                value: "red".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p:active".to_owned()],
            declarations: vec![StyleDeclaration {
                property: "font-weight".to_owned(),
                value: "700".to_owned(),
            }],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let normal = page.snapshot_pixels(100, 50);

    assert!(page.update_hover(10.0, 14.0));
    let hovered = page.snapshot_pixels(100, 50);
    assert_ne!(normal, hovered);
    assert!(!page.update_hover(10.0, 14.0));

    assert!(page.begin_selection(10.0, 14.0));
    let active = page.snapshot_pixels(100, 50);
    assert_ne!(hovered, active);
    assert!(page.end_active());
    assert_eq!(page.snapshot_pixels(100, 50), hovered);
}
