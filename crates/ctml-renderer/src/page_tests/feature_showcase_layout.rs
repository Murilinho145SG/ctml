use std::path::PathBuf;

use ctml_compiler::{AttributeValue, parse, parse_stylesheet_file};

use crate::{StyleDeclaration, StyleRule, page::PageRasterizer};

#[test]
fn feature_showcase_grid_cards_share_a_row_below_the_header() {
    let page = feature_showcase_page(1120, 850);
    let lines = page.layout_runs().collect::<Vec<_>>();
    let topbar = lines
        .iter()
        .find(|line| line.text.contains("COMPILED NATIVE INTERFACE"))
        .expect("topbar text should render");
    let hero_label = lines
        .iter()
        .find(|line| line.text.contains("THE COMPILED UI TOOLKIT"))
        .expect("hero label should render");
    assert!(
        hero_label.line_top >= topbar.line_top + 77.0,
        "hero should follow the 56px topbar and 22px margin: {}, {}",
        topbar.line_top,
        hero_label.line_top
    );

    let cards = ["01 / TYPOGRAPHY", "02 / COLOR + PAINT", "03 / LAYOUT"].map(|label| {
        lines
            .iter()
            .find(|line| line.text.contains(label))
            .unwrap_or_else(|| panic!("showcase label {label:?} should render"))
    });
    let positions = cards.map(|line| {
        (
            line.line_top,
            line.glyphs.first().map_or(0.0, |glyph| glyph.x),
        )
    });
    assert!(
        positions
            .iter()
            .all(|position| (position.0 - positions[0].0).abs() < 1.0),
        "the three showcase cards should share one row: {positions:?}"
    );
    assert!(
        positions[1].1 - positions[0].1 > 250.0 && positions[2].1 - positions[1].1 > 250.0,
        "the cards should occupy three columns: {positions:?}"
    );
}

fn feature_showcase_page(width: u32, height: u32) -> PageRasterizer {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/feature-showcase/src");
    let document =
        parse(&std::fs::read_to_string(root.join("main.ctml")).expect("showcase CTML should load"))
            .expect("showcase CTML should parse");
    let nodes = document.body.iter().map(super::ui_node).collect::<Vec<_>>();
    let mut rules = Vec::new();
    for element in document.stylesheets() {
        let href = element
            .attribute("href")
            .and_then(AttributeValue::as_str)
            .expect("showcase stylesheet should have an href");
        let (stylesheet, _) = parse_stylesheet_file(&root.join(href), &root)
            .expect("showcase stylesheet and imports should parse");
        rules.extend(stylesheet.rules.into_iter().map(|rule| {
            StyleRule {
                selectors: rule.selectors,
                declarations: rule
                    .declarations
                    .into_iter()
                    .map(|declaration| StyleDeclaration {
                        property: declaration.property,
                        value: declaration.value,
                    })
                    .collect(),
                layer: None,
            }
        }));
    }
    let mut page = PageRasterizer::new(&nodes, &rules);
    page.prepare_layout(width, height);
    page
}
