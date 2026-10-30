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
                property: "background".to_owned(),
                value: "yellow".to_owned(),
            },
            StyleDeclaration {
                property: "text-decoration-line".to_owned(),
                value: "underline".to_owned(),
            },
            StyleDeclaration {
                property: "text-decoration-color".to_owned(),
                value: "red".to_owned(),
            },
            StyleDeclaration {
                property: "text-decoration-style".to_owned(),
                value: "double".to_owned(),
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
    let red_rows = pixels
        .chunks_exact(4)
        .enumerate()
        .filter(|(_, pixel)| pixel[..3] == [255, 0, 0])
        .map(|(index, _)| index / 140)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(red_rows.len() >= 2, "double underline should draw two rows");
    assert!(pixels.chunks_exact(4).any(|pixel| pixel[..3] == [0, 0, 0]));
}

#[test]
fn first_line_font_size_changes_glyph_rendering() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("First formatted line".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-line".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "font-size".to_owned(),
            value: "30px".to_owned(),
        }],
    }];
    let mut styled_page = PageRasterizer::new(&nodes, &stylesheet);
    let mut plain_page = PageRasterizer::new(&nodes, &[]);
    assert_eq!(
        styled_page.cursor_spans[0]
            .first_line_font_style
            .as_ref()
            .unwrap()
            .font_size,
        30.0
    );
    assert_ne!(
        styled_page.snapshot_pixels(180, 70),
        plain_page.snapshot_pixels(180, 70)
    );
}

#[test]
fn first_line_text_shadow_is_rasterized() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("First formatted line".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-line".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "text-shadow".to_owned(),
            value: "3px 0px red".to_owned(),
        }],
    }];
    let mut shadowed_page = PageRasterizer::new(&nodes, &stylesheet);
    let mut plain_page = PageRasterizer::new(&nodes, &[]);
    assert_ne!(
        shadowed_page.snapshot_pixels(180, 70),
        plain_page.snapshot_pixels(180, 70)
    );
}

#[test]
fn first_line_decoration_skip_ink_changes_underline_pixels() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("gyp descenders".to_owned())],
    }];
    let make_page = |skip_ink: &str| {
        let stylesheet = [StyleRule {
            selectors: vec!["p::first-line".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "text-decoration-line".to_owned(),
                    value: "underline".to_owned(),
                },
                StyleDeclaration {
                    property: "text-decoration-color".to_owned(),
                    value: "red".to_owned(),
                },
                StyleDeclaration {
                    property: "text-decoration-skip-ink".to_owned(),
                    value: skip_ink.to_owned(),
                },
            ],
        }];
        PageRasterizer::new(&nodes, &stylesheet)
    };
    let mut no_skip_page = make_page("none");
    let mut skip_all_page = make_page("all");
    assert_ne!(
        no_skip_page.snapshot_pixels(180, 70),
        skip_all_page.snapshot_pixels(180, 70)
    );
}

#[test]
fn first_letter_decoration_skip_ink_changes_underline_pixels() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("gyp descenders".to_owned())],
    }];
    let make_page = |skip_ink: &str| {
        let stylesheet = [StyleRule {
            selectors: vec!["p::first-letter".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "text-decoration-line".to_owned(),
                    value: "underline".to_owned(),
                },
                StyleDeclaration {
                    property: "text-decoration-color".to_owned(),
                    value: "red".to_owned(),
                },
                StyleDeclaration {
                    property: "text-decoration-skip-ink".to_owned(),
                    value: skip_ink.to_owned(),
                },
            ],
        }];
        PageRasterizer::new(&nodes, &stylesheet)
    };
    let mut no_skip_page = make_page("none");
    let mut skip_all_page = make_page("all");
    assert_ne!(
        no_skip_page.snapshot_pixels(180, 70),
        skip_all_page.snapshot_pixels(180, 70)
    );
}

#[test]
fn first_line_underline_offset_moves_decoration() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Underline offset".to_owned())],
    }];
    let make_page = |offset: Option<&str>| {
        let mut declarations = vec![
            StyleDeclaration {
                property: "text-decoration-line".to_owned(),
                value: "underline".to_owned(),
            },
            StyleDeclaration {
                property: "text-decoration-color".to_owned(),
                value: "red".to_owned(),
            },
        ];
        if let Some(offset) = offset {
            declarations.push(StyleDeclaration {
                property: "text-underline-offset".to_owned(),
                value: offset.to_owned(),
            });
        }
        let stylesheet = [StyleRule {
            selectors: vec!["p::first-line".to_owned()],
            layer: None,
            declarations,
        }];
        PageRasterizer::new(&nodes, &stylesheet)
    };
    let red_rows = |pixels: Vec<u8>| {
        pixels
            .chunks_exact(4)
            .enumerate()
            .filter(|(_, pixel)| pixel[..3] == [255, 0, 0])
            .map(|(index, _)| index / 180)
            .collect::<std::collections::BTreeSet<_>>()
    };
    let mut default_page = make_page(None);
    let mut offset_page = make_page(Some("8px"));
    assert_ne!(
        red_rows(default_page.snapshot_pixels(180, 70)),
        red_rows(offset_page.snapshot_pixels(180, 70))
    );
}

#[test]
fn text_emphasis_shorthand_respects_source_order_with_longhands() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Marked".to_owned())],
    }];
    let make_page = |shorthand_first: bool| {
        let shorthand = StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "text-emphasis".to_owned(),
                value: "filled dot red".to_owned(),
            }],
        };
        let longhand = StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "text-emphasis-color".to_owned(),
                value: "blue".to_owned(),
            }],
        };
        let stylesheet = if shorthand_first {
            vec![shorthand, longhand]
        } else {
            vec![longhand, shorthand]
        };
        PageRasterizer::new(&nodes, &stylesheet)
    };
    let shorthand_then_longhand = make_page(true);
    let longhand_then_shorthand = make_page(false);
    assert_eq!(
        shorthand_then_longhand.runs[0].style.text_emphasis_color,
        Some([0, 0, 255, 255])
    );
    assert_eq!(
        longhand_then_shorthand.runs[0].style.text_emphasis_color,
        Some([255, 0, 0, 255])
    );
}

#[test]
fn font_variant_shorthand_respects_source_order_with_longhands() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Small caps".to_owned())],
    }];
    let make_page = |shorthand_first: bool| {
        let shorthand = StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-variant".to_owned(),
                value: "normal".to_owned(),
            }],
        };
        let longhand = StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-variant-caps".to_owned(),
                value: "small-caps".to_owned(),
            }],
        };
        let stylesheet = if shorthand_first {
            vec![shorthand, longhand]
        } else {
            vec![longhand, shorthand]
        };
        PageRasterizer::new(&nodes, &stylesheet)
    };
    let shorthand_then_longhand = make_page(true);
    let longhand_then_shorthand = make_page(false);
    assert!(
        shorthand_then_longhand
            .runs
            .iter()
            .any(|run| run.style.font_scale < 1.0)
    );
    assert!(
        longhand_then_shorthand
            .runs
            .iter()
            .all(|run| run.style.font_scale == 1.0)
    );
}
