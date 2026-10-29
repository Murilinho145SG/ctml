use super::{
    ElementIdentity, SelectorPosition, SelectorSibling, parse_stylesheet, selector_matches,
    selector_specificity,
};

#[test]
fn parses_rules_groups_declarations_and_important_values() {
    let stylesheet = parse_stylesheet("p.notice, #target { color: red !important; }").unwrap();

    assert_eq!(stylesheet.rules.len(), 1);
    assert_eq!(stylesheet.rules[0].selectors, ["p.notice", "#target"]);
    assert_eq!(stylesheet.rules[0].declarations[0].property, "color");
    assert_eq!(stylesheet.rules[0].declarations[0].value, "red !important");
}

#[test]
fn parses_inline_declaration_lists_without_a_selector_block() {
    let declarations =
        super::parse_declarations("color: red; font-weight: 700 !important;").unwrap();

    assert_eq!(declarations.len(), 2);
    assert_eq!(declarations[0].property, "color");
    assert_eq!(declarations[1].value, "700 !important");
}

#[test]
fn strips_css_comments_and_accepts_a_utf8_bom() {
    let stylesheet = parse_stylesheet(
        "\u{feff}/* heading */ p/**/.notice { color: red/* inline */; content: \"/* literal */\"; }",
    )
    .unwrap();

    assert_eq!(stylesheet.rules[0].selectors, ["p.notice"]);
    assert_eq!(stylesheet.rules[0].declarations[0].value, "red");
    assert_eq!(
        stylesheet.rules[0].declarations[1].value,
        "\"/* literal */\""
    );
    let comment_error = parse_stylesheet("p {}\n/* unterminated").unwrap_err();
    assert!(comment_error.contains("line 2, column 1"));
}

#[test]
fn tokenizes_escaped_property_names_strings_and_recovers_bad_declarations() {
    let stylesheet = parse_stylesheet(
        "p { color red; \\63 olor: blue; content: \"a:b;c\"; } h1 { color: green; }",
    )
    .unwrap();

    assert_eq!(stylesheet.rules.len(), 2);
    assert_eq!(stylesheet.rules[0].declarations.len(), 2);
    assert_eq!(stylesheet.rules[0].declarations[0].property, "color");
    assert_eq!(stylesheet.rules[0].declarations[0].value, "blue");
    assert_eq!(stylesheet.rules[0].declarations[1].value, "\"a:b;c\"");
    assert_eq!(stylesheet.rules[1].declarations[0].value, "green");
}

