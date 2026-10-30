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

#[test]
fn first_line_letter_spacing_applies_after_an_explicit_line_break() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![
            UiNode::Element {
                tag: "p".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text("lead".to_owned())],
            },
            UiNode::Element {
                tag: "p".to_owned(),
                class: Some("target".to_owned()),
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text("alpha beta gamma delta epsilon".to_owned())],
            },
        ],
    }];
    let page_for = |with_spacing| {
        let mut stylesheet = vec![StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "width".to_owned(),
                value: "75px".to_owned(),
            }],
        }];
        if with_spacing {
            stylesheet.push(StyleRule {
                selectors: vec![".target::first-line".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: "letter-spacing".to_owned(),
                    value: "4px".to_owned(),
                }],
            });
        }
        let mut page = PageRasterizer::new(&nodes, &stylesheet);
        page.prepare_layout(200, 160);
        let first_line_end = page
            .buffer
            .layout_runs()
            .filter(|line| !line.glyphs.is_empty())
            .nth(1)
            .expect("target paragraph should start after the first paragraph")
            .glyphs
            .iter()
            .map(|glyph| glyph.end)
            .max()
            .unwrap();
        (page, first_line_end)
    };

    let (_, base_end) = page_for(false);
    let (spaced_page, spaced_end) = page_for(true);
    let target_block = spaced_page
        .runs
        .iter()
        .filter_map(|run| run.first_line_block)
        .max()
        .expect("target paragraph should have a first-line block");

    assert_ne!(
        base_end, spaced_end,
        "letter spacing should affect the first line of a later paragraph"
    );
    assert!(
        spaced_page
            .cursor_spans
            .iter()
            .filter(|span| {
                span.first_line_block == Some(target_block) && span.start >= spaced_end
            })
            .all(|span| spaced_page.runs[span.run_index].style.letter_spacing == 0.0),
        "wrapped text after the later paragraph's first line should have normal spacing"
    );
}
