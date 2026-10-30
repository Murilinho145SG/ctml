use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn rounded_border_reduces_painted_corner_pixels() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("rounded box".to_owned())],
    }];
    let make_rule = |radius: Option<&str>| {
        let mut declarations = vec![("border", "10px solid red")];
        if let Some(radius) = radius {
            declarations.push(("border-radius", radius));
        }
        StyleRule {
            selectors: vec!["div".to_owned()],
            layer: None,
            declarations: declarations
                .into_iter()
                .map(|(property, value)| StyleDeclaration {
                    property: property.to_owned(),
                    value: value.to_owned(),
                })
                .collect(),
        }
    };
    let count_red = |stylesheet: &[StyleRule]| {
        let mut page = PageRasterizer::new(&nodes, stylesheet);
        let pixels = page.snapshot_pixels(300, 180);
        let geometry = page.overflow_geometry[&1].border_box;
        let (left, top) = (
            (geometry.left - 8.0).floor() as usize,
            (geometry.top - 8.0).floor() as usize,
        );
        let (right, bottom) = (
            (geometry.right - 8.0).ceil() as usize,
            (geometry.bottom - 8.0).ceil() as usize,
        );
        (left, top, right, bottom, pixels)
    };
    let (_, _, _, _, square_pixels) = count_red(&[make_rule(None)]);
    let (left, top, right, bottom, rounded_pixels) = count_red(&[make_rule(Some("45px"))]);
    let red = |pixels: &[u8], x: usize, y: usize| {
        let pixel = &pixels[(y * 300 + x) * 4..][..4];
        pixel[0] > 200 && pixel[1] < 80 && pixel[2] < 80
    };
    let square_count = square_pixels
        .chunks_exact(4)
        .filter(|pixel| pixel[0] > 200 && pixel[1] < 80 && pixel[2] < 80)
        .count();
    let rounded_count = rounded_pixels
        .chunks_exact(4)
        .filter(|pixel| pixel[0] > 200 && pixel[1] < 80 && pixel[2] < 80)
        .count();
    assert!(
        rounded_count < square_count,
        "rounded corners should remove border pixels: {rounded_count} vs {square_count}"
    );
    assert!(
        !red(&rounded_pixels, left + 1, top + 1),
        "outer corner should be clipped by its radius"
    );
    assert!(
        red(&rounded_pixels, (left + right) / 2, top + 1),
        "straight top edge should remain painted"
    );
    assert!(bottom > top);
}
