use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn wrapped_row_gap_resolves_only_on_wrapped_lines_and_resets_after_resize() {
    let mut wrapped_without_gap = wrapped_row_gap_page("0px", None, false);
    let mut wrapped_with_gap = wrapped_row_gap_page("12px", None, false);
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

    let mut single_without_gap = wrapped_row_gap_page("0px", None, false);
    let mut single_with_gap = wrapped_row_gap_page("12px", None, false);
    single_without_gap.prepare_layout(400, 120);
    single_with_gap.prepare_layout(400, 120);
    assert_eq!(
        item_y(&single_with_gap, "Second"),
        item_y(&single_without_gap, "Second")
    );
}

#[test]
fn percentage_wrapped_row_gap_needs_a_definite_cross_size() {
    let mut without_basis = wrapped_row_gap_page("0px", None, false);
    let mut indefinite = wrapped_row_gap_page("10%", None, false);
    without_basis.prepare_layout(60, 120);
    indefinite.prepare_layout(60, 120);
    assert_eq!(
        item_y(&indefinite, "Second"),
        item_y(&without_basis, "Second")
    );

    let mut base = wrapped_row_gap_page("0px", Some(("height", "100px")), false);
    let mut percentage = wrapped_row_gap_page("10%", Some(("height", "100px")), false);
    base.prepare_layout(60, 120);
    percentage.prepare_layout(60, 120);
    let base_distance = item_y(&base, "Second") - item_y(&base, "First");
    let percentage_distance = item_y(&percentage, "Second") - item_y(&percentage, "First");
    assert!((percentage_distance - base_distance - 10.0).abs() < 0.5);
}

#[test]
fn wrapped_row_gap_follows_the_cross_axis_in_vertical_writing() {
    let mut without_gap = wrapped_row_gap_page("0px", None, true);
    let mut with_gap = wrapped_row_gap_page("12px", None, true);
    without_gap.prepare_layout(120, 60);
    with_gap.prepare_layout(120, 60);
    let base_distance = item_y(&without_gap, "Second") - item_y(&without_gap, "First");
    let gap_distance = item_y(&with_gap, "Second") - item_y(&with_gap, "First");
    assert!((gap_distance - base_distance - 12.0).abs() < 0.5);

    let mut base = wrapped_row_gap_page("0px", Some(("width", "100px")), true);
    let mut percentage = wrapped_row_gap_page("10%", Some(("width", "100px")), true);
    base.prepare_layout(120, 60);
    percentage.prepare_layout(120, 60);
    let base_distance = item_y(&base, "Second") - item_y(&base, "First");
    let percentage_distance = item_y(&percentage, "Second") - item_y(&percentage, "First");
    assert!((percentage_distance - base_distance - 10.0).abs() < 0.5);
}

fn wrapped_row_gap_page(
    row_gap: &str,
    cross_size: Option<(&str, &str)>,
    vertical_root: bool,
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
