use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn vertical_root_block_size_constrains_the_block_flow_axis() {
    let text = "block size limits vertical column flow ".repeat(20);
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text(text)],
    }];
    let root_style = |block_size: Option<&str>| {
        let mut declarations = vec![
            StyleDeclaration {
                property: "writing-mode".to_owned(),
                value: "vertical-rl".to_owned(),
            },
            StyleDeclaration {
                property: "inline-size".to_owned(),
                value: "40px".to_owned(),
            },
        ];
        if let Some(block_size) = block_size {
            declarations.push(StyleDeclaration {
                property: "block-size".to_owned(),
                value: block_size.to_owned(),
            });
        }
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations,
        }
    };
    let mut natural = PageRasterizer::new(&nodes, &[root_style(None)]);
    natural.prepare_layout(300, 300);
    let natural_lines = natural.buffer.layout_runs().count();
    let mut constrained = PageRasterizer::new(&nodes, &[root_style(Some("60px"))]);
    constrained.prepare_layout(300, 300);

    assert!(constrained.buffer.layout_runs().count() < natural_lines);
}

#[test]
fn vertical_size_containment_uses_the_physical_width_as_block_fallback() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: Some("contained".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("contained".to_owned())],
    }];
    let fallback_flow = |intrinsic_size: &str| {
        let stylesheet = [
            StyleRule {
                selectors: vec!["body".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: "writing-mode".to_owned(),
                    value: "vertical-rl".to_owned(),
                }],
            },
            StyleRule {
                selectors: vec![".contained".to_owned()],
                layer: None,
                declarations: vec![
                    StyleDeclaration {
                        property: "contain".to_owned(),
                        value: "size".to_owned(),
                    },
                    StyleDeclaration {
                        property: "contain-intrinsic-size".to_owned(),
                        value: intrinsic_size.to_owned(),
                    },
                ],
            },
        ];
        let mut page = PageRasterizer::new(&nodes, &stylesheet);
        page.prepare_layout(300, 300);
        page.runs
            .iter()
            .filter(|run| run.style.flow_spacer)
            .map(|run| run.style.line_height)
            .sum::<f32>()
    };

    let width_fallback_flow = fallback_flow("120px 40px");
    let height_fallback_flow = fallback_flow("40px 120px");
    assert!(
        width_fallback_flow > height_fallback_flow + 40.0,
        "vertical block flow must use contain-intrinsic-width (120px), got spacer heights {width_fallback_flow} and {height_fallback_flow}"
    );
}

#[test]
fn orthogonal_inline_size_containment_uses_the_inline_fallback_in_page_flow() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: Some("contained".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("vertical inline content".repeat(4))],
    }];
    let fallback_flow = |intrinsic_size: &str| {
        let stylesheet = [StyleRule {
            selectors: vec![".contained".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "writing-mode".to_owned(),
                    value: "vertical-rl".to_owned(),
                },
                StyleDeclaration {
                    property: "contain".to_owned(),
                    value: "inline-size".to_owned(),
                },
                StyleDeclaration {
                    property: "contain-intrinsic-size".to_owned(),
                    value: intrinsic_size.to_owned(),
                },
            ],
        }];
        let mut page = PageRasterizer::new(&nodes, &stylesheet);
        page.prepare_layout(300, 300);
        page.runs
            .iter()
            .filter(|run| run.style.flow_spacer)
            .map(|run| run.style.line_height)
            .sum::<f32>()
    };

    let short_inline_fallback = fallback_flow("50px 30px");
    let long_inline_fallback = fallback_flow("50px 60px");
    assert!(
        long_inline_fallback > short_inline_fallback + 20.0,
        "orthogonal inline-size containment should reserve its 30px or 60px physical-height fallback, got spacer heights {short_inline_fallback} and {long_inline_fallback}"
    );
}

