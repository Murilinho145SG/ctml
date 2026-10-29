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
        assert!(max_capped_after_y >= natural_after_y + 20.0);
        assert!(max_capped_after_y < natural_after_y + 60.0);
    }
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
