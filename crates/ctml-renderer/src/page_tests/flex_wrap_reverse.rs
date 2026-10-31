use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn flex_wrap_defaults_to_single_row_and_wrap_allows_items_on_another_line() {
    let mut nowrap = flex_wrap_page(None, "");
    let mut wrap = flex_wrap_page(Some("wrap"), "");
    nowrap.prepare_layout(60, 120);
    wrap.prepare_layout(60, 120);

    assert_eq!(
        item_y(&nowrap, "First"),
        item_y(&nowrap, "Second"),
        "the initial flex-wrap value should keep row items on one line"
    );
    assert!(
        item_y(&wrap, "Second") > item_y(&wrap, "First"),
        "wrap should move an overflowing flex item to a subsequent line"
    );
}

#[test]
fn flex_wrap_reverse_places_the_first_wrapped_row_at_the_cross_end() {
    let mut wrap = flex_wrap_page(Some("wrap"), "");
    let mut reverse = flex_wrap_page(Some("wrap-reverse"), "");
    wrap.prepare_layout(60, 120);
    reverse.prepare_layout(60, 120);
    assert!(item_y(&wrap, "Second") > item_y(&wrap, "First"));
    assert!(
        item_y(&reverse, "First") > item_y(&reverse, "Second"),
        "wrap-reverse should place the first source row at the cross-end"
    );
    let first_glyph = item_glyph(&reverse, "First");
    reverse.update_hover(
        f64::from(first_glyph.x + crate::page::PAGE_PADDING + 2.0),
        f64::from(item_y(&reverse, "First") + crate::page::PAGE_PADDING + 8.0),
    );
    assert_eq!(
        reverse.hovered_path.last(),
        Some(&2),
        "hit testing should follow the moved row"
    );
}

#[test]
fn flex_wrap_reverse_reverses_rows_inside_a_fixed_cross_size() {
    let mut wrap = flex_wrap_page(Some("wrap"), "100px");
    let mut reverse = flex_wrap_page(Some("wrap-reverse"), "100px");
    wrap.prepare_layout(60, 120);
    reverse.prepare_layout(60, 120);

    assert!(item_y(&wrap, "First") < item_y(&wrap, "Second"));
    assert!(
        item_y(&reverse, "First") > item_y(&reverse, "Second"),
        "wrap-reverse should reverse wrapped rows with a definite cross size"
    );
    assert!(
        (item_y(&reverse, "First") - item_y(&wrap, "Second")).abs() < 1.0,
        "wrap-reverse should move the first source row to the second row's regular position"
    );
    assert!(
        (item_y(&reverse, "Second") - item_y(&wrap, "First")).abs() < 1.0,
        "wrap-reverse should move the second source row to the first row's regular position"
    );
}

fn flex_wrap_page(wrap: Option<&str>, height: &str) -> PageRasterizer {
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
            property: "width".to_owned(),
            value: "44px".to_owned(),
        },
    ];
    if !height.is_empty() {
        declarations.push(StyleDeclaration {
            property: "height".to_owned(),
            value: height.to_owned(),
        });
    }
    if let Some(wrap) = wrap {
        declarations.push(StyleDeclaration {
            property: "flex-wrap".to_owned(),
            value: wrap.to_owned(),
        });
    }
    let stylesheet = [StyleRule {
        selectors: vec![".container".to_owned()],
        layer: None,
        declarations,
    }];
    PageRasterizer::new(&nodes, &stylesheet)
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
