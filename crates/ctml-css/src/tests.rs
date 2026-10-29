use super::{ElementIdentity, parse_stylesheet, selector_matches, selector_specificity};

#[test]
fn parses_rules_groups_declarations_and_important_values() {
    let stylesheet = parse_stylesheet("p.notice, #target { color: red !important; }").unwrap();

    assert_eq!(stylesheet.rules.len(), 1);
    assert_eq!(stylesheet.rules[0].selectors, ["p.notice", "#target"]);
    assert_eq!(stylesheet.rules[0].declarations[0].property, "color");
    assert_eq!(stylesheet.rules[0].declarations[0].value, "red !important");
}

#[test]
fn matches_type_class_id_descendant_and_child_selectors() {
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
        "body p.highlighted",
        "body > main.content > p#message.notice",
    ] {
        assert!(
            selector_matches(selector, &paragraph, &ancestors),
            "{selector}"
        );
    }
    assert!(!selector_matches("body > p", &paragraph, &ancestors));
    assert!(!selector_matches("article", &paragraph, &ancestors));
}

#[test]
fn computes_selector_specificity_by_id_class_and_type() {
    assert_eq!(
        selector_specificity("body main.content > p#target"),
        (1, 1, 3)
    );
    assert_eq!(selector_specificity("#target.notice"), (1, 1, 0));
    assert_eq!(selector_specificity("p"), (0, 0, 1));
    assert_eq!(selector_specificity("[data-label=\"#.value\"]"), (0, 1, 0));
}

#[test]
fn matches_attribute_selector_operators_and_case_flags() {
    let element = ElementIdentity {
        tag: "input".to_owned(),
        class: None,
        id: None,
    };
    let attributes = [
        ("disabled".to_owned(), None),
        ("type".to_owned(), Some("TeXt".to_owned())),
        ("class".to_owned(), Some("field active".to_owned())),
        ("lang".to_owned(), Some("en-US".to_owned())),
        ("data-prefix".to_owned(), Some("pre-value".to_owned())),
        ("data-suffix".to_owned(), Some("value-tail".to_owned())),
        ("title".to_owned(), Some("A Friendly World".to_owned())),
    ];

    for selector in [
        "input[disabled]",
        "[type=TeXt]",
        "[type=text i]",
        "[class~=active]",
        "[lang|=en]",
        "[data-prefix^=pre]",
        "[data-suffix$=tail]",
        "[title*=friendly i]",
    ] {
        assert!(
            super::selector_matches_with_attributes(selector, &element, &[], &attributes),
            "{selector}"
        );
    }
    assert!(!super::selector_matches_with_attributes(
        "[type=text]",
        &element,
        &[],
        &attributes
    ));
}

#[test]
fn matches_attribute_selectors_on_ancestors() {
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };
    let ancestors = [body];
    let ancestor_attributes = vec![("data-role".to_owned(), Some("main".to_owned()))];

    assert!(super::selector_matches_with_context(
        "body[data-role=main] p",
        &paragraph,
        &ancestors,
        &[],
        &[ancestor_attributes.clone()],
    ));
    assert!(!super::selector_matches_with_context(
        "body[data-role=sidebar] p",
        &paragraph,
        &ancestors,
        &[],
        &[ancestor_attributes],
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
