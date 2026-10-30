use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn paints_solid_and_dotted_block_borders_with_declared_colors() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("bordered block".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![
            ("border", "5px solid red"),
            ("border-left", "5px dotted blue"),
        ]
        .into_iter()
        .map(|(property, value)| StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        })
        .collect(),
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(300, 200);

    let red_pixels = pixels
        .chunks_exact(4)
        .filter(|pixel| pixel[0] > 200 && pixel[1] < 80 && pixel[2] < 80)
        .count();
    let blue_pixels = pixels
        .chunks_exact(4)
        .filter(|pixel| pixel[2] > 200 && pixel[0] < 80 && pixel[1] < 80)
        .count();
    assert!(red_pixels > 50, "solid red border should be painted");
    assert!(blue_pixels > 10, "dotted blue border should be painted");
}

#[test]
fn current_color_is_used_by_border_shorthand() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("current color".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![("color", "purple"), ("border", "3px solid currentColor")]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(300, 200);
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[0] > 80 && pixel[2] > 80),
        "currentColor should color the element border"
    );
}
