use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn aspect_ratio_sets_auto_block_size_from_the_used_inline_size() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("box".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![
            ("width", "200px"),
            ("aspect-ratio", "2 / 1"),
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
    page.snapshot_pixels(320, 220);
    let rect = page.overflow_geometry[&1].border_box;
    let height = rect.bottom - rect.top;
    assert!(
        (height - 100.0).abs() < 1.0,
        "expected a 100px used height, got {height}"
    );
}

#[test]
fn aspect_ratio_sizes_empty_blocks_and_reserves_the_derived_flow_space() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: Vec::new(),
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![
            ("width", "120px"),
            ("aspect-ratio", "3 / 2"),
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
    page.snapshot_pixels(240, 220);
    let rect = page.overflow_geometry[&1].border_box;
    let height = rect.bottom - rect.top;
    assert!(
        (height - 80.0).abs() < 1.0,
        "expected an 80px used height, got {height}"
    );
}

#[test]
fn aspect_ratio_derives_auto_inline_size_from_a_definite_block_size() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("box".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![
            ("height", "80px"),
            ("aspect-ratio", "2 / 1"),
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
    page.snapshot_pixels(320, 220);
    let rect = page.overflow_geometry[&1].border_box;
    let width = rect.right - rect.left;
    assert!(
        (width - 160.0).abs() < 1.0,
        "expected a 160px used width, got {width}"
    );
}

#[test]
fn border_box_aspect_ratio_includes_padding_in_the_ratio_box() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("box".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![
            ("width", "200px"),
            ("box-sizing", "border-box"),
            ("padding", "10px 0"),
            ("aspect-ratio", "2 / 1"),
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
    page.snapshot_pixels(320, 220);
    let rect = page.overflow_geometry[&1].border_box;
    let height = rect.bottom - rect.top;
    assert!(
        (height - 100.0).abs() < 1.0,
        "expected a 100px border-box height, got {height}"
    );
}