#[test]
fn empty_orthogonal_inline_size_containment_uses_the_inline_fallback_in_page_flow() {
    let nodes = [
        text_paragraph("before"),
        UiNode::Element {
            tag: "div".to_owned(),
            class: Some("contained".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: Vec::new(),
        },
        text_paragraph("after"),
    ];
    let fallback_flow = |intrinsic_size: &str| {
        let stylesheet = [StyleRule {
            selectors: vec![".contained".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "writing-mode".to_owned(),
                    value: "vertical-rl".to_owned(),
                },
                StyleDeclaration {
                    property: "contain".to_owned(),
                    value: "inline-size".to_owned(),
                },
                StyleDeclaration {
                    property: "contain-intrinsic-size".to_owned(),
                    value: intrinsic_size.to_owned(),
                },
            ],
        }];
        let mut page = PageRasterizer::new(&nodes, &stylesheet);
        page.prepare_layout(300, 300);
        text_line_y(&page, "after")
    };

    let short_inline_fallback = fallback_flow("50px 30px");
    let long_inline_fallback = fallback_flow("50px 60px");
    assert!(
        long_inline_fallback > short_inline_fallback + 20.0,
        "empty orthogonal inline-size containment should reserve its 30px or 60px physical-height fallback, got spacer heights {short_inline_fallback} and {long_inline_fallback}"
    );
}

#[test]
fn nested_vertical_block_size_reserves_additional_column_flow() {
    let nodes = [
        UiNode::Element {
            tag: "p".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("first".to_owned())],
        },
        UiNode::Element {
            tag: "p".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("second".to_owned())],
        },
    ];
    let vertical_root = StyleRule {
        selectors: vec!["body".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "writing-mode".to_owned(),
            value: "vertical-rl".to_owned(),
        }],
    };
    let mut natural = PageRasterizer::new(&nodes, std::slice::from_ref(&vertical_root));
    natural.prepare_layout(300, 300);
    let natural_lines = natural.buffer.layout_runs().count();
    let stylesheet = [
        vertical_root,
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "block-size".to_owned(),
                value: "120px".to_owned(),
            }],
        },
    ];
    let mut sized = PageRasterizer::new(&nodes, &stylesheet);
    sized.prepare_layout(300, 300);

    assert!(sized.buffer.layout_runs().count() > natural_lines);
}

#[test]
fn empty_block_size_reserves_space_on_the_vertical_block_axis() {
    let nodes = [
        text_paragraph("before"),
        UiNode::Element {
            tag: "div".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: Vec::new(),
        },
        text_paragraph("after"),
    ];
    let stylesheet = [
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "writing-mode".to_owned(),
                value: "vertical-rl".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["div".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "block-size".to_owned(),
                value: "100px".to_owned(),
            }],
        },
    ];
    let mut natural = PageRasterizer::new(&nodes, &stylesheet[..1]);
    natural.prepare_layout(300, 300);
    let natural_columns = natural.buffer.layout_runs().count();
    let mut sized = PageRasterizer::new(&nodes, &stylesheet);
    sized.prepare_layout(300, 300);

    assert!(sized.buffer.layout_runs().count() > natural_columns);
}

#[test]
fn empty_block_height_reserves_vertical_flow_space() {
    let natural_after_y = empty_block_after_y(Vec::new());
    let zero_height_after_y = empty_block_after_y(vec![StyleDeclaration {
        property: "height".to_owned(),
        value: "0".to_owned(),
    }]);
    assert!((zero_height_after_y - natural_after_y).abs() < 1.0);

    for (property, value) in [
        ("height", "100px"),
        ("height", "25%"),
        ("min-height", "100px"),
        ("block-size", "100px"),
        ("min-block-size", "100px"),
    ] {
        let sized_after_y = empty_block_after_y(vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }]);
        assert!(
            sized_after_y >= natural_after_y + 80.0,
            "{property} should reserve the empty block height"
        );
    }

    for property in ["max-height", "max-block-size"] {
        let max_capped_after_y = empty_block_after_y(vec![
            StyleDeclaration {
                property: "height".to_owned(),
                value: "100px".to_owned(),
            },
            StyleDeclaration {
                property: property.to_owned(),
                value: "40px".to_owned(),
            },
        ]);
        assert!(
            max_capped_after_y >= natural_after_y + 20.0,
            "{property}: {max_capped_after_y} vs {natural_after_y}"
        );
        assert!(
            max_capped_after_y < natural_after_y + 60.0,
            "{property}: {max_capped_after_y} vs {natural_after_y}"
        );
    }
}

