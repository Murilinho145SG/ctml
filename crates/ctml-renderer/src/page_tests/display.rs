use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn run_in_box_joins_the_following_block_line() {
    let nodes = [
        UiNode::Element {
            tag: "h2".to_owned(),
            class: Some("run-in".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("Heading".to_owned())],
        },
        UiNode::Element {
            tag: "p".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("followed by paragraph".to_owned())],
        },
    ];
    let display_rule = |display: &str| StyleRule {
        selectors: vec![".run-in".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "display".to_owned(),
            value: display.to_owned(),
        }],
    };

    let mut run_in = PageRasterizer::new(&nodes, &[display_rule("run-in")]);
    run_in.prepare_layout(400, 120);
    let run_in_lines = run_in.buffer.layout_runs().count();

    let mut block = PageRasterizer::new(&nodes, &[display_rule("block")]);
    block.prepare_layout(400, 120);
    let block_lines = block.buffer.layout_runs().count();

    assert_eq!(run_in_lines, 1);
    assert_eq!(block_lines, 2);

    let mut inline_sibling = PageRasterizer::new(
        &nodes,
        &[
            display_rule("run-in"),
            StyleRule {
                selectors: vec!["p".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: "display".to_owned(),
                    value: "inline".to_owned(),
                }],
            },
        ],
    );
    inline_sibling.prepare_layout(400, 120);
    assert_eq!(inline_sibling.buffer.layout_runs().count(), 2);
}
