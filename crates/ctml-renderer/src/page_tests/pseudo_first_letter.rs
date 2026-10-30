use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

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
                property: "background".to_owned(),
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

#[test]
fn first_letter_border_paints_around_the_initial_glyph() {
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
        declarations: [("border", "2px solid red"), ("border-left-color", "blue")]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }];

    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(140, 70);
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 0, 0]),
        "the initial glyph should receive the pseudo-element border"
    );
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [0, 0, 255]),
        "a border side longhand should override the shorthand"
    );
}

#[test]
fn first_letter_box_shadow_paints_outside_the_initial_glyph_box() {
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
        declarations: [("box-shadow", "0 0 0 3px red"), ("color", "blue")]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }];
    let mut shadowed_page = PageRasterizer::new(&nodes, &stylesheet);
    let mut plain_page = PageRasterizer::new(&nodes, &[]);
    let shadowed_pixels = shadowed_page.snapshot_pixels(140, 70);
    let plain_pixels = plain_page.snapshot_pixels(140, 70);

    assert_ne!(shadowed_pixels, plain_pixels);
    assert!(
        shadowed_pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 0, 0]),
        "the initial glyph should receive its box shadow"
    );
}

#[test]
fn first_letter_border_radius_rounds_the_border_corner() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Wider text".to_owned())],
    }];
    let make_page = |radius: Option<&str>| {
        let mut declarations = vec![StyleDeclaration {
            property: "border".to_owned(),
            value: "3px solid red".to_owned(),
        }];
        if let Some(radius) = radius {
            declarations.push(StyleDeclaration {
                property: "border-radius".to_owned(),
                value: radius.to_owned(),
            });
        }
        let stylesheet = [StyleRule {
            selectors: vec!["p::first-letter".to_owned()],
            layer: None,
            declarations,
        }];
        let mut page = PageRasterizer::new(&nodes, &stylesheet);
        page.prepare_layout(140, 70);
        let line = page.buffer.layout_runs().next().unwrap();
        let glyph = line.glyphs.first().unwrap();
        let x = (glyph.x + crate::page::PAGE_PADDING).floor() as u32;
        let y = (line.line_top + crate::page::PAGE_PADDING).floor() as u32;
        let pixels = page.snapshot_pixels(140, 70);
        let pixel = &pixels[((y * 140 + x) * 4) as usize..][..3];
        pixel == [255, 0, 0]
    };

    assert!(
        make_page(None),
        "square border should paint its top-left corner"
    );
    assert!(
        !make_page(Some("50%")),
        "rounded border should leave its outer top-left corner clear"
    );
}

#[test]
fn first_letter_outline_paints_outside_the_initial_glyph() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Initial letter".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-letter".to_owned()],
        layer: None,
        declarations: [("outline", "2px solid red"), ("outline-offset", "2px")]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }];

    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(140, 70);
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 0, 0]),
        "outline on the initial glyph should paint outside its inline box"
    );
}

#[test]
fn first_letter_inline_margin_and_padding_adjust_line_advance() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("AB".to_owned())],
    }];
    let line_width = |declarations: Vec<StyleDeclaration>| {
        let stylesheet = [StyleRule {
            selectors: vec!["p::first-letter".to_owned()],
            layer: None,
            declarations,
        }];
        let mut page = PageRasterizer::new(&nodes, &stylesheet);
        page.prepare_layout(300, 80);
        page.buffer.layout_runs().next().unwrap().line_w
    };
    let natural_width = line_width(Vec::new());
    let padded_width = line_width(vec![
        StyleDeclaration {
            property: "padding-inline".to_owned(),
            value: "5px".to_owned(),
        },
        StyleDeclaration {
            property: "margin-inline".to_owned(),
            value: "3px".to_owned(),
        },
    ]);
    let negatively_margined_width = line_width(vec![StyleDeclaration {
        property: "margin-inline".to_owned(),
        value: "-2px".to_owned(),
    }]);

    assert!(
        (padded_width - natural_width - 16.0).abs() < 1.0,
        "5px inline padding and 3px inline margin on both sides should add 16px, got {}px",
        padded_width - natural_width
    );
    assert!(
        (negatively_margined_width - natural_width + 4.0).abs() < 1.0,
        "-2px inline margin on each side should remove 4px, got {}px",
        negatively_margined_width - natural_width
    );
}

#[test]
fn first_letter_block_padding_expands_its_background() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Initial letter".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-letter".to_owned()],
        layer: None,
        declarations: [("padding-block", "8px"), ("background-color", "red")]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(140, 70);
    let line_top =
        (page.buffer.layout_runs().next().unwrap().line_top + crate::page::PAGE_PADDING) as usize;
    let pixels = page.snapshot_pixels(140, 70);
    assert!(
        pixels
            .chunks_exact(4)
            .enumerate()
            .any(|(index, pixel)| index / 140 < line_top && pixel[..3] == [255, 0, 0]),
        "block padding should extend the first-letter background above the line box"
    );
}
