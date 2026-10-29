use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn first_line_foreground_applies_only_to_the_first_line_of_a_block() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![
            UiNode::Text("First line".to_owned()),
            UiNode::Element {
                tag: "div".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text("Second line".to_owned())],
            },
        ],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-line".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "color".to_owned(),
                value: "lime".to_owned(),
            },
            StyleDeclaration {
                property: "background-color".to_owned(),
                value: "yellow".to_owned(),
            },
        ],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(140, 70);
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [0, 255, 0])
    );
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 255, 0])
    );
    assert!(pixels.chunks_exact(4).any(|pixel| pixel[..3] == [0, 0, 0]));
}

#[test]
fn first_letter_foreground_and_background_apply_to_one_character() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("First letter".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-letter".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "color".to_owned(),
                value: "lime".to_owned(),
            },
            StyleDeclaration {
                property: "background-color".to_owned(),
                value: "yellow".to_owned(),
            },
        ],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(140, 70);

    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [0, 255, 0])
    );
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 255, 0])
    );
    assert!(pixels.chunks_exact(4).any(|pixel| pixel[..3] == [0, 0, 0]));
}

#[test]
fn first_letter_font_styles_apply_to_the_first_non_whitespace_character() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("First letter".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-letter".to_owned()],
        layer: None,
        declarations: [
            ("font-family", "monospace"),
            ("font-size", "30px"),
            ("font-weight", "700"),
            ("font-style", "italic"),
            ("font-stretch", "condensed"),
        ]
        .into_iter()
        .map(|(property, value)| StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        })
        .collect(),
    }];
    let page = PageRasterizer::new(&nodes, &stylesheet);
    let first = page.runs.iter().find(|run| run.text == "F").unwrap();
    let following = page
        .runs
        .iter()
        .find(|run| run.text.starts_with('i'))
        .unwrap();
    assert_eq!(first.style.font_size, 30.0);
    assert_eq!(first.style.family, "monospace");
    assert_eq!(first.style.weight, cosmic_text::Weight(700));
    assert_eq!(first.style.font_style, cosmic_text::Style::Italic);
    assert_eq!(first.style.stretch, cosmic_text::Stretch::Condensed);
    assert_eq!(following.style.font_size, 16.0);
}
