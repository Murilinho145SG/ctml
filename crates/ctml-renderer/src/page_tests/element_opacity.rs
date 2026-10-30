use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

fn declaration(property: &str, value: &str) -> StyleDeclaration {
    StyleDeclaration {
        property: property.to_owned(),
        value: value.to_owned(),
    }
}

#[test]
fn element_opacity_scales_fragment_background_and_stays_on_text_runs() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Opacity".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![
            declaration("background-color", "blue"),
            declaration("color", "red"),
            declaration("opacity", "0.5"),
        ],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(160, 70);

    assert!(
        page.runs
            .iter()
            .any(|run| { run.style.opacity == 0.5 && run.style.color == [255, 0, 0, 255] })
    );
    assert!(pixels.chunks_exact(4).any(|pixel| {
        (pixel[0] == 127 || pixel[0] == 128)
            && (pixel[1] == 127 || pixel[1] == 128)
            && pixel[2] == 255
    }));
}

#[test]
fn nested_element_opacities_multiply_for_descendant_paints() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "span".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("Nested".to_owned())],
        }],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![
                declaration("color", "transparent"),
                declaration("opacity", "0.5"),
            ],
        },
        StyleRule {
            selectors: vec!["span".to_owned()],
            layer: None,
            declarations: vec![
                declaration("background-color", "lime"),
                declaration("color", "transparent"),
                declaration("opacity", "0.5"),
            ],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(160, 70);

    assert!(page.runs.iter().any(|run| run.style.opacity == 0.25));
    assert!(pixels.chunks_exact(4).any(|pixel| {
        (pixel[0] == 191 || pixel[0] == 192)
            && pixel[1] == 255
            && (pixel[2] == 191 || pixel[2] == 192)
    }));
}

#[test]
fn page_root_opacity_composites_overlapping_page_paints_as_one_group() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Overlap".to_owned())],
    }];
    let declarations = |opacity: &str| {
        vec![
            declaration("background-color", "blue"),
            declaration("opacity", opacity),
        ]
    };
    let opaque_stylesheet = [
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: declarations("1"),
        },
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![declaration("color", "red")],
        },
    ];
    let translucent_stylesheet = [
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: declarations("0.5"),
        },
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![declaration("color", "red")],
        },
    ];
    let opaque = PageRasterizer::new(&nodes, &opaque_stylesheet).snapshot_pixels(160, 70);
    let translucent = PageRasterizer::new(&nodes, &translucent_stylesheet).snapshot_pixels(160, 70);

    assert_eq!(&opaque[..4], &[0, 0, 255, 255]);
    assert!(translucent[0] >= 127 && translucent[0] <= 128);
    assert!(translucent[1] >= 127 && translucent[1] <= 128);
    assert_eq!(&translucent[2..4], &[255, 255]);
    assert!(
        opaque
            .chunks_exact(4)
            .any(|pixel| pixel[0] > pixel[1] && pixel[0] > 180)
    );
    for (opaque, translucent) in opaque.chunks_exact(4).zip(translucent.chunks_exact(4)) {
        for (actual, source) in translucent[..3].iter().zip(&opaque[..3]) {
            let expected = (255.0 + (f32::from(*source) - 255.0) * 0.5).round() as u8;
            assert!(
                actual.abs_diff(expected) <= 1,
                "channel {actual}, expected {expected}, source {source}"
            );
        }
    }
}

#[test]
fn zero_page_root_opacity_leaves_the_white_page_canvas() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Invisible".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: vec![
                declaration("background-color", "blue"),
                declaration("opacity", "0"),
            ],
        },
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![declaration("color", "red")],
        },
    ];
    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(160, 70);

    assert!(
        pixels
            .chunks_exact(4)
            .all(|pixel| pixel == [255, 255, 255, 255])
    );
}

#[test]
fn nested_element_opacity_composites_its_background_and_text_as_one_group() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Overlap".to_owned())],
    }];
    let styles_for = |opacity: &str| {
        [StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![
                declaration("background-color", "blue"),
                declaration("border", "2px solid lime"),
                declaration("box-shadow", "3px 3px 2px 1px green"),
                declaration("color", "red"),
                declaration("opacity", opacity),
            ],
        }]
    };
    let opaque = PageRasterizer::new(&nodes, &styles_for("1")).snapshot_pixels(160, 70);
    let translucent = PageRasterizer::new(&nodes, &styles_for("0.5")).snapshot_pixels(160, 70);

    assert!(
        opaque
            .chunks_exact(4)
            .any(|pixel| pixel[0] == 255 && pixel[1] == 0 && pixel[2] == 0)
    );
    for (opaque, translucent) in opaque.chunks_exact(4).zip(translucent.chunks_exact(4)) {
        for (actual, source) in translucent[..3].iter().zip(&opaque[..3]) {
            let expected = (255.0 + (f32::from(*source) - 255.0) * 0.5).round() as u8;
            assert!(
                actual.abs_diff(expected) <= 1,
                "channel {actual}, expected {expected}, source {source}"
            );
        }
    }
}

