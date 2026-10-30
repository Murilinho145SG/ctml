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
fn paints_background_across_an_empty_sized_block_padding_box() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("empty".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: Vec::new(),
    }];
    let stylesheet = [StyleRule {
        selectors: vec![".empty".to_owned()],
        layer: None,
        declarations: [
            ("width", "40px"),
            ("height", "30px"),
            ("padding", "6px"),
            ("background-color", "red"),
        ]
        .into_iter()
        .map(|(property, value)| StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        })
        .collect(),
    }];
    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(120, 100);
    let red_pixels = pixels
        .chunks_exact(4)
        .enumerate()
        .filter(|(_, pixel)| pixel[..3] == [255, 0, 0])
        .map(|(index, _)| (index % 120, index / 120))
        .collect::<Vec<_>>();
    let min_x = red_pixels.iter().map(|(x, _)| *x).min().unwrap_or_default();
    let max_x = red_pixels.iter().map(|(x, _)| *x).max().unwrap_or_default();
    let min_y = red_pixels.iter().map(|(_, y)| *y).min().unwrap_or_default();
    let max_y = red_pixels.iter().map(|(_, y)| *y).max().unwrap_or_default();

    assert!(
        !red_pixels.is_empty(),
        "the empty block should paint its background"
    );
    assert!(
        max_x - min_x + 1 >= 52,
        "background should include horizontal padding"
    );
    assert!(
        max_y - min_y + 1 >= 42,
        "background should include vertical padding"
    );
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
