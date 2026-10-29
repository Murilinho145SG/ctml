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
            StyleDeclaration {
                property: "text-decoration-line".to_owned(),
                value: "line-through".to_owned(),
            },
            StyleDeclaration {
                property: "text-decoration-color".to_owned(),
                value: "red".to_owned(),
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
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 0, 0])
    );
    assert!(pixels.chunks_exact(4).any(|pixel| pixel[..3] == [0, 0, 0]));
}

#[test]
fn first_letter_text_shadow_is_rasterized() {
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
        declarations: vec![StyleDeclaration {
            property: "text-shadow".to_owned(),
            value: "3px 0px red".to_owned(),
        }],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    assert_eq!(page.runs[0].style.text_shadows.len(), 1);
    let shadowed_pixels = page.snapshot_pixels(140, 70);
    let mut plain_page = PageRasterizer::new(&nodes, &[]);
    assert_ne!(shadowed_pixels, plain_page.snapshot_pixels(140, 70));
}

#[test]
fn first_letter_text_emphasis_is_rendered() {
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
        declarations: vec![StyleDeclaration {
            property: "text-emphasis".to_owned(),
            value: "filled dot red".to_owned(),
        }],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(140, 70);
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 0, 0])
    );
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
