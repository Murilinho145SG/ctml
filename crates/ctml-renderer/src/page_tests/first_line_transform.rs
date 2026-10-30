use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn first_line_inline_margin_and_padding_adjust_line_advance() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("AB".to_owned())],
    }];
    let line_width = |declarations: Vec<StyleDeclaration>| {
        let stylesheet = [StyleRule {
            selectors: vec!["p::first-line".to_owned()],
            layer: None,
            declarations,
        }];
        let mut page = PageRasterizer::new(&nodes, &stylesheet);
        page.prepare_layout(300, 80);
        page.buffer.layout_runs().next().unwrap().line_w
    };
    let natural_width = line_width(Vec::new());
    let spaced_width = line_width(vec![
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
        (spaced_width - natural_width - 16.0).abs() < 1.0,
        "5px inline padding and 3px inline margin on each side should add 16px, got {}px",
        spaced_width - natural_width
    );
    assert!(
        (negatively_margined_width - natural_width + 4.0).abs() < 1.0,
        "-2px inline margin on each side should remove 4px, got {}px",
        negatively_margined_width - natural_width
    );
}

#[test]
fn first_line_block_padding_extends_its_background_without_changing_line_height() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("alpha beta gamma delta epsilon".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: [("width", "72px"), ("line-height", "20px")]
                .into_iter()
                .map(|(property, value)| StyleDeclaration {
                    property: property.to_owned(),
                    value: value.to_owned(),
                })
                .collect(),
        },
        StyleRule {
            selectors: vec!["p::first-line".to_owned()],
            layer: None,
            declarations: [
                ("background-color", "red"),
                ("padding-block-start", "8px"),
                ("padding-block-end", "8px"),
            ]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(180, 120);
    let mut lines = page.buffer.layout_runs();
    let first_line = lines.next().expect("paragraph should have a line");
    let second_line = lines.next().expect("narrow paragraph should wrap");

    assert!((first_line.line_height - second_line.line_height).abs() < 1.0);
    assert!(
        pixels
            .chunks_exact(4)
            .filter(|pixel| pixel[..3] == [255, 0, 0])
            .count()
            > 100,
        "the first-line background should cover its expanded line box"
    );
}

#[test]
fn first_line_border_radius_rounds_its_background() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Rounded first line".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-line".to_owned()],
        layer: None,
        declarations: [("background-color", "red"), ("border-radius", "12px")]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(180, 70);
    assert!(page.runs.iter().any(|run| {
        run.first_line_font_style
            .as_ref()
            .is_some_and(|style| style.border_radius[0][0].is_some())
    }));
    let pixel_at = |x: usize, y: usize| &pixels[(y * 180 + x) * 4..][..3];

    let rounded_corner = pixel_at(1, 1);
    assert_eq!(rounded_corner[0], 255);
    assert!(rounded_corner[1] > 200 && rounded_corner[1] < 255);
    assert_eq!(pixel_at(20, 8), [255, 0, 0]);
}

#[test]
fn first_line_paints_background_images_only_on_the_formatted_line() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("alpha beta gamma delta epsilon".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "width".to_owned(),
                value: "72px".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p::first-line".to_owned()],
            layer: None,
            declarations: [
                ("background", "linear-gradient(red, red)"),
                ("border-radius", "12px"),
            ]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(180, 120);
    let mut lines = page.buffer.layout_runs();
    let first_line = lines.next().expect("paragraph should have a line");
    let second_line = lines.next().expect("narrow paragraph should wrap");
    assert!(
        page.runs.iter().any(|run| {
            run.first_line_font_style
                .as_ref()
                .is_some_and(|style| !style.background_images.is_empty())
        }),
        "the first-line pseudo style should retain its background image"
    );
    let first_line_bottom =
        (first_line.line_top + first_line.line_height + crate::page::PAGE_PADDING) as usize;
    let second_line_top = (second_line.line_top + crate::page::PAGE_PADDING) as usize;
    let red_in_first_line = pixels
        .chunks_exact(4)
        .enumerate()
        .filter(|(index, pixel)| index / 180 < first_line_bottom && pixel[..3] == [255, 0, 0])
        .count();
    let red_after_first_line = pixels
        .chunks_exact(4)
        .enumerate()
        .filter(|(index, pixel)| index / 180 >= second_line_top && pixel[..3] == [255, 0, 0])
        .count();
    assert!(
        red_in_first_line > 0,
        "the first formatted line should be painted"
    );
    assert_eq!(
        red_after_first_line, 0,
        "later lines must not get the pseudo background"
    );
    assert_eq!(&pixels[(1 * 180 + 10) * 4..][..3], [255, 255, 255]);
    assert_eq!(&pixels[(12 * 180 + 20) * 4..][..3], [255, 0, 0]);
}

