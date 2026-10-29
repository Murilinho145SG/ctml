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
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "color".to_owned(),
                value: "red".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p:active".to_owned()],
            layer: None,
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

#[test]
fn focus_and_focus_within_restyle_clicked_links() {
    let nodes = [UiNode::Element {
        tag: "section".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "a".to_owned(),
            class: None,
            id: None,
            attributes: vec![("href".to_owned(), Some("/next".to_owned()))],
            children: vec![UiNode::Text("Focus".to_owned())],
        }],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["a:focus".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "color".to_owned(),
                value: "red".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["section:focus-within".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-weight".to_owned(),
                value: "700".to_owned(),
            }],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let normal = page.snapshot_pixels(100, 50);

    assert!(page.begin_selection(10.0, 14.0));
    assert_ne!(normal, page.snapshot_pixels(100, 50));

    assert!(page.begin_selection(90.0, 45.0));
    assert_eq!(normal, page.snapshot_pixels(100, 50));
}

#[test]
fn numeric_tabindex_makes_generic_elements_focusable() {
    let nodes = [UiNode::Element {
        tag: "span".to_owned(),
        class: None,
        id: None,
        attributes: vec![("tabindex".to_owned(), Some("0".to_owned()))],
        children: vec![UiNode::Text("Focusable".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["span:focus".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "color".to_owned(),
            value: "red".to_owned(),
        }],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let normal = page.snapshot_pixels(120, 50);

    assert!(page.begin_selection(10.0, 14.0));
    assert_ne!(normal, page.snapshot_pixels(120, 50));
}
