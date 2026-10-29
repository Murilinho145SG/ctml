use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

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
}
