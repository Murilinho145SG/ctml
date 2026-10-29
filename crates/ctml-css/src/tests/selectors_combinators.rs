use super::*;

#[test]
fn matches_escaped_selector_identifiers_and_attribute_values() {
    let element = ElementIdentity {
        tag: "my:element".to_owned(),
        class: Some("foo:bar has,comma".to_owned()),
        id: Some("123".to_owned()),
    };
    let attributes = [("data-name".to_owned(), Some("blue: green".to_owned()))];
    for selector in [
        r"my\:element.foo\:bar#\31 23",
        r".has\,comma",
        r#"[data\-name="blue\:\ green"]"#,
    ] {
        assert!(
            super::selector_matches_with_attributes(selector, &element, &[], &attributes),
            "{selector}"
        );
    }
    assert_eq!(
        selector_specificity(r"my\:element.foo\:bar#\31 23"),
        (1, 1, 1)
    );
    assert_eq!(selector_specificity(r"p:n\6f t(.other)"), (0, 1, 1));
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };
    assert!(selector_matches(r"p:n\6f t(.other)", &paragraph, &[]));
}

#[test]
fn matches_adjacent_and_general_sibling_combinators_with_chains() {
    let previous = [
        SelectorSibling {
            element: ElementIdentity {
                tag: "h1".to_owned(),
                class: Some("title".to_owned()),
                id: None,
            },
            attributes: Vec::new(),
            position: SelectorPosition::default(),
            node_id: 0,
        },
        SelectorSibling {
            element: ElementIdentity {
                tag: "p".to_owned(),
                class: Some("intro".to_owned()),
                id: None,
            },
            attributes: Vec::new(),
            position: SelectorPosition::default(),
            node_id: 0,
        },
        SelectorSibling {
            element: ElementIdentity {
                tag: "hr".to_owned(),
                class: None,
                id: None,
            },
            attributes: Vec::new(),
            position: SelectorPosition::default(),
            node_id: 0,
        },
    ];
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("body".to_owned()),
        id: None,
    };

    for selector in ["hr + p", "h1 ~ p", "h1 + p.intro + hr + p.body"] {
        assert!(
            super::selector_matches_with_tree_context(
                selector,
                &current,
                &[],
                &[],
                &[],
                &previous,
                &[],
                SelectorPosition::default(),
                &[],
            ),
            "{selector}"
        );
    }
    assert!(!super::selector_matches_with_tree_context(
        "h1 + p.body",
        &current,
        &[],
        &[],
        &[],
        &previous,
        &[],
        SelectorPosition::default(),
        &[],
    ));
    assert!(!super::selector_matches_with_tree_context(
        "h1 ~ p.body + p",
        &current,
        &[],
        &[],
        &[],
        &previous,
        &[],
        SelectorPosition::default(),
        &[],
    ));
    assert!(!selector_matches("p +", &current, &[]));
}
