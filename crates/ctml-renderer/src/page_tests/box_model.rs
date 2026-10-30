use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn padding_contributes_to_block_flow_and_border_box_widths() {
    let nodes = [
        UiNode::Element {
            tag: "div".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("inside".to_owned())],
        },
        text_paragraph("after"),
    ];
    let mut natural = PageRasterizer::new(&nodes, &[]);
    let padding_stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "padding".to_owned(),
            value: "10px 0".to_owned(),
        }],
    }];
    let mut padded = PageRasterizer::new(&nodes, &padding_stylesheet);
    natural.prepare_layout(300, 500);
    padded.prepare_layout(300, 500);
    let content_delta = text_line_y(&padded, "inside") - text_line_y(&natural, "inside");
    let flow_delta = text_line_y(&padded, "after") - text_line_y(&natural, "after");
    assert!(
        (content_delta - 10.0).abs() < 2.0,
        "10px top padding should inset block content by 10px, got {content_delta}px"
    );
    assert!(
        (flow_delta - 20.0).abs() < 3.0,
        "10px top and bottom padding should add 20px to block flow, got {flow_delta}px"
    );

    for (box_sizing, expected_width) in [("content-box", 120.0), ("border-box", 100.0)] {
        let declarations = vec![
            StyleDeclaration {
                property: "width".to_owned(),
                value: "100px".to_owned(),
            },
            StyleDeclaration {
                property: "height".to_owned(),
                value: "100px".to_owned(),
            },
            StyleDeclaration {
                property: "padding".to_owned(),
                value: "10px".to_owned(),
            },
            StyleDeclaration {
                property: "box-sizing".to_owned(),
                value: box_sizing.to_owned(),
            },
            StyleDeclaration {
                property: "overflow".to_owned(),
                value: "hidden".to_owned(),
            },
        ];
        let sized_nodes = [UiNode::Element {
            tag: "div".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("inside".to_owned())],
        }];
        let stylesheet = [StyleRule {
            selectors: vec!["div".to_owned()],
            layer: None,
            declarations,
        }];
        let mut page = PageRasterizer::new(&sized_nodes, &stylesheet);
        page.snapshot_pixels(300, 300);
        let geometry = page.overflow_geometry[&1];
        let used_width = geometry.clip.right - geometry.clip.left;
        assert!(
            (used_width - expected_width).abs() < 1.0,
            "{box_sizing} width with 10px padding should have a {expected_width}px outer width, got {used_width}px"
        );
        let used_height = geometry.clip.bottom - geometry.clip.top;
        assert!(
            (used_height - expected_width).abs() < 1.0,
            "{box_sizing} height with 10px padding should have a {expected_width}px outer height, got {used_height}px"
        );
    }
}

#[test]
fn border_widths_are_included_in_border_box_dimensions_and_flow() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("inside".to_owned())],
    }];
    for (box_sizing, expected_width, expected_height) in
        [("content-box", 124.0, 130.0), ("border-box", 100.0, 100.0)]
    {
        let stylesheet = [StyleRule {
            selectors: vec!["div".to_owned()],
            layer: None,
            declarations: vec![
                ("width", "100px"),
                ("height", "100px"),
                ("padding", "10px"),
                ("border-width", "5px 2px"),
                ("border-style", "solid"),
                ("box-sizing", box_sizing),
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
        page.snapshot_pixels(300, 300);
        let geometry = page.overflow_geometry[&1];
        assert!(
            ((geometry.clip.right - geometry.clip.left) - expected_width).abs() < 1.0,
            "{box_sizing} should produce {expected_width}px outer width, got {}px",
            geometry.clip.right - geometry.clip.left
        );
        assert!(
            ((geometry.clip.bottom - geometry.clip.top) - expected_height).abs() < 1.0,
            "{box_sizing} should produce {expected_height}px outer height, got {}px",
            geometry.clip.bottom - geometry.clip.top
        );
    }
}

#[test]
fn inline_start_padding_offsets_text_and_resolves_font_relative_lengths() {
    let nodes = [
        UiNode::Element {
            tag: "div".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("inside".to_owned())],
        },
        text_paragraph("after"),
    ];
    let mut natural = PageRasterizer::new(&nodes, &[]);
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "padding-left".to_owned(),
                value: "1em".to_owned(),
            },
            StyleDeclaration {
                property: "font-size".to_owned(),
                value: "20px".to_owned(),
            },
        ],
    }];
    let mut padded = PageRasterizer::new(&nodes, &stylesheet);
    natural.prepare_layout(300, 300);
    padded.prepare_layout(300, 300);
    let natural_x = text_line_x(&natural, "inside");
    let padded_x = text_line_x(&padded, "inside");
    assert!(
        (padded_x - natural_x - 20.0).abs() < 1.0,
        "20px left padding should offset inline content by 20px, got {}px",
        padded_x - natural_x
    );
}

