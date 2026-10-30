use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn first_line_letter_spacing_changes_only_the_first_line_advance() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("AB\nCD".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "white-space".to_owned(),
                value: "pre".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p::first-line".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "letter-spacing".to_owned(),
                value: "5px".to_owned(),
            }],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(180, 100);
    let mut lines = page.buffer.layout_runs();
    let first_line = lines.next().expect("paragraph should have a first line");
    let second_line = lines
        .next()
        .expect("preserved newline should create a second line");

    assert!(
        first_line.line_w > second_line.line_w + 5.0,
        "first-line letter spacing should add advance to the first line only"
    );
}

#[test]
fn first_line_letter_spacing_can_change_wrapping() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("alpha beta gamma delta epsilon".to_owned())],
    }];
    let first_line_end = |letter_spacing: Option<&str>| {
        let declarations = vec![StyleDeclaration {
            property: "width".to_owned(),
            value: "75px".to_owned(),
        }];
        let mut stylesheet = vec![StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations,
        }];
        if let Some(letter_spacing) = letter_spacing {
            stylesheet.push(StyleRule {
                selectors: vec!["p::first-line".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: "letter-spacing".to_owned(),
                    value: letter_spacing.to_owned(),
                }],
            });
        }
        let mut page = PageRasterizer::new(&nodes, &stylesheet);
        page.prepare_layout(200, 120);
        let end = page
            .buffer
            .layout_runs()
            .find(|line| !line.glyphs.is_empty())
            .unwrap()
            .glyphs
            .iter()
            .map(|glyph| glyph.end)
            .max()
            .unwrap();
        (page, end)
    };

    let (_, base_end) = first_line_end(None);
    let (spaced_page, spaced_end) = first_line_end(Some("4px"));
    assert!(
        spaced_page
            .buffer
            .layout_runs()
            .filter(|line| !line.glyphs.is_empty())
            .count()
            > 1,
        "paragraph should wrap to a second line"
    );
    assert_ne!(
        base_end, spaced_end,
        "first-line letter spacing should participate in line wrapping"
    );
    assert!(
        spaced_page
            .cursor_spans
            .iter()
            .filter(|span| span.start >= spaced_end)
            .all(|span| spaced_page.runs[span.run_index].style.letter_spacing == 0.0),
        "characters after the first line should have normal spacing"
    );
}
