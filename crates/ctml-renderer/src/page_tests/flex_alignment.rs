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
fn row_justify_content_space_between_distributes_free_space_between_items() {
    let mut start = aligned_flex_page("row", "justify-content", "flex-start");
    let mut space_between = aligned_flex_page("row", "justify-content", "space-between");
    start.prepare_layout(400, 120);
    space_between.prepare_layout(400, 120);

    let start_gap = item_x(&start, "Second") - item_x(&start, "First");
    let distributed_gap = item_x(&space_between, "Second") - item_x(&space_between, "First");
    assert!(
        distributed_gap > start_gap + 150.0,
        "space-between should distribute the unused row width, observed {distributed_gap} vs {start_gap}"
    );
}

#[test]
fn row_justify_content_space_around_includes_leading_space() {
    let mut start = aligned_flex_page("row", "justify-content", "flex-start");
    let mut around = aligned_flex_page("row", "justify-content", "space-around");
    start.prepare_layout(400, 120);
    around.prepare_layout(400, 120);

    let start_x = item_x(&start, "First");
    let around_x = item_x(&around, "First");
    assert!(
        around_x > start_x + 20.0,
        "leading space: {around_x} vs {start_x}"
    );
    let around_gap = item_x(&around, "Second") - around_x;
    let start_gap = item_x(&start, "Second") - item_x(&start, "First");
    assert!(
        around_gap > start_gap + 40.0,
        "distributed gap: {around_gap} vs {start_gap}"
    );
}

#[test]
fn row_reverse_space_around_distributes_space_on_both_ends() {
    let mut page = aligned_flex_page("row-reverse", "justify-content", "space-around");
    page.prepare_layout(400, 120);

    let first = item_x(&page, "First");
    let second = item_x(&page, "Second");
    assert!(first > second, "row-reverse should reverse item order");
    assert!(
        second > 20.0,
        "space-around should leave leading space: {second}"
    );
    assert!(
        first < 360.0,
        "space-around should leave trailing space: {first}"
    );
}

#[test]
fn row_reverse_space_between_keeps_the_first_source_item_at_the_main_start() {
    let mut page = aligned_flex_page("row-reverse", "justify-content", "space-between");
    page.prepare_layout(400, 120);

    assert!(
        item_x(&page, "First") > item_x(&page, "Second") + 150.0,
        "space-between should span the row while row-reverse places the first source item at the right"
    );
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
