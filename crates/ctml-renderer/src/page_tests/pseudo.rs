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
