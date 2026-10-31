use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn flex_shrink_reduces_main_axis_advance_when_items_overflow() {
    let mut shrinking = flex_shrink_page("1");
    let mut fixed = flex_shrink_page("0");
    shrinking.prepare_layout(200, 120);
    fixed.prepare_layout(200, 120);

    let shrinking_second = item_x(&shrinking, "Second");
    let fixed_second = item_x(&fixed, "Second");
    assert!(
        shrinking_second + 30.0 < fixed_second,
        "flex-shrink should reduce the advance before the second item; shrinking={shrinking_second}, fixed={fixed_second}"
    );
}

fn flex_shrink_page(shrink: &str) -> PageRasterizer {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("container".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: ["MMMMMMMMMMMMMMMMMMMM", "Second"]
            .into_iter()
            .map(|text| UiNode::Element {
                tag: "p".to_owned(),
                class: Some("item".to_owned()),
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text(text.to_owned())],
            })
            .collect(),
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec![".container".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "display".to_owned(),
                value: "flex".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".item".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "flex-shrink".to_owned(),
                value: shrink.to_owned(),
            }],
        },
    ];
    PageRasterizer::new(&nodes, &stylesheet)
}

fn item_x(page: &PageRasterizer, text: &str) -> f32 {
    let line = page
        .buffer
        .layout_runs()
        .find(|line| line.text.contains(text))
        .expect("flex item should have a formatted line");
    let start = line
        .text
        .find(text)
        .expect("flex item text should have an offset");
    line.glyphs
        .iter()
        .find(|glyph| glyph.start == start)
        .expect("flex item should have a first glyph")
        .x
}
