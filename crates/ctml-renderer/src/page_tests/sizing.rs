use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn body_width_constrains_text_wrapping() {
    let text = "a sequence of words that takes several lines when the body is narrow ".repeat(8);
    let narrow_line_count = line_count_with_rule("width", "80px", text.clone(), 400);
    let wide_line_count = line_count_without_rule(text, 400);

    assert!(
        narrow_line_count > wide_line_count,
        "narrow body should wrap to more lines ({narrow_line_count} vs {wide_line_count})"
    );
}

#[test]
fn lh_unit_resolves_against_inherited_line_height() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Line height units".to_owned())],
    }];
    let make_page = |paragraph_declaration: (&str, &str)| {
        PageRasterizer::new(
            &nodes,
            &[
                StyleRule {
                    selectors: vec!["body".to_owned()],
                    layer: None,
                    declarations: vec![StyleDeclaration {
                        property: "line-height".to_owned(),
                        value: "24px".to_owned(),
                    }],
                },
                StyleRule {
                    selectors: vec!["p".to_owned()],
                    layer: None,
                    declarations: vec![StyleDeclaration {
                        property: paragraph_declaration.0.to_owned(),
                        value: paragraph_declaration.1.to_owned(),
                    }],
                },
            ],
        )
    };

    let font_size_page = make_page(("font-size", "2lh"));
    assert_eq!(font_size_page.runs[0].style.font_size, 48.0);

    let line_height_page = make_page(("line-height", "2lh"));
    assert_eq!(line_height_page.runs[0].style.line_height, 48.0);

    let font_shorthand_page = make_page(("font", "12px/2lh serif"));
    assert_eq!(font_shorthand_page.runs[0].style.font_size, 12.0);
    assert_eq!(font_shorthand_page.runs[0].style.line_height, 48.0);
}

#[test]
fn font_size_metric_units_use_the_parent_typeface() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Metric sized text".to_owned())],
    }];
    for unit in ["ex", "ch", "ic", "cap"] {
        let stylesheet = [
            StyleRule {
                selectors: vec!["body".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: "font-size".to_owned(),
                    value: "20px".to_owned(),
                }],
            },
            StyleRule {
                selectors: vec!["p".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: "font-size".to_owned(),
                    value: format!("2{unit}"),
                }],
            },
        ];
        let page = PageRasterizer::new(&nodes, &stylesheet);
        let metrics = page.root_style.font_relative_units;
        let factor = match unit {
            "ex" => metrics.ex,
            "ch" => metrics.ch,
            "ic" => metrics.ic,
            "cap" => metrics.cap,
            _ => unreachable!(),
        };
        let expected_size = 40.0 * factor;
        assert!(
            (page.runs[0].style.font_size - expected_size).abs() < 0.01,
            "{unit} should resolve using parent font metrics"
        );
    }
}

#[test]
fn font_shorthand_resolves_metric_sized_font_from_parent_typeface() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Metric shorthand".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-size".to_owned(),
                value: "20px".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font".to_owned(),
                value: "2ch serif".to_owned(),
            }],
        },
    ];
    let page = PageRasterizer::new(&nodes, &stylesheet);
    let expected_size = 40.0 * page.root_style.font_relative_units.ch;
    assert!((page.runs[0].style.font_size - expected_size).abs() < 0.01);
}

#[test]
fn first_letter_font_size_metric_units_use_the_originating_font() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Metric first letter".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-letter".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "font-size".to_owned(),
            value: "2ex".to_owned(),
        }],
    }];
    let page = PageRasterizer::new(&nodes, &stylesheet);
    let expected_size = 32.0 * page.runs[0].style.font_relative_units.ex;
    assert!((page.runs[0].style.font_size - expected_size).abs() < 0.01);
}