#[test]
fn first_line_box_shadow_paints_outside_the_formatted_line() {
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
            property: "box-shadow".to_owned(),
            value: "0 0 0 3px red".to_owned(),
        }],
    }];
    let rendered = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(180, 70);
    let outside_line = &rendered[(6 * 180 + 20) * 4..][..3];

    assert_eq!(outside_line, [255, 0, 0]);
}

#[test]
fn first_line_outline_paints_outside_the_formatted_line() {
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
        declarations: [("outline", "2px solid red"), ("outline-offset", "2px")]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }];
    let rendered = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(180, 70);
    let above_line = &rendered[(4 * 180 + 20) * 4..][..3];

    assert_eq!(above_line, [255, 0, 0]);
}

#[test]
fn first_line_border_paints_around_the_formatted_line() {
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
            property: "border".to_owned(),
            value: "2px solid red".to_owned(),
        }],
    }];
    let mut baseline = PageRasterizer::new(&nodes, &[]);
    baseline.prepare_layout(180, 70);
    let baseline_width = baseline.buffer.layout_runs().next().unwrap().line_w;
    let mut bordered = PageRasterizer::new(&nodes, &stylesheet);
    bordered.prepare_layout(180, 70);
    let bordered_width = bordered.buffer.layout_runs().next().unwrap().line_w;
    let rendered = bordered.snapshot_pixels(180, 70);

    assert!(
        rendered
            .chunks_exact(4)
            .enumerate()
            .any(|(index, pixel)| index / 180 == 6 && pixel[..3] == [255, 0, 0]),
        "the first-line border should paint along the formatted line's top edge"
    );
    assert!(
        (bordered_width - baseline_width - 4.0).abs() < 1.0,
        "2px start and end borders should add 4px to the first-line advance, got {}px",
        bordered_width - baseline_width
    );
}

#[test]
fn first_line_text_transform_changes_only_the_formatted_first_line() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("alpha beta gamma delta epsilon".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "width".to_owned(),
                value: "72px".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p::first-line".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "text-transform".to_owned(),
                value: "uppercase".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p::first-letter".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-weight".to_owned(),
                value: "700".to_owned(),
            }],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.snapshot_pixels(180, 120);
    let rendered_text = page
        .runs
        .iter()
        .map(|run| run.text.as_str())
        .collect::<String>();

    assert!(rendered_text.starts_with("ALPHA"));
    assert!(rendered_text.contains("gamma"));
    assert!(!rendered_text.contains("GAMMA"));

    let inherited_stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "width".to_owned(),
                    value: "72px".to_owned(),
                },
                StyleDeclaration {
                    property: "text-transform".to_owned(),
                    value: "uppercase".to_owned(),
                },
            ],
        },
        StyleRule {
            selectors: vec!["p::first-line".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "text-transform".to_owned(),
                value: "lowercase".to_owned(),
            }],
        },
    ];
    let mut inherited_page = PageRasterizer::new(&nodes, &inherited_stylesheet);
    inherited_page.snapshot_pixels(180, 120);
    let inherited_text = inherited_page
        .runs
        .iter()
        .map(|run| run.text.as_str())
        .collect::<String>();

    assert!(inherited_text.starts_with("alpha"));
    assert!(inherited_text.contains("GAMMA"));
    assert!(!inherited_text.starts_with("ALPHA"));
}

#[test]
fn first_line_line_height_changes_only_the_first_line_box() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("alpha beta gamma delta epsilon".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "width".to_owned(),
                value: "72px".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p::first-line".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "line-height".to_owned(),
                value: "40px".to_owned(),
            }],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.snapshot_pixels(180, 160);
    let line_heights = page
        .buffer
        .layout_runs()
        .map(|line| line.line_height)
        .collect::<Vec<_>>();

    assert!(line_heights.len() > 1);
    assert!(line_heights[0] >= 40.0);
    assert!(line_heights[1] < line_heights[0]);
}

#[test]
fn first_line_word_spacing_changes_wrapping_without_spacing_later_lines() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text(
            "alpha beta gamma delta epsilon zeta".to_owned(),
        )],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "width".to_owned(),
                    value: "130px".to_owned(),
                },
                StyleDeclaration {
                    property: "word-spacing".to_owned(),
                    value: "10%".to_owned(),
                },
            ],
        },
        StyleRule {
            selectors: vec!["p::first-line".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "word-spacing".to_owned(),
                value: "20%".to_owned(),
            }],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.snapshot_pixels(240, 180);
    let spaced_spaces = page
        .runs
        .iter()
        .filter(|run| run.text == " " && run.style.word_spacing_percent == Some(20.0))
        .count();
    let normal_spaces = page
        .runs
        .iter()
        .filter(|run| run.text == " " && run.style.word_spacing_percent == Some(10.0))
        .count();

    assert!(spaced_spaces > 0);
    assert!(normal_spaces > 0);
    assert!(page.runs.iter().any(|run| {
        !run.text.trim().is_empty()
            && run.style.word_spacing_percent == Some(10.0)
            && run.style.letter_spacing == 0.0
    }));
}
