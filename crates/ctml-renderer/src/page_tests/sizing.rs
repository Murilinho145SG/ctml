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