#[test]
fn nested_opacity_groups_compose_recursively_across_overlapping_boxes() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "p".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Element {
                tag: "span".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text("Overlap".to_owned())],
            }],
        }],
    }];
    let styles_for = |parent_opacity: &str| {
        [
            StyleRule {
                selectors: vec!["div".to_owned()],
                layer: None,
                declarations: vec![
                    declaration("background-color", "yellow"),
                    declaration("opacity", parent_opacity),
                ],
            },
            StyleRule {
                selectors: vec!["p".to_owned()],
                layer: None,
                declarations: vec![
                    declaration("background-color", "blue"),
                    declaration("border", "2px solid lime"),
                    declaration("box-shadow", "3px 3px 2px 1px green"),
                    declaration("color", "red"),
                    declaration("opacity", "0.5"),
                ],
            },
            StyleRule {
                selectors: vec!["span".to_owned()],
                layer: None,
                declarations: vec![
                    declaration("background-color", "lime"),
                    declaration("color", "red"),
                    declaration("opacity", "0.5"),
                ],
            },
        ]
    };
    let child_group = PageRasterizer::new(&nodes, &styles_for("1")).snapshot_pixels(160, 70);
    let nested_groups = PageRasterizer::new(&nodes, &styles_for("0.5")).snapshot_pixels(160, 70);
    let max_error = child_group
        .chunks_exact(4)
        .zip(nested_groups.chunks_exact(4))
        .flat_map(|(source, actual)| {
            source[..3]
                .iter()
                .zip(&actual[..3])
                .map(|(source, actual)| {
                    let expected = (255.0 + (f32::from(*source) - 255.0) * 0.5).round() as u8;
                    actual.abs_diff(expected)
                })
        })
        .max()
        .unwrap_or_default();
    assert!(max_error <= 1, "maximum channel error was {max_error}");
}

#[test]
fn overlapping_sibling_opacity_groups_compose_in_document_order() {
    let nodes = [
        UiNode::Element {
            tag: "p".to_owned(),
            class: None,
            id: Some("first".to_owned()),
            attributes: Vec::new(),
            children: vec![UiNode::Text("First".to_owned())],
        },
        UiNode::Element {
            tag: "p".to_owned(),
            class: None,
            id: Some("second".to_owned()),
            attributes: Vec::new(),
            children: vec![UiNode::Text("Second".to_owned())],
        },
    ];
    let stylesheet = |second_opacity: &str| {
        [
            StyleRule {
                selectors: vec!["p".to_owned()],
                layer: None,
                declarations: vec![
                    declaration("width", "90px"),
                    declaration("height", "44px"),
                    declaration("margin-top", "-20px"),
                ],
            },
            StyleRule {
                selectors: vec!["#first".to_owned()],
                layer: None,
                declarations: vec![
                    declaration("background-color", "red"),
                    declaration("color", "yellow"),
                    declaration("opacity", "0.5"),
                ],
            },
            StyleRule {
                selectors: vec!["#second".to_owned()],
                layer: None,
                declarations: vec![
                    declaration("background-color", "blue"),
                    declaration("border", "2px solid lime"),
                    declaration("box-shadow", "3px 3px 2px 1px green"),
                    declaration("color", "white"),
                    declaration("opacity", second_opacity),
                ],
            },
        ]
    };
    let second_hidden = PageRasterizer::new(&nodes, &stylesheet("0")).snapshot_pixels(160, 90);
    let second_nearly_opaque =
        PageRasterizer::new(&nodes, &stylesheet("0.999")).snapshot_pixels(160, 90);
    let both_translucent = PageRasterizer::new(&nodes, &stylesheet("0.5")).snapshot_pixels(160, 90);
    let max_error = second_hidden
        .chunks_exact(4)
        .zip(second_nearly_opaque.chunks_exact(4))
        .zip(both_translucent.chunks_exact(4))
        .flat_map(|((base, full), actual)| {
            base[..3]
                .iter()
                .zip(&full[..3])
                .zip(&actual[..3])
                .map(|((base, full), actual)| {
                    let expected = (f32::from(*base)
                        + (f32::from(*full) - f32::from(*base)) * (0.5 / 0.999))
                        .round() as u8;
                    actual.abs_diff(expected)
                })
        })
        .max()
        .unwrap_or_default();
    assert!(max_error <= 1, "maximum channel error was {max_error}");
}

#[test]
fn element_opacity_scales_text_decorations() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Underline".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![
            declaration("color", "transparent"),
            declaration("text-decoration-line", "underline"),
            declaration("text-decoration-color", "red"),
            declaration("opacity", "0.5"),
        ],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(160, 70);
    assert!(pixels.chunks_exact(4).any(|pixel| {
        pixel[0] == 255
            && (pixel[1] == 127 || pixel[1] == 128)
            && (pixel[2] == 127 || pixel[2] == 128)
    }));
}
