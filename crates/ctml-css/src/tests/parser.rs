use super::*;

#[test]
fn parses_rules_groups_declarations_and_important_values() {
    let stylesheet = parse_stylesheet("p.notice, #target { color: red !important; }").unwrap();

    assert_eq!(stylesheet.rules.len(), 1);
    assert_eq!(stylesheet.rules[0].selectors, ["p.notice", "#target"]);
    assert_eq!(stylesheet.rules[0].declarations[0].property, "color");
    assert_eq!(stylesheet.rules[0].declarations[0].value, "red !important");
}

#[test]
fn parses_named_ordered_and_nested_layers() {
    let stylesheet = parse_stylesheet(
        "@layer base, theme; @layer components { p { color: red; } @layer buttons { p { color: blue; } } } @layer theme { p { color: green; } }",
    )
    .unwrap();

    assert_eq!(
        stylesheet.layers,
        ["base", "theme", "components.buttons", "components"]
    );
    assert_eq!(stylesheet.rules.len(), 3);
    assert_eq!(stylesheet.rules[0].layer, Some(3));
    assert_eq!(stylesheet.rules[1].layer, Some(2));
    assert_eq!(stylesheet.rules[2].layer, Some(1));
}

#[test]
fn parses_anonymous_layers_in_source_order() {
    let stylesheet = parse_stylesheet(
        "@layer base; @layer { p { color: red; } @layer { p { color: blue; } } } @layer theme { p { color: green; } }",
    )
    .expect("anonymous layers should be supported");

    assert_eq!(
        stylesheet.layers,
        [
            "base",
            "<anonymous-layer-0>.<anonymous-layer-1>",
            "<anonymous-layer-0>",
            "theme"
        ]
    );
    assert_eq!(stylesheet.rules[0].layer, Some(2));
    assert_eq!(stylesheet.rules[1].layer, Some(1));
    assert_eq!(stylesheet.rules[2].layer, Some(3));
}

#[test]
fn parses_escaped_layer_identifiers_without_confusing_path_separators() {
    let stylesheet = parse_stylesheet(
        r"@layer \66 oo; @layer foo\ bar { p { color: red; } } @layer foo.bar { p { color: blue; } } @layer foo\.bar { p { color: green; } }",
    )
    .expect("escaped layer identifiers should parse");

    assert_eq!(
        stylesheet.layers,
        ["foo.bar", "foo", "foo\\ bar", "foo\\.bar"]
    );
    assert_eq!(stylesheet.rules[0].layer, Some(2));
    assert_eq!(stylesheet.rules[1].layer, Some(0));
    assert_eq!(stylesheet.rules[2].layer, Some(3));
    assert!(parse_stylesheet("@layer foo,;").is_err());
}

#[test]
fn rejects_malformed_layer_statements() {
    assert!(parse_stylesheet("@layer first,,second;").is_err());
    assert!(parse_stylesheet("@layer -1invalid;").is_err());
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
fn preserves_custom_property_name_casing() {
    let stylesheet =
        parse_stylesheet("p { --Theme: red; --theme: blue; --empty: ; COLOR: var(--Theme); }")
            .unwrap();

    let declarations = &stylesheet.rules[0].declarations;
    assert_eq!(declarations[0].property, "--Theme");
    assert_eq!(declarations[1].property, "--theme");
    assert_eq!(declarations[2].property, "--empty");
    assert_eq!(declarations[2].value, "");
    assert_eq!(declarations[3].property, "color");
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
fn decodes_escaped_identifiers_in_values_and_recovers_after_bad_strings() {
    let stylesheet = parse_stylesheet(
        "p { color: r\\65 d; content: \"broken\n; background-image: url(bad url); color: blue; }",
    )
    .expect("bad tokens should invalidate only their declarations");

    assert_eq!(stylesheet.rules[0].declarations.len(), 2);
    assert_eq!(stylesheet.rules[0].declarations[0].value, "red");
    assert_eq!(stylesheet.rules[0].declarations[1].property, "color");
    assert_eq!(stylesheet.rules[0].declarations[1].value, "blue");
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
