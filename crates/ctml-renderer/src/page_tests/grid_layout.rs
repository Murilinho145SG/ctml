use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn explicit_grid_columns_place_items_in_rows_and_size_each_cell() {
    let mut page = grid_page(
        None,
        "100px 100px",
        "10px",
        "0px",
        &["Alpha", "Beta", "Gamma"],
    );
    page.prepare_layout(400, 160);

    let alpha = item_position(&page, "Alpha");
    let beta = item_position(&page, "Beta");
    let gamma = item_position(&page, "Gamma");
    assert!(
        (beta.0 - alpha.0 - 110.0).abs() < 1.0,
        "column starts: {alpha:?}, {beta:?}"
    );
    assert!(
        gamma.1 > alpha.1,
        "the third item should start an implicit second row"
    );
}

#[test]
fn repeated_fraction_tracks_distribute_items_across_the_available_width() {
    let mut page = grid_page(None, "repeat(2, 1fr)", "0px", "0px", &["Alpha", "Beta"]);
    page.prepare_layout(400, 160);

    let alpha = item_position(&page, "Alpha");
    let beta = item_position(&page, "Beta");
    assert!(
        beta.0 - alpha.0 > 150.0,
        "fractional track positions: {alpha:?}, {beta:?}"
    );
}

#[test]
fn grid_row_gap_adds_space_between_auto_placed_rows() {
    let mut no_gap = grid_page(None, "100px", "0px", "0px", &["Alpha", "Beta"]);
    let mut with_gap = grid_page(None, "100px", "0px", "12px", &["Alpha", "Beta"]);
    no_gap.prepare_layout(400, 160);
    with_gap.prepare_layout(400, 160);

    let distance_without_gap = item_position(&no_gap, "Beta").1 - item_position(&no_gap, "Alpha").1;
    let distance_with_gap =
        item_position(&with_gap, "Beta").1 - item_position(&with_gap, "Alpha").1;
    assert!(
        (distance_with_gap - distance_without_gap - 12.0).abs() < 1.0,
        "row gap should add 12px: base={distance_without_gap}, with_gap={distance_with_gap}"
    );
}

#[test]
fn mixed_fixed_and_fraction_tracks_use_the_remaining_block_grid_width() {
    let mut page = grid_page(None, "100px 1fr", "20px", "0px", &["Alpha", "Beta"]);
    page.prepare_layout(500, 160);

    let alpha = item_position(&page, "Alpha");
    let beta = item_position(&page, "Beta");
    assert!(
        (beta.0 - alpha.0 - 120.0).abs() < 1.0,
        "mixed track starts should include the fixed track and gap: {alpha:?}, {beta:?}"
    );
}

#[test]
fn percentage_and_fraction_tracks_resolve_against_the_grid_inline_size() {
    let mut page = grid_page(None, "25% 1fr", "20px", "0px", &["Alpha", "Beta"]);
    page.prepare_layout(500, 160);

    let alpha = item_position(&page, "Alpha");
    let beta = item_position(&page, "Beta");
    assert!(
        (beta.0 - alpha.0 - 141.0).abs() < 1.0,
        "percentage track and column gap should set the second column start: {alpha:?}, {beta:?}"
    );
}

#[test]
fn intrinsic_tracks_measure_and_cap_item_text() {
    let mut min_content = grid_page(
        None,
        "min-content 1fr",
        "0px",
        "0px",
        &["Alpha Beta", "Gamma"],
    );
    let mut max_content = grid_page(
        None,
        "max-content 1fr",
        "0px",
        "0px",
        &["Alpha Beta", "Gamma"],
    );
    let mut fit_content = grid_page(
        None,
        "fit-content(55px) 1fr",
        "0px",
        "0px",
        &["Alpha Beta", "Gamma"],
    );
    min_content.prepare_layout(500, 160);
    max_content.prepare_layout(500, 160);
    fit_content.prepare_layout(500, 160);

    let min_content_start = item_position(&min_content, "Gamma").0;
    let max_content_start = item_position(&max_content, "Gamma").0;
    let fit_content_start = item_position(&fit_content, "Gamma").0;
    assert!(
        max_content_start > min_content_start + 10.0,
        "max-content track should include the inter-word advance: min={min_content_start}, max={max_content_start}"
    );
    assert!(
        fit_content_start > min_content_start && fit_content_start < max_content_start,
        "fit-content track should clamp between min-content and max-content: min={min_content_start}, fit={fit_content_start}, max={max_content_start}"
    );
}

#[test]
fn auto_tracks_stretch_to_fill_the_definite_block_grid_width() {
    let mut page = grid_page(None, "auto auto", "0px", "0px", &["Alpha", "Beta"]);
    page.prepare_layout(400, 160);

    let alpha = item_position(&page, "Alpha");
    let beta = item_position(&page, "Beta");
    assert!(
        beta.0 - alpha.0 > 150.0,
        "auto tracks should share remaining width: {alpha:?}, {beta:?}"
    );
}

#[test]
fn fixed_minmax_tracks_respect_the_declared_floor_and_cap() {
    let mut floor = grid_page(
        None,
        "minmax(60px, 120px) 1fr",
        "0px",
        "0px",
        &["Alpha", "Gamma"],
    );
    let mut cap = grid_page(
        None,
        "minmax(0px, 20px) 1fr",
        "0px",
        "0px",
        &["Alpha", "Gamma"],
    );
    floor.prepare_layout(500, 160);
    cap.prepare_layout(500, 160);

    let floor_start = item_position(&floor, "Gamma").0;
    let cap_start = item_position(&cap, "Gamma").0;
    assert!(
        floor_start >= 60.0,
        "minmax floor was not applied: {floor_start}"
    );
    assert!(cap_start <= 21.0, "minmax cap was not applied: {cap_start}");
}

fn grid_page(
    width: Option<&str>,
    columns: &str,
    column_gap: &str,
    row_gap: &str,
    labels: &[&str],
) -> PageRasterizer {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("grid".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: labels
            .iter()
            .map(|label| UiNode::Element {
                tag: "p".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text((*label).to_owned())],
            })
            .collect(),
    }];
    let stylesheet = [StyleRule {
        selectors: vec![".grid".to_owned()],
        layer: None,
        declarations: [
            Some(("display", "grid")),
            Some(("grid-template-columns", columns)),
            Some(("column-gap", column_gap)),
            Some(("row-gap", row_gap)),
            width.map(|width| ("width", width)),
        ]
        .into_iter()
        .flatten()
        .map(|(property, value)| StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        })
        .collect(),
    }];
    PageRasterizer::new(&nodes, &stylesheet)
}

fn item_position(page: &PageRasterizer, item: &str) -> (f32, f32) {
    let (line, glyph) = page
        .buffer
        .layout_runs()
        .find_map(|line| {
            let start = line.text.find(item)?;
            let glyph = line.glyphs.iter().find(|glyph| glyph.start == start)?;
            Some((line, glyph))
        })
        .expect("grid item should have a positioned glyph");
    (glyph.x, line.line_top)
}