#[test]
fn percentage_padding_uses_the_containing_block_inline_size() {
    let nodes = [UiNode::Element {
        tag: "section".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![
            UiNode::Element {
                tag: "div".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text("inside".to_owned())],
            },
            text_paragraph("after"),
        ],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["section".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "width".to_owned(),
                value: "200px".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["div".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "padding-left".to_owned(),
                    value: "10%".to_owned(),
                },
                StyleDeclaration {
                    property: "padding-top".to_owned(),
                    value: "10%".to_owned(),
                },
                StyleDeclaration {
                    property: "padding-bottom".to_owned(),
                    value: "10%".to_owned(),
                },
            ],
        },
    ];
    let mut natural = PageRasterizer::new(&nodes, &[]);
    let mut padded = PageRasterizer::new(&nodes, &stylesheet);
    natural.prepare_layout(300, 300);
    padded.prepare_layout(300, 300);
    let horizontal_offset = text_line_x(&padded, "inside") - text_line_x(&natural, "inside");
    let vertical_offset = text_line_y(&padded, "inside") - text_line_y(&natural, "inside");
    let flow_offset = text_line_y(&padded, "after") - text_line_y(&natural, "after");
    assert!(
        (horizontal_offset - 20.0).abs() < 2.0,
        "10% left padding in a 200px containing block should offset text by 20px, got {horizontal_offset}px"
    );
    assert!(
        (vertical_offset - 20.0).abs() < 3.0,
        "10% top padding should resolve against the 200px containing inline size, got {vertical_offset}px"
    );
    assert!(
        (flow_offset - 40.0).abs() < 5.0,
        "10% top and bottom padding should add 40px to flow, got {flow_offset}px"
    );
}

#[test]
fn logical_block_start_padding_insets_content_and_adds_to_flow() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("inside".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "padding-block-start".to_owned(),
            value: "12px".to_owned(),
        }],
    }];
    let mut natural = PageRasterizer::new(&nodes, &[]);
    let mut padded = PageRasterizer::new(&nodes, &stylesheet);
    natural.prepare_layout(300, 300);
    padded.prepare_layout(300, 300);
    let offset = text_line_y(&padded, "inside") - text_line_y(&natural, "inside");
    assert!(
        (offset - 12.0).abs() < 2.0,
        "expected 12px inset, got {offset}px"
    );
}

#[test]
fn logical_percentage_margins_offset_text_and_add_block_flow_space() {
    let nodes = [UiNode::Element {
        tag: "section".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![
            UiNode::Element {
                tag: "div".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text("inside".to_owned())],
            },
            text_paragraph("after"),
        ],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["section".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "width".to_owned(),
                value: "200px".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["div".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "margin-inline-start".to_owned(),
                    value: "10%".to_owned(),
                },
                StyleDeclaration {
                    property: "margin-block".to_owned(),
                    value: "10%".to_owned(),
                },
            ],
        },
    ];
    let mut natural = PageRasterizer::new(&nodes, &[]);
    let mut margined = PageRasterizer::new(&nodes, &stylesheet);
    natural.prepare_layout(300, 300);
    margined.prepare_layout(300, 300);
    let horizontal_offset = text_line_x(&margined, "inside") - text_line_x(&natural, "inside");
    let vertical_offset = text_line_y(&margined, "inside") - text_line_y(&natural, "inside");
    let flow_offset = text_line_y(&margined, "after") - text_line_y(&natural, "after");
    assert!(
        (horizontal_offset - 20.0).abs() < 2.0,
        "got {horizontal_offset}px"
    );
    assert!(
        (vertical_offset - 20.0).abs() < 3.0,
        "got {vertical_offset}px"
    );
    assert!((flow_offset - 40.0).abs() < 5.0, "got {flow_offset}px");
}

#[test]
fn negative_block_start_margin_pulls_content_upward() {
    let nodes = [
        UiNode::Element {
            tag: "div".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("inside".to_owned())],
        },
        text_paragraph("after"),
    ];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "margin-top".to_owned(),
            value: "-8px".to_owned(),
        }],
    }];
    let mut natural = PageRasterizer::new(&nodes, &[]);
    let mut margined = PageRasterizer::new(&nodes, &stylesheet);
    natural.prepare_layout(300, 300);
    margined.prepare_layout(300, 300);
    let content_offset = text_line_y(&margined, "inside") - text_line_y(&natural, "inside");
    let flow_offset = text_line_y(&margined, "after") - text_line_y(&natural, "after");
    assert!(
        content_offset.abs() < 1.0,
        "text should retain its line position, got {content_offset}px"
    );
    assert!(
        (flow_offset + 8.0).abs() < 2.0,
        "expected -8px flow offset, got {flow_offset}px"
    );
}

#[test]
fn zero_padding_does_not_insert_an_empty_flow_line() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("inside".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "padding".to_owned(),
            value: "0".to_owned(),
        }],
    }];
    let mut natural = PageRasterizer::new(&nodes, &[]);
    let mut zero_padding = PageRasterizer::new(&nodes, &stylesheet);
    natural.prepare_layout(300, 300);
    zero_padding.prepare_layout(300, 300);
    assert_eq!(
        text_line_y(&zero_padding, "inside"),
        text_line_y(&natural, "inside")
    );
}

fn text_paragraph(text: &str) -> UiNode {
    UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text(text.to_owned())],
    }
}

fn text_line_y(page: &PageRasterizer, text: &str) -> f32 {
    page.buffer
        .layout_runs()
        .find(|line| line.text.contains(text))
        .expect("text line should be in layout")
        .line_top
}

fn text_line_x(page: &PageRasterizer, text: &str) -> f32 {
    page.buffer
        .layout_runs()
        .find(|line| line.text.contains(text))
        .and_then(|line| line.glyphs.first().map(|glyph| glyph.x))
        .expect("text glyph should be in layout")
}
