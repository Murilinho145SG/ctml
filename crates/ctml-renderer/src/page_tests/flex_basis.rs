use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn flex_basis_sets_item_advance_from_lengths_and_percentages() {
    let mut natural = flex_basis_page(None, None, false);
    let mut fixed_basis = flex_basis_page(Some("150px"), None, false);
    let mut percentage_basis = flex_basis_page(Some("50%"), None, false);
    let mut calculated_basis = flex_basis_page(Some("calc(25% + 50px)"), None, false);
    let mut auto_width = flex_basis_page(Some("auto"), Some("150px"), false);
    natural.prepare_layout(400, 120);
    fixed_basis.prepare_layout(400, 120);
    percentage_basis.prepare_layout(400, 120);
    calculated_basis.prepare_layout(400, 120);
    auto_width.prepare_layout(400, 120);

    let natural_second = item_x(&natural, "Second");
    let fixed_second = item_x(&fixed_basis, "Second");
    let percentage_second = item_x(&percentage_basis, "Second");
    let calculated_second = item_x(&calculated_basis, "Second");
    let auto_width_second = item_x(&auto_width, "Second");
    assert!(fixed_second > natural_second + 70.0);
    assert!((fixed_second - 150.0).abs() < 2.0);
    assert!(
        (percentage_second - 200.0).abs() < 2.0,
        "50% flex-basis should position the next item at 200px; observed {percentage_second}"
    );
    assert!(
        (calculated_second - 150.0).abs() < 2.0,
        "calc(25% + 50px) should resolve against the 400px container; observed {calculated_second}"
    );
    assert!((auto_width_second - fixed_second).abs() < 2.0);
}

#[test]
fn min_and_max_main_sizes_clamp_the_flex_base_size() {
    let mut min_clamped =
        flex_basis_page_with_constraints(Some("50px"), None, false, Some("120px"), None);
    let mut max_clamped =
        flex_basis_page_with_constraints(Some("150px"), None, false, None, Some("100px"));
    let mut percent_min_clamped =
        flex_basis_page_with_constraints(Some("50px"), None, false, Some("25%"), None);
    min_clamped.prepare_layout(400, 120);
    max_clamped.prepare_layout(400, 120);
    percent_min_clamped.prepare_layout(400, 120);

    let min_second = item_x(&min_clamped, "Second");
    let max_second = item_x(&max_clamped, "Second");
    let percent_min_second = item_x(&percent_min_clamped, "Second");
    assert!(
        (min_second - 120.0).abs() < 2.0,
        "min-width should raise the 50px flex basis to 120px; observed {min_second}"
    );
    assert!(
        (max_second - 100.0).abs() < 2.0,
        "max-width should lower the 150px flex basis to 100px; observed {max_second}"
    );
    assert!(
        (percent_min_second - 100.0).abs() < 2.0,
        "a definite 25% min-width should clamp the basis to 100px; observed {percent_min_second}"
    );
}

#[test]
fn percentage_flex_basis_uses_content_size_in_auto_sized_inline_flex() {
    let mut natural = flex_basis_page(None, None, true);
    let mut percentage = flex_basis_page(Some("50%"), None, true);
    natural.prepare_layout(400, 120);
    percentage.prepare_layout(400, 120);
    assert!((item_x(&percentage, "Second") - item_x(&natural, "Second")).abs() < 1.0);
}

#[test]
fn content_and_max_content_flex_basis_share_max_content_sizing() {
    let mut content = wrapped_flex_basis_page("content");
    let mut max_content = wrapped_flex_basis_page("max-content");
    content.prepare_layout(120, 180);
    max_content.prepare_layout(120, 180);

    let content_y = item_y(&content, "Second");
    let max_content_y = item_y(&max_content, "Second");
    assert!((max_content_y - content_y).abs() < 0.5);
}

fn wrapped_flex_basis_page(basis: &str) -> PageRasterizer {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("container".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: [
            ("first", "First First First First First First"),
            ("second", "Second"),
        ]
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
    PageRasterizer::new(
        &nodes,
        &[
            rule(
                ".container",
                &[
                    ("display", "flex"),
                    ("flex-wrap", "wrap"),
                    ("width", "60px"),
                ],
            ),
            rule(".first", &[("flex-basis", basis)]),
        ],
    )
}

fn item_y(page: &PageRasterizer, text: &str) -> f32 {
    page.buffer
        .layout_runs()
        .find(|line| {
            let Some(start) = line.text.find(text) else {
                return false;
            };
            line.glyphs.iter().any(|glyph| glyph.start == start)
        })
        .expect("flex item should have a formatted line")
        .line_top
}

fn flex_basis_page(basis: Option<&str>, width: Option<&str>, inline_flex: bool) -> PageRasterizer {
    flex_basis_page_with_constraints(basis, width, inline_flex, None, None)
}

fn flex_basis_page_with_constraints(
    basis: Option<&str>,
    width: Option<&str>,
    inline_flex: bool,
    min_width: Option<&str>,
    max_width: Option<&str>,
) -> PageRasterizer {
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
    if let Some(min_width) = min_width {
        first_declarations.push(("min-width", min_width));
    }
    if let Some(max_width) = max_width {
        first_declarations.push(("max-width", max_width));
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
