use super::*;

#[test]
fn matches_structural_pseudo_classes_for_children_and_types() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };
    let position = SelectorPosition {
        child_index: 4,
        child_count: 4,
        type_index: 2,
        type_count: 2,
        is_empty: true,
        is_root: false,
        ..SelectorPosition::default()
    };
    assert!(!selector_matches(":last-child", &current, &[]));
    assert!(!selector_matches(":nth-last-child(1)", &current, &[]));
    for selector in [
        ":last-child",
        ":nth-child(even)",
        ":nth-child(2n)",
        ":nth-last-child(odd)",
        ":nth-of-type(2)",
        ":nth-last-of-type(-n+2)",
        ":empty",
        ":not(:first-child)",
    ] {
        assert!(
            super::selector_matches_with_tree_context(
                selector,
                &current,
                &[],
                &[],
                &[],
                &[],
                &[],
                position,
                &[],
            ),
            "{selector}"
        );
    }
    for selector in [
        ":first-child",
        ":only-child",
        ":only-of-type",
        ":nth-child(2n+1)",
        ":nth-of-type(odd)",
    ] {
        assert!(
            !super::selector_matches_with_tree_context(
                selector,
                &current,
                &[],
                &[],
                &[],
                &[],
                &[],
                position,
                &[],
            ),
            "{selector}"
        );
    }
}

#[test]
fn matches_root_and_structural_pseudo_classes_on_ancestors() {
    let root = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };
    assert!(super::selector_matches_with_tree_context(
        ":root > p:first-child",
        &current,
        &[root],
        &[],
        &[],
        &[],
        &[Vec::new()],
        SelectorPosition {
            child_index: 1,
            child_count: 2,
            type_index: 1,
            type_count: 2,
            ..SelectorPosition::default()
        },
        &[SelectorPosition {
            is_root: true,
            child_count: 1,
            ..SelectorPosition::default()
        }],
    ));
}

#[test]
fn preserves_commas_inside_attribute_values_when_splitting_selector_groups() {
    let stylesheet = parse_stylesheet("[data-label=\"a,b\"], p { color: red; }").unwrap();
    assert_eq!(stylesheet.rules[0].selectors, ["[data-label=\"a,b\"]", "p"]);
}

#[test]
fn supports_is_where_and_not_selector_functions() {
    let element = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("notice".to_owned()),
        id: Some("message".to_owned()),
    };

    for selector in [
        ":is(span, p.notice)",
        ":where(#message, p)",
        ":not(.disabled, [hidden])",
    ] {
        assert!(selector_matches(selector, &element, &[]), "{selector}");
    }
    assert!(!selector_matches(":not(.notice, p)", &element, &[]));
    assert_eq!(selector_specificity(":where(#message)"), (0, 0, 0));
    assert_eq!(selector_specificity(":is(.notice, #message)"), (1, 0, 0));
    assert_eq!(
        parse_stylesheet(".item:is(.active, .selected) { color: red; }")
            .unwrap()
            .rules[0]
            .selectors,
        [".item:is(.active, .selected)"]
    );
}
