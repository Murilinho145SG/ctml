use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn paints_text_fragment_background_color() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Background".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "background-color".to_owned(),
            value: "red".to_owned(),
        }],
    }];

    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(120, 50);

    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 0, 0])
    );
}

#[test]
fn composites_translucent_text_background_over_page_color() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Alpha".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "background-color".to_owned(),
            value: "rgb(0 255 0 / 50%)".to_owned(),
        }],
    }];

    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(100, 50);

    assert!(pixels.chunks_exact(4).any(|pixel| {
        pixel[0] > 120 && pixel[0] < 140 && pixel[1] == 255 && pixel[2] > 120 && pixel[2] < 140
    }));
}

#[test]
fn nests_element_backgrounds_behind_child_text() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![
            UiNode::Text("Parent ".to_owned()),
            UiNode::Element {
                tag: "span".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text("child".to_owned())],
            },
        ],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "background-color".to_owned(),
                value: "red".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["span".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "background-color".to_owned(),
                value: "blue".to_owned(),
            }],
        },
    ];

    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(160, 50);

    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 0, 0])
    );
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [0, 0, 255])
    );
}