#[test]
fn body_min_and_max_width_constrain_text_wrapping() {
    let text = "words that wrap when the body width is constrained ".repeat(8);
    let min_width_lines = line_count_with_rule("min-width", "800px", text.clone(), 160);
    let min_width_reference = line_count_without_rule(text.clone(), 160);
    let max_width_lines = line_count_with_rule("max-width", "80px", text.clone(), 400);
    let wide_lines = line_count_without_rule(text, 400);

    assert!(min_width_lines < min_width_reference);
    assert!(max_width_lines > wide_lines);
}

#[test]
fn logical_inline_size_properties_use_the_horizontal_width_axis() {
    let text = "logical inline size aliases constrain text wrapping ".repeat(8);
    let inline_size_lines = line_count_with_rule("inline-size", "80px", text.clone(), 400);
    let min_inline_size_lines = line_count_with_rule("min-inline-size", "800px", text.clone(), 160);
    let min_reference_lines = line_count_without_rule(text.clone(), 160);
    let max_inline_size_lines = line_count_with_rule("max-inline-size", "80px", text.clone(), 400);
    let wide_lines = line_count_without_rule(text, 400);

    assert!(inline_size_lines > wide_lines);
    assert!(min_inline_size_lines < min_reference_lines);
    assert!(max_inline_size_lines > wide_lines);
}

#[test]
fn vertical_writing_uses_inline_size_on_the_column_flow_axis() {
    let text = "logical vertical size constrains column text flow ".repeat(10);
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text(text)],
    }];
    let vertical_root = StyleRule {
        selectors: vec!["body".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "writing-mode".to_owned(),
            value: "vertical-rl".to_owned(),
        }],
    };
    let mut unconstrained = PageRasterizer::new(&nodes, std::slice::from_ref(&vertical_root));
    unconstrained.prepare_layout(1000, 500);
    let unconstrained_lines = unconstrained.buffer.layout_runs().count();

    let root_with_inline_size = [
        vertical_root.clone(),
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "inline-size".to_owned(),
                value: "40px".to_owned(),
            }],
        },
    ];
    let mut root_sized = PageRasterizer::new(&nodes, &root_with_inline_size);
    root_sized.prepare_layout(1000, 500);
    let root_sized_lines = root_sized.buffer.layout_runs().count();
    assert!(
        root_sized_lines > unconstrained_lines,
        "root inline-size should constrain columns ({root_sized_lines} vs {unconstrained_lines}, {:?})",
        root_sized.root_style.inline_size
    );

    let nested_with_inline_size = [
        vertical_root,
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "inline-size".to_owned(),
                value: "40px".to_owned(),
            }],
        },
    ];
    let mut nested_sized = PageRasterizer::new(&nodes, &nested_with_inline_size);
    nested_sized.prepare_layout(1000, 500);
    assert!(nested_sized.buffer.layout_runs().count() > unconstrained_lines);
}

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

#[test]
fn block_element_width_constrains_nested_text_wrapping() {
    let text = "words inside an inline child wrap to fit the parent block ".repeat(8);
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "section".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Element {
                tag: "span".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text(text)],
            }],
        }],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "width".to_owned(),
            value: "160px".to_owned(),
        }],
    }];
    let nested_stylesheet = [
        stylesheet[0].clone(),
        StyleRule {
            selectors: vec!["section".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "inline-size".to_owned(),
                value: "50%".to_owned(),
            }],
        },
    ];
    let mut parent_width = PageRasterizer::new(&nodes, &stylesheet);
    let mut wide = PageRasterizer::new(&nodes, &[]);
    let mut nested_width = PageRasterizer::new(&nodes, &nested_stylesheet);
    parent_width.prepare_layout(400, 1200);
    wide.prepare_layout(400, 1200);
    nested_width.prepare_layout(400, 1200);

    let parent_line_count = parent_width.buffer.layout_runs().count();
    let wide_line_count = wide.buffer.layout_runs().count();
    let nested_line_count = nested_width.buffer.layout_runs().count();
    assert!(parent_line_count > wide_line_count);
    assert!(nested_line_count > parent_line_count);
}

