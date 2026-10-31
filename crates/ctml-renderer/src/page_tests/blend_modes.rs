use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

fn declaration(property: &str, value: &str) -> StyleDeclaration {
    StyleDeclaration {
        property: property.to_owned(),
        value: value.to_owned(),
    }
}

#[test]
fn mix_blend_mode_combines_a_child_background_with_its_parent_backdrop() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("backdrop".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "p".to_owned(),
            class: Some("source".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("X".to_owned())],
        }],
    }];
    let stylesheet = |mode: &str, opacity: &str| {
        [
            StyleRule {
                selectors: vec![".backdrop".to_owned()],
                layer: None,
                declarations: vec![
                    declaration("width", "60px"),
                    declaration("height", "50px"),
                    declaration("background-color", "blue"),
                ],
            },
            StyleRule {
                selectors: vec![".source".to_owned()],
                layer: None,
                declarations: vec![
                    declaration("color", "transparent"),
                    declaration("width", "50px"),
                    declaration("height", "40px"),
                    declaration("background-color", "red"),
                    declaration("mix-blend-mode", mode),
                    declaration("opacity", opacity),
                ],
            },
        ]
    };
    let normal = PageRasterizer::new(&nodes, &stylesheet("normal", "1")).snapshot_pixels(100, 80);
    let multiply =
        PageRasterizer::new(&nodes, &stylesheet("multiply", "1")).snapshot_pixels(100, 80);
    let translucent_normal =
        PageRasterizer::new(&nodes, &stylesheet("normal", "0.5")).snapshot_pixels(100, 80);
    let translucent_multiply =
        PageRasterizer::new(&nodes, &stylesheet("multiply", "0.5")).snapshot_pixels(100, 80);

    assert!(
        normal
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 0, 0])
    );
    assert!(
        multiply
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [0, 0, 0])
    );
    assert!(
        translucent_normal
            .chunks_exact(4)
            .zip(translucent_multiply.chunks_exact(4))
            .any(|(normal, blended)| {
                normal[0] >= 127
                    && normal[0] <= 128
                    && normal[2] >= 127
                    && normal[2] <= 128
                    && blended[0] == 0
                    && blended[2] >= 127
                    && blended[2] <= 128
            })
    );
}

#[test]
fn isolation_prevents_descendant_blending_with_the_outside_backdrop() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("backdrop".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "section".to_owned(),
            class: Some("context".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Element {
                tag: "p".to_owned(),
                class: Some("source".to_owned()),
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text("X".to_owned())],
            }],
        }],
    }];
    let stylesheet = |isolation: &str| {
        [
            StyleRule {
                selectors: vec![".backdrop".to_owned()],
                layer: None,
                declarations: vec![
                    declaration("width", "60px"),
                    declaration("height", "50px"),
                    declaration("background-color", "blue"),
                ],
            },
            StyleRule {
                selectors: vec![".context".to_owned()],
                layer: None,
                declarations: vec![declaration("isolation", isolation)],
            },
            StyleRule {
                selectors: vec![".source".to_owned()],
                layer: None,
                declarations: vec![
                    declaration("color", "transparent"),
                    declaration("width", "50px"),
                    declaration("height", "40px"),
                    declaration("background-color", "red"),
                    declaration("mix-blend-mode", "multiply"),
                ],
            },
        ]
    };
    let blended_outside = PageRasterizer::new(&nodes, &stylesheet("auto")).snapshot_pixels(100, 80);
    let isolated = PageRasterizer::new(&nodes, &stylesheet("isolate")).snapshot_pixels(100, 80);

    assert!(
        blended_outside
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [0, 0, 0])
    );
    assert!(
        isolated
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 0, 0])
    );
}

#[test]
fn isolated_blending_inside_translucent_group_uses_the_group_backdrop() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("backdrop".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "section".to_owned(),
            class: Some("context".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Element {
                tag: "p".to_owned(),
                class: Some("source".to_owned()),
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text("X".to_owned())],
            }],
        }],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec![".backdrop".to_owned()],
            layer: None,
            declarations: vec![
                declaration("width", "80px"),
                declaration("height", "70px"),
                declaration("background-color", "blue"),
            ],
        },
        StyleRule {
            selectors: vec![".context".to_owned()],
            layer: None,
            declarations: vec![
                declaration("width", "60px"),
                declaration("height", "50px"),
                declaration("background-color", "yellow"),
                declaration("opacity", "0.5"),
                declaration("isolation", "isolate"),
            ],
        },
        StyleRule {
            selectors: vec![".source".to_owned()],
            layer: None,
            declarations: vec![
                declaration("color", "transparent"),
                declaration("width", "30px"),
                declaration("height", "30px"),
                declaration("background-color", "red"),
                declaration("mix-blend-mode", "multiply"),
            ],
        },
    ];
    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(100, 90);
    let pixel_at = |x: usize, y: usize| {
        let index = (y * 100 + x) * 4;
        &pixels[index..index + 4]
    };
    assert_eq!(pixel_at(10, 10), [128, 0, 127, 255]);
    assert_eq!(pixel_at(40, 40), [128, 128, 127, 255]);
    assert_eq!(pixel_at(70, 10), [0, 0, 255, 255]);
}

#[test]
fn root_body_mix_blend_mode_composites_over_the_page_canvas() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Root".to_owned())],
    }];
    let stylesheet = |mode: &str| {
        [StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: vec![
                declaration("background-color", "red"),
                declaration("mix-blend-mode", mode),
                declaration("color", "transparent"),
            ],
        }]
    };
    let normal = PageRasterizer::new(&nodes, &stylesheet("normal")).snapshot_pixels(100, 60);
    let mut screen_page = PageRasterizer::new(&nodes, &stylesheet("screen"));
    let screen = screen_page.snapshot_pixels(100, 60);

    assert!(
        normal
            .chunks_exact(4)
            .all(|pixel| pixel == [255, 0, 0, 255])
    );
    assert!(
        screen
            .chunks_exact(4)
            .all(|pixel| pixel == [255, 255, 255, 255])
    );
}
