use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn display_flex_places_block_children_in_a_shared_row() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("container".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: ["First item", "Second item"]
            .into_iter()
            .map(|text| UiNode::Element {
                tag: "p".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text(text.to_owned())],
            })
            .collect(),
    }];
    let stylesheet = [StyleRule {
        selectors: vec![".container".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "display".to_owned(),
            value: "flex".to_owned(),
        }],
    }];

    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(400, 120);

    assert_eq!(page.buffer.layout_runs().count(), 1);
}
