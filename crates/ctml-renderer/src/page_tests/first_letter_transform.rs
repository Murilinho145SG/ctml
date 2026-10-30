use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn first_letter_text_transform_uses_source_text_and_only_changes_the_letter() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Hello".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
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
                property: "text-transform".to_owned(),
                value: "lowercase".to_owned(),
            }],
        },
    ];
    let page = PageRasterizer::new(&nodes, &stylesheet);

    assert_eq!(page.runs[0].text, "h");
    assert_eq!(page.runs[1].text, "ELLO");

    let expansion_nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("ßeta".to_owned())],
    }];
    let expansion_page = PageRasterizer::new(&expansion_nodes, &stylesheet);
    assert_eq!(expansion_page.runs[0].text, "ß");
    assert_eq!(expansion_page.runs[1].text, "ETA");

    let standalone_nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("ß".to_owned())],
    }];
    let standalone_page = PageRasterizer::new(&standalone_nodes, &stylesheet);
    assert_eq!(standalone_page.runs[0].text, "ß");
}

#[test]
fn first_letter_includes_associated_unicode_punctuation_and_space() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("« H!)ello".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-letter".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "font-weight".to_owned(),
                value: "700".to_owned(),
            },
            StyleDeclaration {
                property: "color".to_owned(),
                value: "lime".to_owned(),
            },
        ],
    }];
    let page = PageRasterizer::new(&nodes, &stylesheet);

    assert_eq!(
        page.runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>(),
        "« H!)ello"
    );
    assert_eq!(
        page.runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<Vec<_>>(),
        ["«", " ", "H!)", "ello"]
    );
    assert_eq!(page.runs[0].style.weight, page.runs[1].style.weight);
    assert_eq!(page.runs[1].style.weight, page.runs[2].style.weight);
    assert_ne!(page.runs[2].style.weight, page.runs[3].style.weight);
    assert_eq!(
        page.cursor_spans
            .iter()
            .filter(|span| span.first_letter_foreground == Some([0, 255, 0, 255]))
            .map(|span| span.character)
            .collect::<String>(),
        "« H!)"
    );
}

#[test]
fn first_letter_transform_changes_the_letter_but_preserves_punctuation() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("“H!”ello".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
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
                property: "text-transform".to_owned(),
                value: "lowercase".to_owned(),
            }],
        },
    ];
    let page = PageRasterizer::new(&nodes, &stylesheet);

    assert_eq!(page.runs[0].text, "“h!”");
    assert_eq!(page.runs[1].text, "ELLO");
}

#[test]
fn first_letter_inherits_first_line_font_style_properties() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Alpha".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p::first-line".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-size".to_owned(),
                value: "30px".to_owned(),
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
    let page = PageRasterizer::new(&nodes, &stylesheet);

    assert_eq!(page.runs[0].text, "A");
    assert_eq!(page.runs[0].style.font_size, 30.0);
    assert_ne!(page.runs[0].style.weight, page.runs[1].style.weight);
}
