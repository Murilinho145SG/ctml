use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn root_intrinsic_width_keywords_measure_text_min_content_and_max_content_sizes() {
    let text = "unbreakabletoken small word ".repeat(5);
    let min_content_lines = line_count_with_rule("body", "width", "min-content", text.clone());
    let max_content_lines = line_count_with_rule("body", "width", "max-content", text.clone());
    let fit_content_lines = line_count_with_rule("body", "width", "fit-content(320px)", text);

    assert!(
        min_content_lines > fit_content_lines,
        "min-content should wrap at the longest word ({min_content_lines} vs {fit_content_lines})"
    );
    assert_eq!(
        max_content_lines, 1,
        "max-content should keep each paragraph on one line"
    );
    assert!(
        fit_content_lines > max_content_lines,
        "fit-content should clamp to its 320px argument ({fit_content_lines} vs {max_content_lines})"
    );
}

#[test]
fn nested_intrinsic_width_keywords_measure_their_own_text() {
    let text = "unbreakabletoken small word ".repeat(5);
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text(text)],
    }];
    let line_count = |width: &str| {
        let mut page = PageRasterizer::new(
            &nodes,
            &[StyleRule {
                selectors: vec!["p".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: "width".to_owned(),
                    value: width.to_owned(),
                }],
            }],
        );
        page.prepare_layout(400, 300);
        assert!(page.runs[0].style.layout_sizes.iter().any(|size| {
            size.intrinsic_min_content_width.is_some() && size.intrinsic_max_content_width.is_some()
        }));
        page.buffer.layout_runs().count()
    };

    let min_content_lines = line_count("min-content");
    let max_content_lines = line_count("max-content");
    assert!(
        min_content_lines > max_content_lines,
        "nested min-content should be narrower than max-content ({min_content_lines} vs {max_content_lines})"
    );
}

fn line_count_with_rule(selector: &str, property: &str, value: &str, text: String) -> usize {
    let nodes = [UiNode::Text(text)];
    let stylesheet = [StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(400, 300);
    page.buffer.layout_runs().count()
}
