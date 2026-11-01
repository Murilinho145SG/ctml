use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn flex_row_items_with_block_children_share_the_row() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("row".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![
            block_item("left", "Left top", "Left bottom"),
            block_item("right", "Right top", "Right bottom"),
        ],
    }];
    let stylesheet = [
        rule(".row", "display", "flex"),
        rule(".row", "gap", "20px"),
        rule(".left", "width", "50%"),
        rule(".right", "width", "40%"),
    ];

    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(400, 120);

    let left_top = glyph(page.layout_runs(), "Left top");
    let right_top = glyph(page.layout_runs(), "Right top");
    let left_bottom = glyph(page.layout_runs(), "Left bottom");
    let right_bottom = glyph(page.layout_runs(), "Right bottom");

    assert!((left_top.1 - right_top.1).abs() < 1.0);
    assert!((left_bottom.1 - right_bottom.1).abs() < 1.0);
    assert!(right_top.0 > left_top.0 + 100.0);
}

#[test]
fn nested_flex_rows_apply_columns_at_each_depth() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("row".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![
            block_item("left", "Left top", "Left bottom"),
            UiNode::Element {
                tag: "div".to_owned(),
                class: Some("right".to_owned()),
                id: None,
                attributes: Vec::new(),
                children: vec![
                    block_item("inner-a", "Inner A", "Inner A bottom"),
                    block_item("inner-b", "Inner B", "Inner B bottom"),
                ],
            },
        ],
    }];
    let stylesheet = [
        rule(".row", "display", "flex"),
        rule(".row", "gap", "20px"),
        rule(".left", "width", "40%"),
        rule(".right", "width", "40%"),
        rule(".inner-a", "width", "45%"),
        rule(".inner-b", "width", "45%"),
        rule(".right", "display", "flex"),
    ];

    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(600, 160);

    let inner_a = glyph(page.layout_runs(), "Inner A");
    let inner_b = glyph(page.layout_runs(), "Inner B");
    let left = glyph(page.layout_runs(), "Left top");

    assert!(inner_a.0 > left.0 + 100.0);
    assert!(inner_b.0 > inner_a.0 + 20.0);
    assert!((inner_a.1 - inner_b.1).abs() < 1.0);
}

fn block_item(class: &str, first: &str, second: &str) -> UiNode {
    UiNode::Element {
        tag: "div".to_owned(),
        class: Some(class.to_owned()),
        id: None,
        attributes: Vec::new(),
        children: [first, second]
            .into_iter()
            .map(|text| UiNode::Element {
                tag: "p".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text(text.to_owned())],
            })
            .collect(),
    }
}

fn rule(selector: &str, property: &str, value: &str) -> StyleRule {
    StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    }
}

fn glyph<'a>(runs: impl Iterator<Item = crate::page::PageLayoutRun<'a>>, text: &str) -> (f32, f32) {
    for run in runs {
        if let Some(start) = run.text.find(text) {
            let glyph = run
                .glyphs
                .iter()
                .find(|glyph| glyph.start == start)
                .expect("text should be laid out");
            return (glyph.x, run.line_top);
        }
    }
    panic!("{text} should be laid out");
}
