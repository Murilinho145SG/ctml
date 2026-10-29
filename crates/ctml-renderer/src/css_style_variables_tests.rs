use super::*;

fn element(tag: &str, class: &str) -> ElementIdentity {
    ElementIdentity {
        tag: tag.to_owned(),
        class: Some(class.to_owned()),
        id: None,
    }
}

#[test]
fn resolves_custom_properties_and_nested_fallbacks_in_supported_values() {
    let declarations = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "--Primary".to_owned(),
                value: "#336699".to_owned(),
            },
            StyleDeclaration {
                property: "--accent".to_owned(),
                value: "var(--missing, var(--Primary))".to_owned(),
            },
            StyleDeclaration {
                property: "color".to_owned(),
                value: "var(--accent)".to_owned(),
            },
            StyleDeclaration {
                property: "text-transform".to_owned(),
                value: "var(--case, uppercase)".to_owned(),
            },
        ],
    };
    let style = resolve_style(
        &TextStyle::default(),
        &element("p", "sample"),
        &[],
        &[declarations],
    );

    assert_eq!(style.color, [51, 102, 153, 255]);
    assert_eq!(style.text_transform, TextTransform::Uppercase);
    assert_eq!(style.custom_properties["--Primary"], "#336699");
    assert!(!style.custom_properties.contains_key("--primary"));
}

#[test]
fn computes_custom_properties_before_inheriting_them() {
    let parent_rule = StyleRule {
        selectors: vec![".parent".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "--base".to_owned(),
                value: "red".to_owned(),
            },
            StyleDeclaration {
                property: "--derived".to_owned(),
                value: "var(--base)".to_owned(),
            },
        ],
    };
    let parent = resolve_style(
        &TextStyle::default(),
        &element("div", "parent"),
        &[],
        &[parent_rule],
    );
    let child_rule = StyleRule {
        selectors: vec![".child".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "--base".to_owned(),
                value: "blue".to_owned(),
            },
            StyleDeclaration {
                property: "color".to_owned(),
                value: "var(--derived)".to_owned(),
            },
        ],
    };
    let child = resolve_style(&parent, &element("span", "child"), &[], &[child_rule]);

    assert_eq!(child.color, [255, 0, 0, 255]);
    assert_eq!(child.custom_properties["--derived"], "red");
    assert_eq!(child.custom_properties["--base"], "blue");
}

#[test]
fn invalid_cycles_use_fallbacks_and_invalid_color_declarations_unset() {
    let inherited = TextStyle {
        color: [0, 0, 255, 255],
        ..TextStyle::default()
    };
    let declarations = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "--first".to_owned(),
                value: "var(--second, red)".to_owned(),
            },
            StyleDeclaration {
                property: "--second".to_owned(),
                value: "var(--first, green)".to_owned(),
            },
            StyleDeclaration {
                property: "color".to_owned(),
                value: "var(--first, #123456)".to_owned(),
            },
        ],
    };
    let fallback = resolve_style(
        &inherited,
        &element("p", "sample"),
        &[],
        &[declarations.clone()],
    );
    assert_eq!(fallback.color, [18, 52, 86, 255]);
    assert!(!fallback.custom_properties.contains_key("--first"));
    assert!(!fallback.custom_properties.contains_key("--second"));

    let invalid_color = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "color".to_owned(),
                value: "red".to_owned(),
            },
            StyleDeclaration {
                property: "color".to_owned(),
                value: "var(--missing)".to_owned(),
            },
        ],
    };
    let invalid = resolve_style(&inherited, &element("p", "sample"), &[], &[invalid_color]);
    assert_eq!(invalid.color, inherited.color);

    let empty_variable = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "--empty".to_owned(),
                value: String::new(),
            },
            StyleDeclaration {
                property: "color".to_owned(),
                value: "var(--empty, red)".to_owned(),
            },
        ],
    };
    let empty = resolve_style(&inherited, &element("p", "sample"), &[], &[empty_variable]);
    assert_eq!(empty.color, inherited.color);
}

#[test]
fn invalid_substituted_color_uses_unset_instead_of_an_earlier_color() {
    let mut inherited = TextStyle::default();
    inherited.color = [12, 34, 56, 255];
    let rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "--length".to_owned(),
                value: "12px".to_owned(),
            },
            StyleDeclaration {
                property: "color".to_owned(),
                value: "blue".to_owned(),
            },
            StyleDeclaration {
                property: "color".to_owned(),
                value: "var(--length)".to_owned(),
            },
        ],
    };

    let style = resolve_style(&inherited, &element("p", "sample"), &[], &[rule]);

    assert_eq!(style.color, inherited.color);
}
