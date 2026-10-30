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
