use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn flex_shrink_respects_automatic_min_content_size() {
    let mut shrinking = flex_shrink_page("1", None, "MMMMMMMMMM MMMMMMMMMM");
    let mut fixed = flex_shrink_page("0", None, "MMMMMMMMMM MMMMMMMMMM");
    shrinking.prepare_layout(200, 120);
    fixed.prepare_layout(200, 120);

    let shrinking_second = item_x(&shrinking, "Second");
    let fixed_second = item_x(&fixed, "Second");
    assert!(
        shrinking_second + 30.0 < fixed_second,
        "flex-shrink should reduce the advance to the longest word without shrinking below it; shrinking={shrinking_second}, fixed={fixed_second}"
    );
}

#[test]
fn explicit_zero_min_width_allows_shrinking_below_a_long_word() {
    let mut automatic = flex_shrink_page("1", None, "MMMMMMMMMMMMMMMMMMMM");
    let mut zero_minimum = flex_shrink_page("1", Some("0"), "MMMMMMMMMMMMMMMMMMMM");
    automatic.prepare_layout(200, 120);
    zero_minimum.prepare_layout(200, 120);

    assert!(
        item_x(&zero_minimum, "Second") + 30.0 < item_x(&automatic, "Second"),
        "min-width: 0 should allow shrinking below the automatic min-content size"
    );
}

fn flex_shrink_page(shrink: &str, min_width: Option<&str>, first_item: &str) -> PageRasterizer {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("container".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: [first_item, "Second"]
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
    let mut stylesheet = vec![
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
    if let Some(min_width) = min_width {
        stylesheet.push(StyleRule {
            selectors: vec![".item".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "min-width".to_owned(),
                value: min_width.to_owned(),
            }],
        });
    }
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
