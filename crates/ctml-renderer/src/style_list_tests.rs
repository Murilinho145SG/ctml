use super::compile_runs_with_font_system;
use crate::{StyleDeclaration, StyleRule, UiNode};

fn compile_runs(
    nodes: &[UiNode],
    stylesheet: &[StyleRule],
    hovered_path: &[usize],
    active_path: &[usize],
    focused_path: &[usize],
    keyboard_focus: bool,
) -> (Vec<super::TextRun>, super::TextStyle) {
    compile_runs_with_font_system(
        nodes,
        stylesheet,
        hovered_path,
        active_path,
        focused_path,
        keyboard_focus,
        None,
        ctml_css::ViewportSize::default(),
    )
}

#[test]
fn list_items_render_bullets_and_ordered_numbers() {
    let nodes = [
        list_container("ul", &["apple", "pear"]),
        list_container("ol", &["first", "second"]),
    ];

    let (runs, _) = compile_runs(&nodes, &[], &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "• apple\n• pear\n1. first\n2. second"
    );
}

#[test]
fn list_style_type_inherits_shapes_and_alphabetic_numbering() {
    let nodes = [
        list_container("ul", &["apple", "pear"]),
        list_container("ol", &["first", "second"]),
    ];
    let stylesheet = [("ul", "square"), ("ol", "upper-alpha")].map(|(selector, value)| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "list-style-type".to_owned(),
            value: value.to_owned(),
        }],
    });

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "▪ apple\n▪ pear\nA. first\nB. second"
    );
}

#[test]
fn list_style_type_formats_roman_greek_and_leading_zero_markers() {
    for (style_type, count, expected) in [
        ("lower-roman", 4, "i. ii. iii. iv. "),
        ("upper-roman", 4, "I. II. III. IV. "),
        ("lower-greek", 3, "α. β. γ. "),
        ("decimal-leading-zero", 3, "01. 02. 03. "),
    ] {
        let labels = (0..count)
            .map(|index| format!("item{index}"))
            .collect::<Vec<_>>();
        let nodes = [list_container(
            "ol",
            &labels.iter().map(String::as_str).collect::<Vec<_>>(),
        )];
        let stylesheet = [StyleRule {
            selectors: vec!["ol".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "list-style-type".to_owned(),
                value: style_type.to_owned(),
            }],
        }];
        let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);
        let text = runs.iter().map(|run| run.text.as_str()).collect::<String>();
        let expected_text = expected
            .split_whitespace()
            .zip(&labels)
            .map(|(marker, label)| format!("{marker} {label}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(text, expected_text, "unexpected {style_type} markers");
    }
}

#[test]
fn list_style_type_none_suppresses_markers_without_hiding_items() {
    let nodes = [list_container("ul", &["apple", "pear"])];
    let stylesheet = [StyleRule {
        selectors: vec!["ul".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "list-style-type".to_owned(),
            value: "none".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "apple\npear"
    );
}

#[test]
fn list_style_shorthand_and_type_longhand_follow_source_order() {
    let nodes = [list_container("ul", &["apple"])];
    let declarations = [("list-style-type", "square"), ("list-style", "upper-alpha")];
    let stylesheet = declarations.map(|(property, value)| StyleRule {
        selectors: vec!["ul".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    });
    let (shorthand_last, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);
    assert_eq!(
        shorthand_last
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>(),
        "A. apple"
    );

    let stylesheet =
        [("list-style", "upper-alpha"), ("list-style-type", "square")].map(|(property, value)| {
            StyleRule {
                selectors: vec!["ul".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: property.to_owned(),
                    value: value.to_owned(),
                }],
            }
        });
    let (longhand_last, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);
    assert_eq!(
        longhand_last
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>(),
        "▪ apple"
    );
}

fn list_container(tag: &str, labels: &[&str]) -> UiNode {
    UiNode::Element {
        tag: tag.to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: labels
            .iter()
            .map(|label| UiNode::Element {
                tag: "li".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text((*label).to_owned())],
            })
            .collect(),
    }
}
