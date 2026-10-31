use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn flex_basis_sets_item_advance_from_lengths_and_percentages() {
    let mut natural = flex_basis_page(None, None, false);
    let mut fixed_basis = flex_basis_page(Some("150px"), None, false);
    let mut percentage_basis = flex_basis_page(Some("50%"), None, false);
    let mut auto_width = flex_basis_page(Some("auto"), Some("150px"), false);
    natural.prepare_layout(400, 120);
    fixed_basis.prepare_layout(400, 120);
    percentage_basis.prepare_layout(400, 120);
    auto_width.prepare_layout(400, 120);

    let natural_second = item_x(&natural, "Second");
    let fixed_second = item_x(&fixed_basis, "Second");
    let percentage_second = item_x(&percentage_basis, "Second");
    let auto_width_second = item_x(&auto_width, "Second");
    assert!(fixed_second > natural_second + 70.0);
    assert!((fixed_second - 150.0).abs() < 2.0);
    assert!(
        (percentage_second - 200.0).abs() < 2.0,
        "50% flex-basis should position the next item at 200px; observed {percentage_second}"
    );
    assert!((auto_width_second - fixed_second).abs() < 2.0);
}

#[test]
fn percentage_flex_basis_uses_content_size_in_auto_sized_inline_flex() {
    let mut natural = flex_basis_page(None, None, true);
    let mut percentage = flex_basis_page(Some("50%"), None, true);
    natural.prepare_layout(400, 120);
    percentage.prepare_layout(400, 120);
    assert!((item_x(&percentage, "Second") - item_x(&natural, "Second")).abs() < 1.0);
}

fn flex_basis_page(basis: Option<&str>, width: Option<&str>, inline_flex: bool) -> PageRasterizer {
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
    let rule = |selector: &str, declarations: Vec<(&str, &str)>| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: declarations
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    };
    let mut first_declarations = Vec::new();
    if let Some(basis) = basis {
        first_declarations.push(("flex-basis", basis));
    }
    if let Some(width) = width {
        first_declarations.push(("width", width));
    }
    let display = if inline_flex { "inline-flex" } else { "flex" };
    let mut container_declarations = vec![("display", display)];
    if !inline_flex {
        container_declarations.push(("width", "400px"));
    }
    PageRasterizer::new(
        &nodes,
        &[
            rule(".container", container_declarations),
            rule(".first", first_declarations),
        ],
    )
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
