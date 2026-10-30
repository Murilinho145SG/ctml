use super::sizing_vertical::text_paragraph;
use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

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
