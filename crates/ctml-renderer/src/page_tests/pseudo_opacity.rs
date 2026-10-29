use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn first_line_opacity_composites_its_background() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Opacity background".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-line".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "background-color".to_owned(),
                value: "blue".to_owned(),
            },
            StyleDeclaration {
                property: "opacity".to_owned(),
                value: "50%".to_owned(),
            },
        ],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(180, 70);

    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [127, 127, 255] || pixel[..3] == [128, 128, 255])
    );
    assert!(
        page.runs
            .iter()
            .any(|run| run.first_line_opacity == Some(0.5))
    );
}

#[test]
fn first_letter_opacity_composites_its_background() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Opacity background".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-letter".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "background-color".to_owned(),
                value: "lime".to_owned(),
            },
            StyleDeclaration {
                property: "opacity".to_owned(),
                value: "0.5".to_owned(),
            },
        ],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(180, 70);

    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [127, 255, 127] || pixel[..3] == [128, 255, 128])
    );
    assert!(
        page.runs
            .iter()
            .any(|run| run.first_letter_opacity == Some(0.5))
    );
}

#[test]
fn zero_first_line_opacity_hides_text_and_its_shadow() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Invisible first line".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-line".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "opacity".to_owned(),
                value: "0".to_owned(),
            },
            StyleDeclaration {
                property: "text-shadow".to_owned(),
                value: "3px 0px red".to_owned(),
            },
        ],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(180, 70);

    assert!(
        pixels
            .chunks_exact(4)
            .all(|pixel| pixel[..3] == [255, 255, 255])
    );
}

#[test]
fn zero_first_letter_opacity_hides_its_glyph() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("A".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-letter".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "opacity".to_owned(),
            value: "0".to_owned(),
        }],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(80, 60);

    assert!(
        pixels
            .chunks_exact(4)
            .all(|pixel| pixel[..3] == [255, 255, 255])
    );
}
