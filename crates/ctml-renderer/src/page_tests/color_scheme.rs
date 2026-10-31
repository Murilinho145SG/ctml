use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn dark_color_scheme_selects_system_colors_in_rasterized_text_and_backgrounds() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Dark scheme".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: [
            ("color", "CanvasText"),
            ("background-color", "Canvas"),
            ("color-scheme", "dark"),
        ]
        .into_iter()
        .map(|(property, value)| StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        })
        .collect(),
    }];
    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(180, 80);

    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [24, 24, 24])
    );
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [245, 245, 245])
    );
}