#[test]
fn skips_malformed_declarations_and_preserves_unknown_properties() {
    let stylesheet =
        parse_stylesheet("p { color red; future-property: value; color: blue; }").unwrap();

    assert_eq!(stylesheet.rules[0].declarations.len(), 2);
    assert_eq!(
        stylesheet.rules[0].declarations[0].property,
        "future-property"
    );
    assert_eq!(stylesheet.rules[0].declarations[1].property, "color");
    assert_eq!(stylesheet.rules[0].declarations[1].value, "blue");
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

#[test]
fn expands_relative_imports_in_place_and_tracks_build_dependencies() {
    let root = temporary_directory("ordered-imports");
    std::fs::create_dir_all(&root).expect("temporary stylesheet root should be created");
    std::fs::write(root.join("tokens.css"), ".item { font-weight: 700; }")
        .expect("tokens stylesheet should be written");
    std::fs::write(
        root.join("base.css"),
        "@import \"tokens.css\";\n.item { color: red; }",
    )
    .expect("base stylesheet should be written");
    std::fs::write(
        root.join("main.css"),
        "@IMPORT url(\"base.css\");\n.item { color: blue; }",
    )
    .expect("main stylesheet should be written");

    let (stylesheet, dependencies) =
        super::parse_stylesheet_file(root.join("main.css"), &root).unwrap();

    assert_eq!(stylesheet.rules.len(), 3);
    assert_eq!(stylesheet.rules[0].declarations[0].value, "700");
    assert_eq!(stylesheet.rules[1].declarations[0].value, "red");
    assert_eq!(stylesheet.rules[2].declarations[0].value, "blue");
    assert_eq!(
        dependencies
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        ["main.css", "base.css", "tokens.css"]
    );
    std::fs::remove_dir_all(root).expect("temporary stylesheet root should be removed");
}

#[test]
fn rejects_import_cycles_and_paths_outside_the_root() {
    let root = temporary_directory("invalid-imports");
    std::fs::create_dir_all(&root).expect("temporary stylesheet root should be created");
    std::fs::write(root.join("first.css"), "@import \"second.css\";")
        .expect("first stylesheet should be written");
    std::fs::write(root.join("second.css"), "@import \"first.css\";")
        .expect("second stylesheet should be written");

    let cycle = super::parse_stylesheet_file(root.join("first.css"), &root)
        .expect_err("cyclic imports should be rejected");
    assert!(cycle.contains("stylesheet import cycle"));

    std::fs::write(root.join("escape.css"), "@import \"../outside.css\";")
        .expect("escape stylesheet should be written");
    let escape = super::parse_stylesheet_file(root.join("escape.css"), &root)
        .expect_err("imports outside the root should be rejected");
    assert!(escape.contains("stay inside the stylesheet root"));

    std::fs::write(
        root.join("external.css"),
        "@import url(\"https://example.com/site.css\");",
    )
    .expect("external stylesheet should be written");
    let external = super::parse_stylesheet_file(root.join("external.css"), &root)
        .expect_err("network imports should be rejected");
    assert!(external.contains("stay inside the stylesheet root"));

    std::fs::write(
        root.join("late.css"),
        ".item { color: red; } @import \"first.css\";",
    )
    .expect("late-import stylesheet should be written");
    let late = super::parse_stylesheet_file(root.join("late.css"), &root)
        .expect_err("imports after qualified rules should be rejected");
    assert!(late.contains("must precede qualified rules"));
    std::fs::remove_dir_all(root).expect("temporary stylesheet root should be removed");
}

#[test]
fn stylesheet_files_require_utf8_encoding() {
    let root = temporary_directory("encoding");
    std::fs::create_dir_all(&root).expect("temporary stylesheet root should be created");
    std::fs::write(root.join("invalid.css"), [0xff, 0xfe])
        .expect("invalid UTF-8 stylesheet should be written");

    let error = super::parse_stylesheet_file(root.join("invalid.css"), &root)
        .expect_err("non-UTF-8 stylesheet files should be rejected");
    assert!(error.contains("valid UTF-8"));
    std::fs::remove_dir_all(root).expect("temporary stylesheet root should be removed");
}

#[test]
fn reports_stylesheet_file_and_line_for_syntax_errors() {
    let root = temporary_directory("source-locations");
    std::fs::create_dir_all(&root).expect("temporary stylesheet root should be created");
    std::fs::write(root.join("broken.css"), "p { color: red; }\n@unsupported;")
        .expect("malformed stylesheet should be written");

    let error = super::parse_stylesheet_file(root.join("broken.css"), &root)
        .expect_err("unsupported malformed rules should be reported");
    assert!(error.contains("broken.css: line 2, column"), "{error}");

    std::fs::write(
        root.join("late-import.css"),
        ".item {}\n@import \"missing.css\";",
    )
    .expect("late import stylesheet should be written");
    let error = super::parse_stylesheet_file(root.join("late-import.css"), &root)
        .expect_err("late imports should include a diagnostic location");
    assert!(error.contains("late-import.css: line 2, column"), "{error}");

    std::fs::create_dir_all(root.join("nested"))
        .expect("nested stylesheet directory should be created");
    std::fs::write(root.join("entry.css"), "@import \"nested/broken.css\";")
        .expect("entry stylesheet should be written");
    std::fs::write(root.join("nested/broken.css"), ".item {}\n@unsupported;")
        .expect("nested malformed stylesheet should be written");
    let error = super::parse_stylesheet_file(root.join("entry.css"), &root)
        .expect_err("nested syntax errors should identify their source file");
    assert!(
        error.contains("nested/broken.css: line 2, column"),
        "{error}"
    );
    std::fs::remove_dir_all(root).expect("temporary stylesheet root should be removed");
}

fn temporary_directory(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("ctml-css-{name}-{}-{nonce}", std::process::id()))
}
