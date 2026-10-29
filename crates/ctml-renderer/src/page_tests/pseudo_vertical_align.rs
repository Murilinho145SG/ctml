use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn first_letter_vertical_align_applies_signed_font_relative_offset() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Alpha beta".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-letter".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "vertical-align".to_owned(),
            value: "0.5em".to_owned(),
        }],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let styled_pixels = page.snapshot_pixels(180, 80);

    assert_eq!(page.runs[0].text, "A");
    assert!((page.runs[0].style.vertical_align_offset - 8.0).abs() < 0.01);
    assert_eq!(page.runs[1].style.vertical_align_offset, 0.0);
    assert_ne!(
        styled_pixels,
        PageRasterizer::new(&nodes, &[]).snapshot_pixels(180, 80)
    );
}

#[test]
fn first_line_vertical_align_applies_percentages_of_line_height() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Alpha beta gamma delta".to_owned())],
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
                property: "vertical-align".to_owned(),
                value: "25%".to_owned(),
            }],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let styled_pixels = page.snapshot_pixels(180, 120);

    let first_line_style = page.runs[0]
        .first_line_font_style
        .as_ref()
        .expect("first-line style should be retained");
    assert!(
        (first_line_style.vertical_align_offset - first_line_style.line_height * 0.25).abs() < 0.01
    );
    assert_ne!(
        styled_pixels,
        PageRasterizer::new(&nodes, &[]).snapshot_pixels(180, 120)
    );
}
