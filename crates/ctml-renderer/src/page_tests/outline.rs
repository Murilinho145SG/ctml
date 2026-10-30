use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn outline_paints_outside_the_border_box_and_honors_offset() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("outlined block".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![
            ("border", "3px solid red"),
            ("outline", "4px dashed blue"),
            ("outline-offset", "5px"),
        ]
        .into_iter()
        .map(|(property, value)| StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        })
        .collect(),
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(320, 180);
    let rect = page.overflow_geometry[&1].border_box;
    let x = (rect.left - 9.0 - 8.0 + 4.0) as usize;
    let outer_top = (rect.top - 9.0 - 8.0) as usize;
    let blue_at = |y: usize| {
        let pixel = &pixels[(y * 320 + x) * 4..][..4];
        pixel[0] < 80 && pixel[1] < 80 && pixel[2] > 200
    };
    let blue_pixels = pixels
        .chunks_exact(4)
        .filter(|pixel| pixel[0] < 80 && pixel[1] < 80 && pixel[2] > 200)
        .count();
    assert!(blue_pixels > 20, "outline should paint dashed blue pixels");
    assert!(
        blue_at(outer_top + 1),
        "outline should start outside the border box after its offset"
    );
    assert!(
        !blue_at(outer_top + 6),
        "the positive offset should leave a gap before the outline"
    );
}

#[test]
fn rounded_outline_tracks_the_block_corner_shape() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("rounded outline".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![("border-radius", "35px"), ("outline", "5px solid blue")]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(300, 180);
    let rect = page.overflow_geometry[&1].border_box;
    let x = (rect.left - 8.0 - 4.0) as usize;
    let y = (rect.top - 8.0 - 4.0) as usize;
    let pixel = &pixels[(y * 300 + x) * 4..][..4];
    assert!(
        pixel[0] > 200 && pixel[1] > 200,
        "the rounded outline should leave its outer corner unpainted"
    );
}
