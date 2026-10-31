use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn flex_grow_distributes_free_space_by_item_weight() {
    let mut first_equal = flex_grow_page("1", false);
    let mut first_weighted = flex_grow_page("3", false);
    first_equal.prepare_layout(400, 120);
    first_weighted.prepare_layout(400, 120);

    let equal_start = item_x(&first_equal, "Second");
    let weighted_start = item_x(&first_weighted, "Second");
    assert!(
        weighted_start > equal_start + 40.0,
        "the first item should receive more free space at a 3:1 grow ratio; equal={equal_start}, weighted={weighted_start}"
    );
}

#[test]
fn flex_grow_does_not_expand_an_auto_sized_inline_flex_container() {
    let mut no_grow = flex_grow_page("0", true);
    let mut grow = flex_grow_page("3", true);
    no_grow.prepare_layout(400, 120);
    grow.prepare_layout(400, 120);
    assert!((item_x(&grow, "Second") - item_x(&no_grow, "Second")).abs() < 1.0);
}

fn flex_grow_page(first_grow: &str, inline: bool) -> PageRasterizer {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("container".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: [("first", "First"), ("second", "Second")]
            .into_iter()
            .map(|(class, text)| UiNode::Element {
                tag: "p".to_owned(),
                class: Some(class.to_owned()),
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text(text.to_owned())],
            })
            .collect(),
    }];
    let rule = |selector: &str, declarations: &[(&str, &str)]| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: declarations
            .iter()
            .map(|(property, value)| StyleDeclaration {
                property: (*property).to_owned(),
                value: (*value).to_owned(),
            })
            .collect(),
    };
    let stylesheet = [
        rule(
            ".container",
            &[("display", if inline { "inline-flex" } else { "flex" })],
        ),
        rule(".first", &[("flex-grow", first_grow)]),
        rule(".second", &[("flex-grow", "1")]),
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