#[test]
fn content_block_min_height_reserves_remaining_vertical_space() {
    let natural_after_y = content_block_after_y(Vec::new());
    for property in ["min-height", "min-block-size"] {
        let sized_after_y = content_block_after_y(vec![StyleDeclaration {
            property: property.to_owned(),
            value: "140px".to_owned(),
        }]);
        assert!(
            sized_after_y >= natural_after_y + 100.0,
            "{property} should reserve space below the block's content"
        );
    }
    let short_min_height_after_y = content_block_after_y(vec![StyleDeclaration {
        property: "min-height".to_owned(),
        value: "10px".to_owned(),
    }]);
    assert!((short_min_height_after_y - natural_after_y).abs() < 1.0);
}

#[test]
fn content_block_height_reserves_space_and_max_height_caps_it() {
    let natural_after_y = content_block_after_y(Vec::new());
    let height_after_y = content_block_after_y(vec![StyleDeclaration {
        property: "height".to_owned(),
        value: "140px".to_owned(),
    }]);
    assert!(height_after_y >= natural_after_y + 100.0);

    let capped_height_after_y = content_block_after_y(vec![
        StyleDeclaration {
            property: "height".to_owned(),
            value: "200px".to_owned(),
        },
        StyleDeclaration {
            property: "max-height".to_owned(),
            value: "100px".to_owned(),
        },
    ]);
    let max_height_only_after_y = content_block_after_y(vec![StyleDeclaration {
        property: "height".to_owned(),
        value: "100px".to_owned(),
    }]);
    assert!((capped_height_after_y - max_height_only_after_y).abs() < 1.0);
}

#[test]
fn nested_content_block_min_heights_compose_in_flow() {
    let inner = UiNode::Element {
        tag: "section".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("content".to_owned())],
    };
    let outer = UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![inner],
    };
    let nodes = [text_paragraph("before"), outer, text_paragraph("after")];
    let stylesheet = [
        StyleRule {
            selectors: vec!["div".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "min-height".to_owned(),
                value: "180px".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["section".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "min-height".to_owned(),
                value: "100px".to_owned(),
            }],
        },
    ];
    let mut sized = PageRasterizer::new(&nodes, &stylesheet);
    let mut natural = PageRasterizer::new(&nodes, &[]);
    sized.prepare_layout(400, 500);
    natural.prepare_layout(400, 500);
    assert!(text_line_y(&sized, "after") >= text_line_y(&natural, "after") + 160.0);
}

fn content_block_after_y(declarations: Vec<StyleDeclaration>) -> f32 {
    let block = UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("content".to_owned())],
    };
    let nodes = [text_paragraph("before"), block, text_paragraph("after")];
    if declarations.is_empty() {
        let mut page = PageRasterizer::new(&nodes, &[]);
        page.prepare_layout(400, 400);
        return text_line_y(&page, "after");
    }
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations,
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(400, 400);
    text_line_y(&page, "after")
}

fn empty_block_after_y(declarations: Vec<StyleDeclaration>) -> f32 {
    let nodes = [
        text_paragraph("before"),
        UiNode::Element {
            tag: "div".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: Vec::new(),
        },
        text_paragraph("after"),
    ];
    if declarations.is_empty() {
        let mut page = PageRasterizer::new(&nodes, &[]);
        page.prepare_layout(400, 400);
        return text_line_y(&page, "after");
    }
    let stylesheet = [StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations,
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(400, 400);
    text_line_y(&page, "after")
}

pub(super) fn text_paragraph(text: &str) -> UiNode {
    UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text(text.to_owned())],
    }
}

pub(super) fn text_line_y(page: &PageRasterizer, text: &str) -> f32 {
    page.buffer
        .layout_runs()
        .find(|line| line.text.contains(text))
        .expect("text line should be in layout")
        .line_top
}
