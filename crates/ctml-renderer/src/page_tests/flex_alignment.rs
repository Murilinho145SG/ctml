use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn row_justify_content_aligns_flex_items_on_the_main_axis() {
    let mut start = aligned_flex_page("row", "justify-content", "flex-start");
    let mut center = aligned_flex_page("row", "justify-content", "center");
    let mut end = aligned_flex_page("row", "justify-content", "flex-end");
    for page in [&mut start, &mut center, &mut end] {
        page.prepare_layout(400, 120);
    }

    let start_x = item_x(&start, "First");
    let center_x = item_x(&center, "First");
    let end_x = item_x(&end, "First");
    assert!(center_x > start_x + 20.0, "center offset: {center_x}");
    assert!(end_x > center_x + 20.0, "end offset: {end_x}");
    for page in [&start, &center, &end] {
        assert!(
            item_x(page, "Second") > item_x(page, "First"),
            "items should keep source order"
        );
    }
}

#[test]
fn row_reverse_places_the_first_flex_item_at_the_main_end() {
    let mut page = aligned_flex_page("row-reverse", "justify-content", "flex-start");
    let mut flex_end = aligned_flex_page("row-reverse", "justify-content", "flex-end");
    page.prepare_layout(400, 120);
    flex_end.prepare_layout(400, 120);

    assert!(
        item_x(&page, "First") > item_x(&page, "Second"),
        "row-reverse should place the first item at the right side"
    );
    assert!(
        item_x(&flex_end, "First") > item_x(&flex_end, "Second"),
        "flex-end should retain the reversed item sequence"
    );
    assert!(
        item_x(&page, "First") > item_x(&flex_end, "First") + 100.0,
        "flex-end should move the row-reverse group to the left side"
    );
}

#[test]
fn column_align_items_aligns_flex_items_on_the_cross_axis() {
    let mut start = aligned_flex_page("column", "align-items", "flex-start");
    let mut center = aligned_flex_page("column", "align-items", "center");
    let mut end = aligned_flex_page("column", "align-items", "flex-end");
    for page in [&mut start, &mut center, &mut end] {
        page.prepare_layout(400, 120);
    }

    let start_x = item_x(&start, "First");
    let center_x = item_x(&center, "First");
    let end_x = item_x(&end, "First");
    assert!(center_x > start_x + 20.0, "center offset: {center_x}");
    assert!(end_x > center_x + 20.0, "end offset: {end_x}");
    for page in [&start, &center, &end] {
        assert!(
            item_y(page, "Second") > item_y(page, "First"),
            "column items should keep source order"
        );
    }
}

#[test]
fn align_self_overrides_the_column_container_alignment_per_item() {
    let mut center = align_self_flex_page("center");
    let mut end = align_self_flex_page("flex-end");
    center.prepare_layout(400, 120);
    end.prepare_layout(400, 120);

    let first_x = item_x(&center, "First");
    let center_x = item_x(&center, "Second");
    let end_x = item_x(&end, "Second");
    assert!(center_x > first_x + 20.0, "center offset: {center_x}");
    assert!(end_x > center_x + 20.0, "end offset: {end_x}");
    assert!(item_y(&center, "Second") > item_y(&center, "First"));
}
fn aligned_flex_page(direction: &str, property: &str, value: &str) -> PageRasterizer {
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
    let stylesheet = [StyleRule {
        selectors: vec![".container".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "display".to_owned(),
                value: "flex".to_owned(),
            },
            StyleDeclaration {
                property: "flex-direction".to_owned(),
                value: direction.to_owned(),
            },
            StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            },
        ],
    }];
    PageRasterizer::new(&nodes, &stylesheet)
}

fn align_self_flex_page(value: &str) -> PageRasterizer {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("container".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: ["First", "Second"]
            .into_iter()
            .enumerate()
            .map(|(index, text)| UiNode::Element {
                tag: "p".to_owned(),
                class: (index == 1).then(|| "second".to_owned()),
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
            declarations: vec![
                StyleDeclaration {
                    property: "display".to_owned(),
                    value: "flex".to_owned(),
                },
                StyleDeclaration {
                    property: "flex-direction".to_owned(),
                    value: "column".to_owned(),
                },
                StyleDeclaration {
                    property: "align-items".to_owned(),
                    value: "flex-start".to_owned(),
                },
            ],
        },
        StyleRule {
            selectors: vec![".second".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "align-self".to_owned(),
                value: value.to_owned(),
            }],
        },
    ];
    PageRasterizer::new(&nodes, &stylesheet)
}

fn item_x(page: &PageRasterizer, item: &str) -> f32 {
    item_glyph(page, item).x
}

fn item_y(page: &PageRasterizer, item: &str) -> f32 {
    page.buffer
        .layout_runs()
        .find(|line| line.text.contains(item))
        .expect("flex item should be on a formatted line")
        .line_top
}

fn item_glyph<'a>(page: &'a PageRasterizer, item: &str) -> &'a cosmic_text::LayoutGlyph {
    let line = page
        .buffer
        .layout_runs()
        .find(|line| line.text.contains(item))
        .expect("flex item should be on a formatted line");
    let start = line
        .text
        .find(item)
        .expect("flex item should have text in its line");
    line.glyphs
        .iter()
        .find(|glyph| glyph.start == start)
        .expect("first flex item glyph should be laid out")
}
