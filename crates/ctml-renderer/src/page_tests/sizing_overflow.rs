use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn content_block_height_keeps_content_at_the_block_start_and_flows_after_its_used_height() {
    let text = "word ".repeat(80);
    let nodes = [
        text_paragraph("before"),
        UiNode::Element {
            tag: "div".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text(text)],
        },
        text_paragraph("after"),
    ];
    let stylesheet = [size_rule("height", "40px")];
    let max_height_stylesheet = [size_rule("max-height", "40px")];
    let max_block_size_stylesheet = [size_rule("max-block-size", "40px")];
    let mut natural = PageRasterizer::new(&nodes, &[]);
    let mut sized = PageRasterizer::new(&nodes, &stylesheet);
    let mut max_height = PageRasterizer::new(&nodes, &max_height_stylesheet);
    let mut max_block_size = PageRasterizer::new(&nodes, &max_block_size_stylesheet);
    natural.prepare_layout(400, 1200);
    sized.prepare_layout(400, 1200);
    max_height.prepare_layout(400, 1200);
    max_block_size.prepare_layout(400, 1200);

    let natural_content_y = text_line_y(&natural, "word");
    let sized_content_y = text_line_y(&sized, "word");
    let expected_after_y = sized_content_y + 40.0;
    assert!(
        (sized_content_y - natural_content_y).abs() < 1.0,
        "fixed height must not move content from the block start: {sized_content_y} vs {natural_content_y}"
    );
    assert_flow_at_used_end(&sized, expected_after_y, "height");
    assert_flow_at_used_end(&max_height, expected_after_y, "max-height");
    assert_flow_at_used_end(&max_block_size, expected_after_y, "max-block-size");
}

#[test]
fn overflow_hidden_clips_descendant_text_and_backgrounds_at_the_used_block_extent() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "span".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("word ".repeat(100))],
        }],
    }];
    let span_background = StyleRule {
        selectors: vec!["span".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "background-color".to_owned(),
            value: "red".to_owned(),
        }],
    };
    let mut stylesheet = vec![size_rule("height", "40px")];
    stylesheet[0].declarations.push(StyleDeclaration {
        property: "overflow".to_owned(),
        value: "hidden".to_owned(),
    });
    stylesheet.push(span_background.clone());
    let mut hidden_page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = hidden_page.snapshot_pixels(240, 220);
    let mut clip_stylesheet = stylesheet.clone();
    clip_stylesheet[0]
        .declarations
        .iter_mut()
        .find(|declaration| declaration.property == "overflow")
        .expect("overflow declaration should exist")
        .value = "clip visible".to_owned();
    let clip_pixels = PageRasterizer::new(&nodes, &clip_stylesheet).snapshot_pixels(240, 220);
    let mut visible_stylesheet = vec![size_rule("height", "40px")];
    visible_stylesheet.push(span_background);
    let visible_pixels = PageRasterizer::new(&nodes, &visible_stylesheet).snapshot_pixels(240, 220);
    let has_paint_below_clip = |pixels: &[u8]| {
        pixels
            .chunks_exact(4)
            .enumerate()
            .filter(|(index, _)| index / 240 >= 48)
            .any(|(_, pixel)| pixel[0] < 245 || pixel[1] < 245 || pixel[2] < 245)
    };
    assert!(
        has_paint_below_clip(&visible_pixels),
        "overflow: visible should keep the default"
    );
    let below_clip_has_paint = has_paint_below_clip(&pixels);
    assert!(
        !has_paint_below_clip(&clip_pixels),
        "overflow: clip should clip descendant paint"
    );
    assert!(
        !below_clip_has_paint,
        "overflowing text or its background escaped the 40px clip"
    );
    assert!(
        !hidden_page.begin_selection(20.0, 65.0),
        "clipped text should not receive pointer or selection hits"
    );
}

#[test]
fn overflow_hidden_clips_text_and_backgrounds_at_the_inline_extent() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "span".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text(
                "a long unwrapped line that overflows".to_owned(),
            )],
        }],
    }];
    let mut declarations = vec![
        StyleDeclaration {
            property: "width".to_owned(),
            value: "40px".to_owned(),
        },
        StyleDeclaration {
            property: "height".to_owned(),
            value: "80px".to_owned(),
        },
        StyleDeclaration {
            property: "white-space".to_owned(),
            value: "nowrap".to_owned(),
        },
    ];
    declarations.push(StyleDeclaration {
        property: "overflow-x".to_owned(),
        value: "hidden".to_owned(),
    });
    declarations.push(StyleDeclaration {
        property: "overflow-y".to_owned(),
        value: "visible".to_owned(),
    });
    let stylesheet = [
        StyleRule {
            selectors: vec!["div".to_owned()],
            layer: None,
            declarations,
        },
        StyleRule {
            selectors: vec!["span".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "background-color".to_owned(),
                value: "red".to_owned(),
            }],
        },
    ];
    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(240, 120);
    let mut visible_block = stylesheet[0].clone();
    visible_block
        .declarations
        .retain(|declaration| !declaration.property.starts_with("overflow"));
    let visible_stylesheet = [visible_block, stylesheet[1].clone()];
    let visible_pixels = PageRasterizer::new(&nodes, &visible_stylesheet).snapshot_pixels(240, 120);
    let has_escaped_paint = |pixels: &[u8]| {
        pixels.chunks_exact(4).enumerate().any(|(index, pixel)| {
            index % 240 >= 48 && (pixel[0] < 245 || pixel[1] < 245 || pixel[2] < 245)
        })
    };
    assert!(has_escaped_paint(&visible_pixels));
    let escaped_paint = has_escaped_paint(&pixels);
    assert!(
        !escaped_paint,
        "overflowing inline content escaped the 40px width clip"
    );
}

