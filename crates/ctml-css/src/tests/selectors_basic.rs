use super::*;

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
    assert_eq!(selector_specificity("main:has(> p.active)"), (0, 1, 2));
    assert_eq!(selector_specificity("p::before"), (0, 0, 2));
    assert_eq!(selector_specificity("p:before"), (0, 0, 2));
    assert_eq!(selector_specificity("p:after"), (0, 0, 2));
    assert_eq!(selector_specificity("p:first-line"), (0, 0, 2));
    assert_eq!(selector_specificity("p:first-letter"), (0, 0, 2));
    assert_eq!(
        selector_specificity("li:nth-child(2n of .item, #target)"),
        (1, 1, 1)
    );
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
