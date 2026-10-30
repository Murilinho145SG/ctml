use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn first_line_box_shadow_includes_block_padding() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("First formatted line".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-line".to_owned()],
        layer: None,
        declarations: [("padding-block", "4px"), ("box-shadow", "0 0 0 2px red")]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }];
    let rendered = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(180, 70);

    assert_eq!(&rendered[(3 * 180 + 20) * 4..][..3], [255, 0, 0]);
}
