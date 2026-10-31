use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn display_flex_places_block_children_in_a_shared_row() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("container".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: ["First item", "Second item"]
            .into_iter()
            .map(|text| UiNode::Element {
                tag: "p".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text(text.to_owned())],
            })
            .collect(),
    }];
    let stylesheet = [StyleRule {
        selectors: vec![".container".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "display".to_owned(),
            value: "flex".to_owned(),
        }],
    }];

    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(400, 120);

    assert_eq!(page.buffer.layout_runs().count(), 1);
}

#[test]
fn flex_direction_column_stacks_inline_children_as_items() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("container".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: ["First item", "Second item"]
            .into_iter()
            .map(|text| UiNode::Element {
                tag: "span".to_owned(),
                class: None,
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
            selectors: vec![".container".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "flex-direction".to_owned(),
                value: "column".to_owned(),
            }],
        },
    ];

    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(400, 120);

    assert_eq!(page.buffer.layout_runs().count(), 2);
}

#[test]
fn column_gap_adds_inline_advance_between_row_items() {
    let mut without_gap = two_item_flex_page("row", None);
    let mut with_gap = two_item_flex_page("row", Some("12px"));
    without_gap.prepare_layout(400, 120);
    with_gap.prepare_layout(400, 120);

    let baseline = second_item_x(&without_gap) - first_item_x(&without_gap);
    let spaced = second_item_x(&with_gap) - first_item_x(&with_gap);
    assert!(
        (spaced - baseline - 12.0).abs() < 1.0,
        "gap advance was {}",
        spaced - baseline
    );
}

#[test]
fn percentage_column_gap_uses_the_flex_container_inline_size() {
    let mut without_gap = two_item_flex_page("row", None);
    let mut page = two_item_flex_page("row", Some("10%"));
    without_gap.prepare_layout(400, 120);
    page.prepare_layout(400, 120);

    let baseline_advance = second_item_x(&without_gap) - first_item_x(&without_gap);
    let spaced_advance = second_item_x(&page) - first_item_x(&page);
    assert!(
        (spaced_advance - baseline_advance - 40.0).abs() < 1.0,
        "10% column gap should resolve to 40px, observed {}",
        spaced_advance - baseline_advance
    );
}

#[test]
fn percentage_gap_stays_zero_in_an_inline_flex_with_auto_main_size() {
    let mut without_gap = inline_row_flex_page(None);
    let mut with_gap = inline_row_flex_page(Some("10%"));
    let mut with_calc = inline_row_flex_page(Some("calc(10% + 3px)"));
    without_gap.prepare_layout(400, 120);
    with_gap.prepare_layout(400, 120);
    with_calc.prepare_layout(400, 120);

    let baseline_advance = second_item_x(&without_gap) - first_item_x(&without_gap);
    let spaced_advance = second_item_x(&with_gap) - first_item_x(&with_gap);
    assert!(
        (spaced_advance - baseline_advance).abs() < 1.0,
        "percentage gap on an auto-sized inline flex container should be zero, observed {}",
        spaced_advance - baseline_advance
    );
    let calc_advance = second_item_x(&with_calc) - first_item_x(&with_calc);
    assert!(
        (calc_advance - baseline_advance - 3.0).abs() < 1.0,
        "indefinite percentage should resolve to zero while calc length remains, observed {}",
        calc_advance - baseline_advance
    );
}

#[test]
fn row_gap_adds_vertical_flow_between_column_items() {
    let mut without_gap = two_item_flex_page("column", None);
    let mut with_gap = two_item_flex_page("column", Some("12px"));
    without_gap.prepare_layout(200, 200);
    with_gap.prepare_layout(200, 200);

    let baseline = second_item_y(&without_gap) - first_item_y(&without_gap);
    let spaced = second_item_y(&with_gap) - first_item_y(&with_gap);
    assert!(
        (spaced - baseline - 12.0).abs() < 1.0,
        "gap advance was {}",
        spaced - baseline
    );
}

#[test]
fn flex_direction_column_reverse_reverses_item_order_and_keeps_row_gap() {
    let mut without_gap = two_item_flex_page("column-reverse", None);
    let mut with_gap = two_item_flex_page("column-reverse", Some("12px"));
    without_gap.prepare_layout(200, 200);
    with_gap.prepare_layout(200, 200);

    assert!(
        first_item_y(&without_gap) > second_item_y(&without_gap),
        "column-reverse should place the first source item after the second"
    );
    let baseline = first_item_y(&without_gap) - second_item_y(&without_gap);
    let spaced = first_item_y(&with_gap) - second_item_y(&with_gap);
    assert!(
        (spaced - baseline - 12.0).abs() < 1.0,
        "row-gap should keep its size in column-reverse, observed {}",
        spaced - baseline
    );
}

#[test]
fn percentage_row_gap_uses_an_explicit_flex_container_block_size() {
    let mut without_gap = two_item_flex_page_with_height("column", None, None);
    let mut with_gap = two_item_flex_page_with_height("column", Some("10%"), Some("200px"));
    without_gap.prepare_layout(200, 200);
    with_gap.prepare_layout(200, 200);

    let baseline = second_item_y(&without_gap) - first_item_y(&without_gap);
    let spaced = second_item_y(&with_gap) - first_item_y(&with_gap);
    assert!(
        (spaced - baseline - 20.0).abs() < 1.0,
        "10% row gap should resolve to 20px, observed {}",
        spaced - baseline
    );
}

