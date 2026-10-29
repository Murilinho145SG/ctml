use super::*;

#[test]
fn matches_inherited_language_and_direction_pseudo_classes() {
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
    let body_attributes = vec![
        ("lang".to_owned(), Some("en-US".to_owned())),
        ("dir".to_owned(), Some("rtl".to_owned())),
    ];
    let ancestor_attributes = [body_attributes.clone()];
    for selector in [":lang(en)", ":lang(\"EN-us\")", ":lang(*)", ":dir(rtl)"] {
        assert!(
            super::selector_matches_with_context(
                selector,
                &paragraph,
                std::slice::from_ref(&body),
                &[],
                &ancestor_attributes,
            ),
            "{selector}"
        );
    }
    for selector in [":lang(fr)", ":dir(ltr)"] {
        assert!(
            !super::selector_matches_with_context(
                selector,
                &paragraph,
                std::slice::from_ref(&body),
                &[],
                &ancestor_attributes,
            ),
            "{selector}"
        );
    }
    let current_attributes = [
        ("lang".to_owned(), Some("fr-CA".to_owned())),
        ("dir".to_owned(), Some("ltr".to_owned())),
    ];
    assert!(super::selector_matches_with_context(
        ":lang(fr):dir(ltr)",
        &paragraph,
        std::slice::from_ref(&body),
        &current_attributes,
        &ancestor_attributes,
    ));
    assert!(super::selector_matches(":dir(ltr)", &paragraph, &[]));
    assert!(!super::selector_matches_with_context(
        ":lang(en)",
        &paragraph,
        std::slice::from_ref(&body),
        &[("lang".to_owned(), Some(String::new()))],
        &ancestor_attributes,
    ));
}

#[test]
fn matches_link_pseudo_classes_for_ctml_anchors() {
    let anchor = ElementIdentity {
        tag: "a".to_owned(),
        class: None,
        id: None,
    };
    let href = [("href".to_owned(), Some("https://example.test".to_owned()))];
    for selector in ["a:link", "a:any-link", "a[href]:any-link"] {
        assert!(
            super::selector_matches_with_attributes(selector, &anchor, &[], &href),
            "{selector}"
        );
    }
    for selector in ["a:visited", "a:link"] {
        assert!(
            !super::selector_matches_with_attributes(selector, &anchor, &[], &[]),
            "{selector}"
        );
    }
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };
    assert!(!super::selector_matches_with_attributes(
        ":any-link",
        &paragraph,
        &[],
        &href,
    ));
}

#[test]
fn matches_focus_and_focus_within_from_selector_position() {
    let element = identity("a", None);
    let position = SelectorPosition {
        node_id: 2,
        is_focused: true,
        is_focus_within: true,
        ..SelectorPosition::default()
    };
    assert!(super::selector_matches_with_tree_context(
        ":focus",
        &element,
        &[],
        &[],
        &[],
        &[],
        &[],
        position,
        &[],
    ));
    assert!(super::selector_matches_with_tree_context(
        ":focus-within",
        &element,
        &[],
        &[],
        &[],
        &[],
        &[],
        position,
        &[],
    ));
    assert!(!super::selector_matches_with_tree_context(
        ":focus-visible",
        &element,
        &[],
        &[],
        &[],
        &[],
        &[],
        position,
        &[],
    ));
}

#[test]
fn matches_focus_visible_only_when_keyboard_modality_is_visible() {
    let element = identity("a", None);
    let keyboard_position = SelectorPosition {
        is_focused: true,
        is_focus_visible: true,
        ..SelectorPosition::default()
    };
    let pointer_position = SelectorPosition {
        is_focused: true,
        ..SelectorPosition::default()
    };

    for (position, expected) in [(keyboard_position, true), (pointer_position, false)] {
        assert_eq!(
            super::selector_matches_with_tree_context(
                ":focus-visible",
                &element,
                &[],
                &[],
                &[],
                &[],
                &[],
                position,
                &[],
            ),
            expected
        );
    }
}