#[test]
fn nested_vertical_block_uses_block_size_for_page_inline_extent() {
    let text = "a sequence of words within a nested vertical block ".repeat(8);
    let nodes = [UiNode::Element {
        tag: "section".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text(text)],
    }];
    let vertical_block = [StyleRule {
        selectors: vec!["section".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "writing-mode".to_owned(),
                value: "vertical-rl".to_owned(),
            },
            StyleDeclaration {
                property: "block-size".to_owned(),
                value: "160px".to_owned(),
            },
        ],
    }];
    let mut natural = PageRasterizer::new(&nodes, &[]);
    let mut constrained = PageRasterizer::new(&nodes, &vertical_block);
    natural.prepare_layout(400, 1200);
    constrained.prepare_layout(400, 1200);

    assert!(
        constrained.buffer.layout_runs().count() > natural.buffer.layout_runs().count(),
        "a vertical child block's block-size maps to the page's horizontal text extent"
    );
}

#[test]
fn nested_vertical_inline_size_reserves_page_block_extent() {
    let nodes = [
        text_paragraph("before"),
        UiNode::Element {
            tag: "section".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("vertical child content".to_owned())],
        },
        text_paragraph("after"),
    ];
    let mut natural = PageRasterizer::new(&nodes, &[]);
    natural.prepare_layout(300, 500);
    let natural_lines = natural.buffer.layout_runs().count();
    let stylesheet = [StyleRule {
        selectors: vec!["section".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "writing-mode".to_owned(),
                value: "vertical-rl".to_owned(),
            },
            StyleDeclaration {
                property: "inline-size".to_owned(),
                value: "120px".to_owned(),
            },
        ],
    }];
    let mut sized = PageRasterizer::new(&nodes, &stylesheet);
    sized.prepare_layout(300, 500);

    assert!(
        sized.buffer.layout_runs().count() > natural_lines,
        "nested vertical inline-size should reserve the page's block-axis extent"
    );
}

#[test]
fn nested_vertical_physical_and_logical_sizes_follow_source_order() {
    let text = "source order chooses the nested vertical block extent ".repeat(12);
    let nodes = [UiNode::Element {
        tag: "section".to_owned(),
        class: None,
        id: None,
        attributes: vec![],
        children: vec![UiNode::Text(text)],
    }];
    let make_page = |first: (&str, &str), second: (&str, &str)| {
        let mut page = PageRasterizer::new(
            &nodes,
            &[StyleRule {
                selectors: vec!["section".to_owned()],
                layer: None,
                declarations: vec![
                    StyleDeclaration {
                        property: "writing-mode".to_owned(),
                        value: "vertical-rl".to_owned(),
                    },
                    StyleDeclaration {
                        property: first.0.to_owned(),
                        value: first.1.to_owned(),
                    },
                    StyleDeclaration {
                        property: second.0.to_owned(),
                        value: second.1.to_owned(),
                    },
                ],
            }],
        );
        page.prepare_layout(400, 1000);
        page.buffer.layout_runs().count()
    };

    let logical_last = make_page(("width", "160px"), ("block-size", "80px"));
    let physical_last = make_page(("block-size", "80px"), ("width", "160px"));
    assert!(logical_last > physical_last);
}

fn line_count_with_rule(property: &str, value: &str, text: String, viewport_width: u32) -> usize {
    let nodes = [UiNode::Text(text)];
    let stylesheet = [StyleRule {
        selectors: vec!["body".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(viewport_width, 300);
    let line_count = page.buffer.layout_runs().count();
    line_count
}

fn line_count_without_rule(text: String, viewport_width: u32) -> usize {
    let nodes = [UiNode::Text(text)];
    let mut page = PageRasterizer::new(&nodes, &[]);
    page.prepare_layout(viewport_width, 300);
    page.buffer.layout_runs().count()
}
