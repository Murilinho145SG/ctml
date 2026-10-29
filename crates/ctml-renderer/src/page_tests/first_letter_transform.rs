use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn first_letter_text_transform_uses_source_text_and_only_changes_the_letter() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Hello".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "text-transform".to_owned(),
                value: "uppercase".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p::first-letter".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "text-transform".to_owned(),
                value: "lowercase".to_owned(),
            }],
        },
    ];
    let page = PageRasterizer::new(&nodes, &stylesheet);

    assert_eq!(page.runs[0].text, "h");
    assert_eq!(page.runs[1].text, "ELLO");

    let expansion_nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("ßeta".to_owned())],
    }];
    let expansion_page = PageRasterizer::new(&expansion_nodes, &stylesheet);
    assert_eq!(expansion_page.runs[0].text, "ß");
    assert_eq!(expansion_page.runs[1].text, "ETA");
}
