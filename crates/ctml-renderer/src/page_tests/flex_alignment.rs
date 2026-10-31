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
fn row_justify_content_space_evenly_distributes_equal_edge_and_item_gaps() {
    let mut page = aligned_flex_page("row", "justify-content", "space-evenly");
    page.prepare_layout(800, 120);

    let first = item_x(&page, "First");
    let second = item_x(&page, "Second");
    assert_eq!(item_y(&page, "First"), item_y(&page, "Second"));
    let line = page
        .buffer
        .layout_runs()
        .find(|line| line.text.contains("First"))
        .expect("flex items should share a formatted line");
    let gap_widths = line
        .glyphs
        .iter()
        .filter(|glyph| [0, 8, 17].contains(&glyph.start))
        .map(|glyph| glyph.w)
        .collect::<Vec<_>>();
    assert_eq!(gap_widths.len(), 3, "leading, item, and trailing gaps");
    assert!(
        gap_widths.iter().copied().fold(f32::NEG_INFINITY, f32::max)
            - gap_widths.iter().copied().fold(f32::INFINITY, f32::min)
            < 0.1,
        "all three gaps should be equal: {gap_widths:?}"
    );
    assert!(
        first > 100.0 && second > first,
        "items should follow the leading gap"
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
    let mut normal = aligned_flex_page("column", "align-items", "normal");
    let mut stretch = aligned_flex_page("column", "align-items", "stretch");
    let mut center = aligned_flex_page("column", "align-items", "center");
    let mut end = aligned_flex_page("column", "align-items", "flex-end");
    for page in [&mut start, &mut normal, &mut stretch, &mut center, &mut end] {
        page.prepare_layout(400, 120);
    }

    let start_x = item_x(&start, "First");
    let normal_x = item_x(&normal, "First");
    let stretch_x = item_x(&stretch, "First");
    let center_x = item_x(&center, "First");
    let end_x = item_x(&end, "First");
    assert!((stretch_x - start_x).abs() < 0.1);
    assert!((normal_x - stretch_x).abs() < 0.1);
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
fn row_align_items_positions_different_line_heights_on_the_cross_axis() {
    let mut start = row_align_items_page("flex-start");
    let mut normal = row_align_items_page("normal");
    let mut stretch = row_align_items_page("stretch");
    let mut center = row_align_items_page("center");
    let mut end = row_align_items_page("flex-end");
    for page in [&mut start, &mut normal, &mut stretch, &mut center, &mut end] {
        page.prepare_layout(400, 120);
    }

    let start_y = item_glyph_y(&start, "Short");
    assert!((item_glyph_y(&normal, "Short") - start_y).abs() < 0.1);
    assert!((item_glyph_y(&stretch, "Short") - start_y).abs() < 0.1);
    let center_y = item_glyph_y(&center, "Short");
    let end_y = item_glyph_y(&end, "Short");
    assert!(
        center_y > start_y + 4.0,
        "center offset: {start_y} -> {center_y}"
    );
    assert!(end_y > center_y + 4.0, "end offset: {center_y} -> {end_y}");

    let text_x = item_x(&start, "Short") + crate::page::PAGE_PADDING + 12.0;
    let start_painted_y = item_painted_y_bounds(&mut start, text_x).0;
    let center_painted_y = item_painted_y_bounds(&mut center, text_x).0;
    let end_painted_y = item_painted_y_bounds(&mut end, text_x).0;
    assert!(center_painted_y > start_painted_y + 4);
    assert!(end_painted_y > center_painted_y + 4);

    let glyph_x = end
        .layout_runs()
        .find_map(|line| {
            let start = line.text.find("Short")?;
            let glyph = line.glyphs.iter().find(|glyph| glyph.start == start)?;
            Some(glyph.x)
        })
        .expect("the aligned item should have a glyph");
    end.update_hover(
        f64::from(glyph_x + crate::page::PAGE_PADDING + 2.0),
        f64::from(crate::page::PAGE_PADDING + 8.0 + end_y - start_y),
    );
    assert_eq!(end.hovered_path.last(), Some(&3));
}

#[test]
fn row_align_self_overrides_the_container_cross_alignment() {
    let mut centered = row_align_items_page_with_self("center", None);
    let mut end = row_align_items_page_with_self("center", Some("flex-end"));
    centered.prepare_layout(400, 120);
    end.prepare_layout(400, 120);
    assert!(
        item_glyph_y(&end, "Short") > item_glyph_y(&centered, "Short") + 4.0,
        "align-self should override the row container's align-items value"
    );
}

#[test]
fn row_baseline_alignment_positions_small_text_against_the_large_item_baseline() {
    let mut start = row_align_items_page("flex-start");
    let mut baseline = row_align_items_page("baseline");
    start.prepare_layout(400, 120);
    baseline.prepare_layout(400, 120);

    let text_x = item_x(&start, "Short") + crate::page::PAGE_PADDING + 12.0;
    let start_top = item_painted_y_bounds(&mut start, text_x).0;
    let baseline_top = item_painted_y_bounds(&mut baseline, text_x).0;
    assert!(
        baseline_top > start_top + 4,
        "baseline alignment should lower the smaller glyph run: {start_top} -> {baseline_top}"
    );
}

#[test]
fn align_self_overrides_the_column_container_alignment_per_item() {
    let mut normal = align_self_flex_page("normal");
    let mut center = align_self_flex_page("center");
    let mut end = align_self_flex_page("flex-end");
    normal.prepare_layout(400, 120);
    center.prepare_layout(400, 120);
    end.prepare_layout(400, 120);

    let first_x = item_x(&center, "First");
    assert!((item_x(&normal, "Second") - item_x(&normal, "First")).abs() < 0.1);
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

fn row_align_items_page(alignment: &str) -> PageRasterizer {
    row_align_items_page_with_self(alignment, None)
}

fn row_align_items_page_with_self(alignment: &str, align_self: Option<&str>) -> PageRasterizer {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("container".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: [("large", "Tall"), ("small", "Short")]
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
    let mut stylesheet = vec![
        StyleRule {
            selectors: vec![".container".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "display".to_owned(),
                    value: "flex".to_owned(),
                },
                StyleDeclaration {
                    property: "align-items".to_owned(),
                    value: alignment.to_owned(),
                },
            ],
        },
        StyleRule {
            selectors: vec![".large".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-size".to_owned(),
                value: "40px".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".small".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-size".to_owned(),
                value: "12px".to_owned(),
            }],
        },
    ];
    if let Some(align_self) = align_self {
        stylesheet.push(StyleRule {
            selectors: vec![".small".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "align-self".to_owned(),
                value: align_self.to_owned(),
            }],
        });
    }
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
    page.layout_runs()
        .find(|line| {
            let Some(start) = line.text.find(item) else {
                return false;
            };
            line.glyphs.iter().any(|glyph| glyph.start == start)
        })
        .expect("flex item should be on a formatted line")
        .line_top
}

fn item_glyph_y(page: &PageRasterizer, item: &str) -> f32 {
    page.layout_runs()
        .find_map(|line| {
            let start = line.text.find(item)?;
            line.glyphs
                .iter()
                .find(|glyph| glyph.start == start)
                .map(|glyph| glyph.y)
        })
        .expect("flex item should have a positioned glyph")
}

fn item_painted_y_bounds(page: &mut PageRasterizer, text_x: f32) -> (i32, i32) {
    let pixels = page.render_layout_pixels(400, 120, crate::style::WritingMode::HorizontalTb);
    let x_start = (text_x.floor() as usize).min(399);
    let x_end = (x_start + 80).min(400);
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    for y in 0..120_usize {
        for x in x_start..x_end {
            let index = (y * 400 + x) * 4;
            if pixels[index..index + 3]
                .iter()
                .any(|channel| *channel < 245)
            {
                min_y = min_y.min(y as i32);
                max_y = max_y.max(y as i32);
            }
        }
    }
    assert!(
        min_y < i32::MAX,
        "the aligned text should produce painted pixels"
    );
    (min_y, max_y)
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
