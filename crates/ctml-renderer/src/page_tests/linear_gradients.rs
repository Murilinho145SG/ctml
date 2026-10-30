use super::super::{PAGE_PADDING, PageRasterizer};
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn repeating_linear_gradient_repeats_between_its_outer_stops() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("gradient".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: Vec::new(),
    }];
    let stylesheet = [StyleRule {
        selectors: vec![".gradient".to_owned()],
        layer: None,
        declarations: [
            ("width", "80px"),
            ("height", "24px"),
            (
                "background-image",
                "repeating-linear-gradient(to right, red 0%, blue 25%)",
            ),
        ]
        .into_iter()
        .map(|(property, value)| StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        })
        .collect(),
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(120, 80);
    let rect = page.overflow_geometry[&1].border_box;
    let pixels = page.snapshot_pixels(120, 80);
    let y = (rect.top + 12.0).round() as usize;
    let read = |x: usize| {
        let pixel = &pixels[(y * 120 + x) * 4..][..3];
        [pixel[0], pixel[1], pixel[2]]
    };
    let red_start = read((rect.left - PAGE_PADDING + 2.0).round() as usize);
    let blue_stop = read((rect.left - PAGE_PADDING + 19.0).round() as usize);
    let next_red = read((rect.left - PAGE_PADDING + 22.0).round() as usize);

    assert!(
        red_start[0] > red_start[2],
        "first stop region: {red_start:?}"
    );
    assert!(
        blue_stop[2] > blue_stop[0],
        "last stop region: {blue_stop:?}"
    );
    assert!(next_red[0] > next_red[2], "next repeat: {next_red:?}");
}
