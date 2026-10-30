use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn text_transform_applies_full_width_and_full_size_kana_to_text() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("ABC ぁｧ".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: [("text-transform", "uppercase full-width full-size-kana")]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }];
    let page = PageRasterizer::new(&nodes, &stylesheet);
    let rendered_text = page
        .runs
        .iter()
        .map(|run| run.text.as_str())
        .collect::<String>();

    assert_eq!(rendered_text, "ＡＢＣ あア");
}

#[test]
fn full_width_transforms_spaces_only_when_white_space_is_preserved() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("A B".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "white-space".to_owned(),
                value: "pre".to_owned(),
            },
            StyleDeclaration {
                property: "text-transform".to_owned(),
                value: "full-width".to_owned(),
            },
        ],
    }];
    let page = PageRasterizer::new(&nodes, &stylesheet);
    let rendered_text = page
        .runs
        .iter()
        .map(|run| run.text.as_str())
        .collect::<String>();

    assert_eq!(rendered_text, "Ａ\u{3000}Ｂ\n");
}

#[test]
fn first_line_text_transform_applies_full_width_and_full_size_kana() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("aぁｧ\nlater".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "white-space".to_owned(),
                value: "pre".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p::first-line".to_owned()],
            layer: None,
            declarations: [("text-transform", "uppercase full-width full-size-kana")]
                .into_iter()
                .map(|(property, value)| StyleDeclaration {
                    property: property.to_owned(),
                    value: value.to_owned(),
                })
                .collect(),
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(180, 100);
    let rendered_text = page
        .runs
        .iter()
        .map(|run| run.text.as_str())
        .collect::<String>();

    assert_eq!(rendered_text, "Ａあア\nlater\n");
}
