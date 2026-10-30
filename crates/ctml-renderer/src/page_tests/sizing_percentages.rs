use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn nested_content_block_percentage_height_uses_the_containing_block_extent() {
    for property in ["height", "block-size"] {
        let nodes = [UiNode::Element {
            tag: "div".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Element {
                tag: "section".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text("short".to_owned())],
            }],
        }];
        let stylesheet = [
            StyleRule {
                selectors: vec!["div".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: "height".to_owned(),
                    value: "200px".to_owned(),
                }],
            },
            StyleRule {
                selectors: vec!["section".to_owned()],
                layer: None,
                declarations: vec![
                    StyleDeclaration {
                        property: property.to_owned(),
                        value: "50%".to_owned(),
                    },
                    StyleDeclaration {
                        property: "overflow".to_owned(),
                        value: "hidden".to_owned(),
                    },
                ],
            },
        ];
        let mut page = PageRasterizer::new(&nodes, &stylesheet);
        page.snapshot_pixels(300, 500);
        let geometry = page.overflow_geometry[&2];
        let used_height = geometry.clip.bottom - geometry.clip.top;

        assert!(
            (used_height - 100.0).abs() < 1.0,
            "50% nested {property} should use the containing 200px block, got {used_height}px"
        );
    }
}