#[test]
fn row_gap_does_not_split_contiguous_anonymous_text_items() {
    let mut without_gap = anonymous_text_flex_page(None);
    let mut with_gap = anonymous_text_flex_page(Some("12px"));
    without_gap.prepare_layout(400, 120);
    with_gap.prepare_layout(400, 120);

    let baseline_text_advance = glyph_x(&without_gap, "B") - glyph_x(&without_gap, "A");
    let spaced_text_advance = glyph_x(&with_gap, "B") - glyph_x(&with_gap, "A");
    assert!((spaced_text_advance - baseline_text_advance).abs() < 1.0);

    let baseline_element_advance = glyph_x(&without_gap, "C") - glyph_x(&without_gap, "B");
    let spaced_element_advance = glyph_x(&with_gap, "C") - glyph_x(&with_gap, "B");
    assert!((spaced_element_advance - baseline_element_advance - 12.0).abs() < 1.0);
}

fn two_item_flex_page(direction: &str, gap: Option<&str>) -> PageRasterizer {
    two_item_flex_page_with_height(direction, gap, None)
}

fn inline_row_flex_page(gap: Option<&str>) -> PageRasterizer {
    let nodes = [UiNode::Element {
        tag: "span".to_owned(),
        class: Some("container".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: ["First", "Second"]
            .into_iter()
            .map(|text| UiNode::Element {
                tag: "span".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text(text.to_owned())],
            })
            .collect(),
    }];
    let mut declarations = vec![StyleDeclaration {
        property: "display".to_owned(),
        value: "inline-flex".to_owned(),
    }];
    if let Some(gap) = gap {
        declarations.push(StyleDeclaration {
            property: "gap".to_owned(),
            value: gap.to_owned(),
        });
    }
    let stylesheet = [StyleRule {
        selectors: vec![".container".to_owned()],
        layer: None,
        declarations,
    }];
    PageRasterizer::new(&nodes, &stylesheet)
}

fn two_item_flex_page_with_height(
    direction: &str,
    gap: Option<&str>,
    height: Option<&str>,
) -> PageRasterizer {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("container".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: ["First", "Second"]
            .into_iter()
            .map(|text| UiNode::Element {
                tag: "p".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text(text.to_owned())],
            })
            .collect(),
    }];
    let mut declarations = vec![
        StyleDeclaration {
            property: "display".to_owned(),
            value: "flex".to_owned(),
        },
        StyleDeclaration {
            property: "flex-direction".to_owned(),
            value: direction.to_owned(),
        },
    ];
    if let Some(gap) = gap {
        declarations.push(StyleDeclaration {
            property: "gap".to_owned(),
            value: gap.to_owned(),
        });
    }
    if let Some(height) = height {
        declarations.push(StyleDeclaration {
            property: "height".to_owned(),
            value: height.to_owned(),
        });
    }
    let stylesheet = [StyleRule {
        selectors: vec![".container".to_owned()],
        layer: None,
        declarations,
    }];
    PageRasterizer::new(&nodes, &stylesheet)
}

fn anonymous_text_flex_page(gap: Option<&str>) -> PageRasterizer {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("container".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![
            UiNode::Text("A".to_owned()),
            UiNode::Text("B".to_owned()),
            UiNode::Element {
                tag: "span".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text("C".to_owned())],
            },
        ],
    }];
    let mut declarations = vec![StyleDeclaration {
        property: "display".to_owned(),
        value: "flex".to_owned(),
    }];
    if let Some(gap) = gap {
        declarations.push(StyleDeclaration {
            property: "gap".to_owned(),
            value: gap.to_owned(),
        });
    }
    let stylesheet = [StyleRule {
        selectors: vec![".container".to_owned()],
        layer: None,
        declarations,
    }];
    PageRasterizer::new(&nodes, &stylesheet)
}

fn glyph_x(page: &PageRasterizer, text: &str) -> f32 {
    item_glyph(page, text).x
}

fn item_glyph<'a>(page: &'a PageRasterizer, item: &str) -> &'a cosmic_text::LayoutGlyph {
    let line = page
        .buffer
        .layout_runs()
        .find(|line| line.text.contains(item))
        .expect("flex item should be in a formatted line");
    let start = line
        .text
        .find(item)
        .expect("item text should have an index");
    line.glyphs
        .iter()
        .find(|glyph| glyph.start == start)
        .expect("first item glyph should be laid out")
}

fn first_item_x(page: &PageRasterizer) -> f32 {
    item_glyph(page, "First").x
}

fn second_item_x(page: &PageRasterizer) -> f32 {
    item_glyph(page, "Second").x
}

fn first_item_y(page: &PageRasterizer) -> f32 {
    page.buffer
        .layout_runs()
        .find(|line| line.text.contains("First"))
        .expect("first item should be on a formatted line")
        .line_top
}

fn second_item_y(page: &PageRasterizer) -> f32 {
    page.buffer
        .layout_runs()
        .find(|line| line.text.contains("Second"))
        .expect("second item should be on a formatted line")
        .line_top
}

#[path = "flex_order.rs"]
mod flex_order;

#[path = "flex_grow.rs"]
mod flex_grow;

#[path = "flex_shrink.rs"]
mod flex_shrink;

#[path = "flex_basis.rs"]
mod flex_basis;
