use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

fn block_shadow_pixels(shadow: &str) -> (Vec<u8>, crate::page::overflow_clip::ClipRect) {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("shadow".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![
            ("box-shadow", shadow),
            ("background-color", "white"),
            ("overflow", "hidden"),
        ]
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
    (pixels, rect)
}

#[test]
fn outer_box_shadow_paints_beyond_the_block_edge() {
    let (pixels, rect) = block_shadow_pixels("8px 0 0 blue");
    let x = (rect.right + 4.0) as usize;
    let y = ((rect.top + rect.bottom) / 2.0) as usize;
    let pixel = &pixels[(y * 300 + x) * 4..][..4];
    assert!(
        pixel[2] > 200 && pixel[0] < 80,
        "outer shadow should paint past the block border box"
    );
}

#[test]
fn inset_box_shadow_paints_inside_the_block() {
    let (pixels, rect) = block_shadow_pixels("inset 0 0 8px blue");
    let x = ((rect.left + rect.right) / 2.0) as usize;
    let y = (rect.top + 2.0) as usize;
    let pixel = &pixels[(y * 300 + x) * 4..][..4];
    assert!(
        pixel[2] > pixel[0],
        "inset shadow should tint the inner edge"
    );
}

#[test]
fn box_shadow_uses_the_rounded_block_silhouette() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("rounded shadow".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![
            ("border-radius", "40px"),
            ("box-shadow", "0 0 0 4px blue"),
            ("overflow", "hidden"),
        ]
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
    let corner_x = (rect.left - 2.0) as usize;
    let corner_y = (rect.top - 2.0) as usize;
    let top_x = ((rect.left + rect.right) / 2.0) as usize;
    let top_y = (rect.top - 2.0) as usize;
    let is_blue = |x: usize, y: usize| {
        let pixel = &pixels[(y * 300 + x) * 4..][..4];
        pixel[2] > 200 && pixel[0] < 80
    };
    assert!(
        !is_blue(corner_x, corner_y),
        "rounded shadow should leave the square corner empty"
    );
    assert!(
        is_blue(top_x, top_y),
        "the straight outer shadow should still paint"
    );
}