#[test]
fn overflow_hidden_clips_the_block_axis_in_vertical_root_writing_mode() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("vertical overflow ".repeat(100))],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "writing-mode".to_owned(),
                    value: "vertical-rl".to_owned(),
                },
                StyleDeclaration {
                    property: "inline-size".to_owned(),
                    value: "100px".to_owned(),
                },
            ],
        },
        StyleRule {
            selectors: vec!["div".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "block-size".to_owned(),
                    value: "50px".to_owned(),
                },
                StyleDeclaration {
                    property: "inline-size".to_owned(),
                    value: "100px".to_owned(),
                },
                StyleDeclaration {
                    property: "overflow".to_owned(),
                    value: "hidden".to_owned(),
                },
            ],
        },
    ];
    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(240, 240);
    let mut visible_block = stylesheet[1].clone();
    visible_block
        .declarations
        .retain(|declaration| declaration.property != "overflow");
    let visible_stylesheet = [stylesheet[0].clone(), visible_block];
    let visible_pixels = PageRasterizer::new(&nodes, &visible_stylesheet).snapshot_pixels(240, 240);
    let has_escaped_paint = |pixels: &[u8]| {
        pixels.chunks_exact(4).enumerate().any(|(index, pixel)| {
            index % 240 < 181 && (pixel[0] < 245 || pixel[1] < 245 || pixel[2] < 245)
        })
    };
    assert!(has_escaped_paint(&visible_pixels));
    let escaped_paint = has_escaped_paint(&pixels);
    assert!(
        !escaped_paint,
        "vertical overflow escaped the block-axis clip"
    );
}

#[test]
fn overflow_hidden_clips_nested_orthogonal_writing_mode_bounds() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("nested vertical overflow ".repeat(100))],
    }];
    for (writing_mode, bounds, outside_point) in [
        ("vertical-rl", (192.0, 8.0, 232.0, 88.0), (120.0, 20.0)),
        ("vertical-lr", (8.0, 152.0, 48.0, 232.0), (120.0, 180.0)),
    ] {
        let rule = |with_overflow: bool| {
            let mut declarations = vec![
                StyleDeclaration {
                    property: "writing-mode".to_owned(),
                    value: writing_mode.to_owned(),
                },
                StyleDeclaration {
                    property: "block-size".to_owned(),
                    value: "40px".to_owned(),
                },
                StyleDeclaration {
                    property: "inline-size".to_owned(),
                    value: "80px".to_owned(),
                },
                StyleDeclaration {
                    property: "background-color".to_owned(),
                    value: "#00ff00".to_owned(),
                },
            ];
            if with_overflow {
                declarations.push(StyleDeclaration {
                    property: "overflow".to_owned(),
                    value: "hidden".to_owned(),
                });
            }
            StyleRule {
                selectors: vec!["p".to_owned()],
                layer: None,
                declarations,
            }
        };
        let mut clipped_page = PageRasterizer::new(&nodes, &[rule(true)]);
        let clipped_pixels = clipped_page.snapshot_pixels(240, 240);
        let visible_pixels = PageRasterizer::new(&nodes, &[rule(false)]).snapshot_pixels(240, 240);
        let has_paint_outside_box = |pixels: &[u8]| {
            pixels.chunks_exact(4).enumerate().any(|(index, pixel)| {
                let x = (index % 240) as f32;
                let y = (index / 240) as f32;
                (x < bounds.0 || x >= bounds.2 || y < bounds.1 || y >= bounds.3)
                    && (pixel[0] < 245 || pixel[1] < 245 || pixel[2] < 245)
            })
        };
        assert!(has_paint_outside_box(&visible_pixels));
        assert!(!has_paint_outside_box(&clipped_pixels));
        assert!(!clipped_page.begin_selection(outside_point.0, outside_point.1));
    }
}

#[test]
fn root_overflow_auto_scrolls_with_wheel_offsets_but_hidden_does_not() {
    let nodes = (0..40)
        .map(|index| text_paragraph(&format!("scroll line {index}")))
        .collect::<Vec<_>>();
    let body_rule = |overflow: &str| StyleRule {
        selectors: vec!["body".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "overflow-y".to_owned(),
            value: overflow.to_owned(),
        }],
    };
    let mut scrollable = PageRasterizer::new(&nodes, &[body_rule("auto")]);
    let before = scrollable.snapshot_pixels(220, 100);
    assert!(scrollable.scroll_by(0.0, -36.0));
    let after = scrollable.snapshot_pixels(220, 100);
    assert_ne!(
        before, after,
        "root scrolling should move the viewport content"
    );
    let offset = scrollable.buffer.scroll();
    assert!(offset.line > 0 || offset.vertical > 0.0);

    let mut clipped = PageRasterizer::new(&nodes, &[body_rule("hidden")]);
    clipped.snapshot_pixels(220, 100);
    assert!(!clipped.scroll_by(0.0, -36.0));
}

fn size_rule(property: &str, value: &str) -> StyleRule {
    StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "width".to_owned(),
                value: "100px".to_owned(),
            },
            StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            },
        ],
    }
}

fn assert_flow_at_used_end(page: &PageRasterizer, expected_y: f32, property: &str) {
    let after_y = text_line_y(page, "after");
    assert!(
        (after_y - expected_y).abs() < 3.0,
        "following flow after {property} should start at {expected_y}, got {after_y}"
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
