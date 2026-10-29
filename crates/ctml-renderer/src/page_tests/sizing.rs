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
                property: "width".to_owned(),
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
