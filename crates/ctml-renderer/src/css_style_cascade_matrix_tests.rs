use super::*;

#[test]
fn cascade_uses_specificity_then_source_order_for_each_declaration() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: Some("target".to_owned()),
    };
    let rules = [
        ("p", "#0000ff"),
        (".sample", "#008000"),
        ("#target", "#ff0000"),
        ("p.sample", "#ffffff"),
    ]
    .map(|(selector, value)| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "color".to_owned(),
            value: value.to_owned(),
        }],
    });
    let mut style = TextStyle::default();
    apply_matching_styles(&mut style, &current, &[], &rules);
    assert_eq!(style.color, [255, 0, 0, 255]);
}

#[test]
fn important_declarations_precede_specificity_and_source_order() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: Some("target".to_owned()),
    };
    let rules = [
        ("#target", "#ff0000"),
        ("p", "#0000ff !IMPORTANT"),
        ("#target", "#008000"),
        (".sample", "#ffffff !important"),
    ]
    .map(|(selector, value)| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "color".to_owned(),
            value: value.to_owned(),
        }],
    });
    let mut style = TextStyle::default();

    apply_matching_styles(&mut style, &current, &[], &rules);

    assert_eq!(style.color, [255, 255, 255, 255]);
}

#[test]
fn cascade_layers_order_normal_and_important_declarations() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: Some("target".to_owned()),
    };
    let stylesheet = ctml_css::parse_stylesheet(
        "@layer low, high; @layer low { #target { color: red; } p { color: red !important; } } @layer high { p { color: blue; } #target { color: blue !important; } } p { color: green !important; }",
    )
    .unwrap();
    let mut style = TextStyle::default();

    apply_matching_styles(&mut style, &current, &[], &stylesheet.rules);

    assert_eq!(style.color, [255, 0, 0, 255]);
}

#[test]
fn later_layers_outrank_earlier_layers_before_specificity() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: Some("target".to_owned()),
    };
    let stylesheet = ctml_css::parse_stylesheet(
        "@layer low, high; @layer low { #target { color: red; } } @layer high { p { color: blue; } }",
    )
    .unwrap();
    let mut style = TextStyle::default();

    apply_matching_styles(&mut style, &current, &[], &stylesheet.rules);

    assert_eq!(style.color, [0, 0, 255, 255]);
}

#[test]
fn unlayered_normal_rules_outrank_layered_normal_rules() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: Some("target".to_owned()),
    };
    let stylesheet =
        ctml_css::parse_stylesheet("@layer theme { #target { color: red; } } p { color: blue; }")
            .unwrap();
    let mut style = TextStyle::default();

    apply_matching_styles(&mut style, &current, &[], &stylesheet.rules);

    assert_eq!(style.color, [0, 0, 255, 255]);
}

#[test]
fn revert_layer_uses_the_previous_layer_for_normal_and_important_rules() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };
    let stylesheet = ctml_css::parse_stylesheet(
        "@layer low, high; @layer low { p { color: red; color: red !important; } } @layer high { p { color: blue; color: revert-layer; color: blue !important; color: revert-layer !important; } }",
    )
    .unwrap();
    let mut style = TextStyle::default();

    apply_matching_styles(&mut style, &current, &[], &stylesheet.rules);

    assert_eq!(style.color, [255, 0, 0, 255]);
}

#[test]
fn unlayered_revert_layer_falls_back_to_the_latest_named_layer() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };
    let stylesheet = ctml_css::parse_stylesheet(
        "@layer theme { p { color: red; } } p { color: blue; } p { color: revert-layer; }",
    )
    .unwrap();
    let mut style = TextStyle::default();

    apply_matching_styles(&mut style, &current, &[], &stylesheet.rules);

    assert_eq!(style.color, [255, 0, 0, 255]);
}

#[test]
fn attribute_selectors_apply_through_the_style_cascade() {
    let current = ElementIdentity {
        tag: "input".to_owned(),
        class: None,
        id: None,
    };
    let attributes = [("type".to_owned(), Some("TeXt".to_owned()))];
    let rules = [
        StyleRule {
            selectors: vec!["input[type=text i]".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "color".to_owned(),
                value: "#008000".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["input[type=password]".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-weight".to_owned(),
                value: "700".to_owned(),
            }],
        },
    ];
    let mut style = TextStyle::default();

    apply_matching_styles_with_attributes(&mut style, &current, &[], &attributes, &rules);

    assert_eq!(style.color, [0, 128, 0, 255]);
    assert_eq!(style.weight, Weight::NORMAL);
}

#[test]
fn css_wide_inherit_initial_and_unset_keywords_resolve_supported_properties() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };
    let rules = [
        StyleRule {
            selectors: vec![".sample".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "color".to_owned(),
                    value: "#ff0000".to_owned(),
                },
                StyleDeclaration {
                    property: "font-weight".to_owned(),
                    value: "700".to_owned(),
                },
                StyleDeclaration {
                    property: "font-size".to_owned(),
                    value: "32px".to_owned(),
                },
                StyleDeclaration {
                    property: "background-color".to_owned(),
                    value: "#000000".to_owned(),
                },
                StyleDeclaration {
                    property: "text-decoration-line".to_owned(),
                    value: "underline".to_owned(),
                },
            ],
        },
        StyleRule {
            selectors: vec![".sample".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "color".to_owned(),
                    value: "initial".to_owned(),
                },
                StyleDeclaration {
                    property: "font-weight".to_owned(),
                    value: "inherit".to_owned(),
                },
                StyleDeclaration {
                    property: "font-size".to_owned(),
                    value: "unset".to_owned(),
                },
                StyleDeclaration {
                    property: "background-color".to_owned(),
                    value: "unset".to_owned(),
                },
                StyleDeclaration {
                    property: "text-decoration-line".to_owned(),
                    value: "unset".to_owned(),
                },
            ],
        },
    ];
    let inherited = TextStyle {
        color: [0, 0, 255, 255],
        weight: Weight(300),
        font_size: 24.0,
        background: [12, 34, 56],
        ..TextStyle::default()
    };

    let style = resolve_style(&inherited, &current, &[], &rules);

    assert_eq!(style.color, [0, 0, 0, 255]);
    assert_eq!(style.weight, Weight(300));
    assert_eq!(style.font_size, 24.0);
    assert_eq!(style.background, [255, 255, 255]);
    assert!(!style.underline);
}

#[test]
fn selector_matrix_covers_type_class_id_universal_compound_and_descendant() {
    use super::super::selector_matches;

    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let main = ElementIdentity {
        tag: "main".to_owned(),
        class: Some("content".to_owned()),
        id: Some("root".to_owned()),
    };
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("notice highlighted".to_owned()),
        id: Some("message".to_owned()),
    };
    let ancestors = [body, main];

    for selector in [
        "p",
        ".notice",
        "#message",
        "*",
        "p.notice",
        "p.notice.highlighted",
        "body main p",
    ] {
        assert!(
            selector_matches(selector, &paragraph, &ancestors),
            "{selector}"
        );
    }
    assert!(!selector_matches("#other", &paragraph, &ancestors));
    assert!(!selector_matches("article", &paragraph, &ancestors));
}

#[test]
fn selector_group_matches_the_most_specific_member_only_once() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: Some("target".to_owned()),
    };
    let rule = StyleRule {
        selectors: vec!["p".to_owned(), ".sample".to_owned(), "#target".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "font-weight".to_owned(),
            value: "700".to_owned(),
        }],
    };
    let style = resolve_style(&TextStyle::default(), &current, &[], &[rule]);
    assert_eq!(style.weight, Weight::BOLD);
}
