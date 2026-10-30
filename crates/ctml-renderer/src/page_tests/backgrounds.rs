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
fn paints_content_block_background_across_its_padding_box() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("content".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("A".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec![".content".to_owned()],
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
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(120, 100);
    let rect = page.overflow_geometry[&1].border_box;
    let pixels = page.snapshot_pixels(120, 100);
    let x = (rect.left + 2.0).round() as usize;
    let y = (rect.top + 2.0).round() as usize;
    let pixel = &pixels[(y * 120 + x) * 4..][..3];

    assert_eq!(
        pixel,
        [255, 0, 0],
        "padding area should receive the block background"
    );
}

#[test]
fn paints_inline_background_across_padding_box() {
    let nodes = [UiNode::Element {
        tag: "span".to_owned(),
        class: Some("padded".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("X".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec![".padded".to_owned()],
        layer: None,
        declarations: [("padding", "8px"), ("background-color", "red")]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }];
    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(100, 70);
    let pixel = &pixels[(20 * 100 + 8) * 4..][..3];

    assert_eq!(
        pixel,
        [255, 0, 0],
        "inline padding should receive the background"
    );
}

#[test]
fn paints_background_for_an_empty_inline_element() {
    let nodes = [UiNode::Element {
        tag: "span".to_owned(),
        class: Some("empty".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: Vec::new(),
    }];
    let stylesheet = [StyleRule {
        selectors: vec![".empty".to_owned()],
        layer: None,
        declarations: [("padding", "8px"), ("background-color", "red")]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }];
    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(100, 70);
    let pixel = &pixels[(20 * 100 + 8) * 4..][..3];

    assert_eq!(pixel, [255, 0, 0], "empty inline padding should be painted");
}

#[test]
fn paints_inline_background_on_each_wrapped_text_fragment() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("narrow".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "span".to_owned(),
            class: Some("highlight".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("alpha beta gamma delta".to_owned())],
        }],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec![".narrow".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "width".to_owned(),
                value: "65px".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".highlight".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "background-color".to_owned(),
                value: "red".to_owned(),
            }],
        },
    ];
    let rendered = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(120, 120);
    let red_rows = rendered
        .chunks_exact(4)
        .enumerate()
        .filter(|(_, pixel)| pixel[..3] == [255, 0, 0])
        .map(|(index, _)| index / 120)
        .collect::<std::collections::HashSet<_>>();

    assert!(
        red_rows.len() > 20,
        "wrapped inline background should paint every text line fragment"
    );
    assert!(
        red_rows.iter().max().unwrap() - red_rows.iter().min().unwrap() >= 20,
        "background should span multiple line fragments"
    );
}

#[test]
fn empty_inline_background_does_not_change_adjacent_text_transformation() {
    let nodes = [
        UiNode::Text("apple".to_owned()),
        UiNode::Element {
            tag: "span".to_owned(),
            class: Some("empty".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: Vec::new(),
        },
        UiNode::Text("banana".to_owned()),
    ];
    let stylesheet = [
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "text-transform".to_owned(),
                value: "capitalize".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".empty".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "background-color".to_owned(),
                value: "red".to_owned(),
            }],
        },
    ];
    let page = PageRasterizer::new(&nodes, &stylesheet);
    let rendered_text = page
        .runs
        .iter()
        .map(|run| run.text.as_str())
        .collect::<String>()
        .replace('\u{200b}', "");

    assert!(
        rendered_text.contains("Applebanana"),
        "empty inline decoration should not create a word boundary: {rendered_text:?}"
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

#[test]
fn inline_background_still_paints_over_a_same_color_block_background() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![
            UiNode::Text("before ".to_owned()),
            UiNode::Element {
                tag: "span".to_owned(),
                class: Some("child".to_owned()),
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text("child".to_owned())],
            },
            UiNode::Text(" after".to_owned()),
        ],
    }];
    let color = "rgb(255 255 0 / 50%)";
    let stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "background-color".to_owned(),
                value: color.to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".child".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "background-color".to_owned(),
                value: color.to_owned(),
            }],
        },
    ];
    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(180, 60);
    let parent_only = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "background-color".to_owned(),
            value: color.to_owned(),
        }],
    }];
    let baseline = PageRasterizer::new(&nodes, &parent_only).snapshot_pixels(180, 60);

    assert_ne!(
        pixels, baseline,
        "inline background should composite above the block fill"
    );
}
