use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn wrapped_row_gap_resolves_only_on_wrapped_lines_and_resets_after_resize() {
    let mut wrapped_without_gap = wrapped_row_gap_page("0px", None, false, None);
    let mut wrapped_with_gap = wrapped_row_gap_page("12px", None, false, None);
    wrapped_without_gap.prepare_layout(60, 120);
    wrapped_with_gap.prepare_layout(60, 120);

    let base_distance =
        item_y(&wrapped_without_gap, "Second") - item_y(&wrapped_without_gap, "First");
    let gap_distance = item_y(&wrapped_with_gap, "Second") - item_y(&wrapped_with_gap, "First");
    assert!((gap_distance - base_distance - 12.0).abs() < 0.5);

    wrapped_with_gap.prepare_layout(400, 120);
    assert_eq!(
        item_y(&wrapped_with_gap, "Second"),
        item_y(&wrapped_with_gap, "First")
    );
    wrapped_with_gap.prepare_layout(60, 120);
    let resized_gap_distance =
        item_y(&wrapped_with_gap, "Second") - item_y(&wrapped_with_gap, "First");
    assert!((resized_gap_distance - gap_distance).abs() < 0.5);

    let mut single_without_gap = wrapped_row_gap_page("0px", None, false, None);
    let mut single_with_gap = wrapped_row_gap_page("12px", None, false, None);
    single_without_gap.prepare_layout(400, 120);
    single_with_gap.prepare_layout(400, 120);
    assert_eq!(
        item_y(&single_with_gap, "Second"),
        item_y(&single_without_gap, "Second")
    );
}

#[test]
fn percentage_wrapped_row_gap_needs_a_definite_cross_size() {
    let mut without_basis = wrapped_row_gap_page("0px", None, false, None);
    let mut indefinite = wrapped_row_gap_page("10%", None, false, None);
    without_basis.prepare_layout(60, 120);
    indefinite.prepare_layout(60, 120);
    assert_eq!(
        item_y(&indefinite, "Second"),
        item_y(&without_basis, "Second")
    );

    let mut base = wrapped_row_gap_page("0px", Some(("height", "100px")), false, None);
    let mut percentage = wrapped_row_gap_page("10%", Some(("height", "100px")), false, None);
    base.prepare_layout(60, 120);
    percentage.prepare_layout(60, 120);
    let base_distance = item_y(&base, "Second") - item_y(&base, "First");
    let percentage_distance = item_y(&percentage, "Second") - item_y(&percentage, "First");
    assert!((percentage_distance - base_distance - 10.0).abs() < 0.5);
}

#[test]
fn wrapped_row_gap_follows_the_cross_axis_in_vertical_writing() {
    let mut without_gap = wrapped_row_gap_page("0px", None, true, None);
    let mut with_gap = wrapped_row_gap_page("12px", None, true, None);
    without_gap.prepare_layout(120, 60);
    with_gap.prepare_layout(120, 60);
    let base_distance = item_y(&without_gap, "Second") - item_y(&without_gap, "First");
    let gap_distance = item_y(&with_gap, "Second") - item_y(&with_gap, "First");
    assert!((gap_distance - base_distance - 12.0).abs() < 0.5);

    let mut base = wrapped_row_gap_page("0px", Some(("width", "100px")), true, None);
    let mut percentage = wrapped_row_gap_page("10%", Some(("width", "100px")), true, None);
    base.prepare_layout(120, 60);
    percentage.prepare_layout(120, 60);
    let base_distance = item_y(&base, "Second") - item_y(&base, "First");
    let percentage_distance = item_y(&percentage, "Second") - item_y(&percentage, "First");
    assert!((percentage_distance - base_distance - 10.0).abs() < 0.5);
}

#[test]
fn align_content_space_between_distributes_remaining_cross_axis_space() {
    let mut start = wrapped_row_gap_page("0px", Some(("height", "100px")), false, None);
    let mut space_between = wrapped_row_gap_page(
        "0px",
        Some(("height", "100px")),
        false,
        Some("space-between"),
    );
    start.prepare_layout(60, 120);
    space_between.prepare_layout(60, 120);

    let start_distance = item_y(&start, "Second") - item_y(&start, "First");
    let distributed_distance = item_y(&space_between, "Second") - item_y(&space_between, "First");
    assert!(
        distributed_distance > start_distance + 50.0,
        "align-content: space-between should use free height between wrapped rows: base={start_distance}, distributed={distributed_distance}"
    );
}

fn wrapped_row_gap_page(
    row_gap: &str,
    cross_size: Option<(&str, &str)>,
    vertical_root: bool,
    align_content: Option<&str>,
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
        ("display", "flex"),
        ("flex-wrap", "wrap"),
        ("row-gap", row_gap),
    ];
    if let Some(cross_size) = cross_size {
        declarations.push(cross_size);
    }
    if let Some(align_content) = align_content {
        declarations.push(("align-content", align_content));
    }
    let mut stylesheet = vec![StyleRule {
        selectors: vec![".container".to_owned()],
        layer: None,
        declarations: declarations
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }];
    if vertical_root {
        stylesheet.push(StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "writing-mode".to_owned(),
                value: "vertical-rl".to_owned(),
            }],
        });
    }
    PageRasterizer::new(&nodes, &stylesheet)
}

fn item_y(page: &PageRasterizer, item: &str) -> f32 {
    page.buffer
        .layout_runs()
        .find(|line| {
            let Some(start) = line.text.find(item) else {
                return false;
            };
            line.glyphs.iter().any(|glyph| glyph.start == start)
        })
        .expect("flex item should be on a formatted line")
        .line_top
}
