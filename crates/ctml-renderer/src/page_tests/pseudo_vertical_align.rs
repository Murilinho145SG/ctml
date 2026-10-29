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

#[test]
fn first_letter_vertical_align_keywords_use_parent_line_metrics() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Alpha beta".to_owned())],
    }];
    for value in [
        "middle",
        "text-top",
        "text-bottom",
        "top",
        "bottom",
        "super",
    ] {
        let stylesheet = [StyleRule {
            selectors: vec!["p::first-letter".to_owned()],
            layer: None,
            declarations: [
                ("font-size", "32px"),
                ("line-height", "40px"),
                ("vertical-align", value),
            ]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
        }];
        let mut page = PageRasterizer::new(&nodes, &stylesheet);
        let styled_pixels = page.snapshot_pixels(180, 100);
        let parent = &page.root_style;
        let pseudo = &page.runs[0].style;
        let expected = match value {
            "middle" => parent.font_relative_units.ex * parent.font_size / 2.0,
            "text-top" => {
                parent.font_relative_units.cap * parent.font_size
                    - pseudo.font_relative_units.cap * pseudo.font_size
            }
            "text-bottom" => (pseudo.font_size - parent.font_size) * 0.2,
            "top" => (parent.line_height - pseudo.line_height) / 2.0,
            "bottom" => (pseudo.line_height - parent.line_height) / 2.0,
            "super" => parent.font_size / 3.0,
            _ => unreachable!(),
        };
        assert_eq!(page.runs[0].text, "A");
        assert!(
            (pseudo.vertical_align_offset - expected).abs() < 0.01,
            "unexpected offset for {value}: {} != {expected}",
            pseudo.vertical_align_offset
        );
        assert_ne!(
            styled_pixels,
            PageRasterizer::new(&nodes, &[]).snapshot_pixels(180, 100),
            "{value} should change the rendered first letter"
        );
    }
}

#[test]
fn first_line_vertical_align_middle_shifts_the_formatted_line() {
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
                value: "middle".to_owned(),
            }],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let styled_pixels = page.snapshot_pixels(180, 120);
    let style = page.runs[0]
        .first_line_font_style
        .as_ref()
        .expect("first-line style should be retained");
    assert!(style.vertical_align_offset > 0.0);
    assert_ne!(
        styled_pixels,
        PageRasterizer::new(&nodes, &[]).snapshot_pixels(180, 120)
    );
}
