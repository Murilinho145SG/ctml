use super::*;

#[test]
fn all_shorthand_resets_supported_properties_in_cascade_order() {
    let inherited = TextStyle {
        family: "Open Sans".to_owned(),
        font_size: 21.0,
        weight: Weight(600),
        font_style: Style::Italic,
        font_variant_numeric: vec!["tabular-nums".to_owned()],
        color: [0, 0, 255, 255],
        background: [12, 34, 56],
        text_decoration_color: Some([255, 0, 0, 255]),
        decoration_thickness: Some(3.0),
        cursor: CursorIcon::Pointer,
        ..TextStyle::default()
    };
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };
    let reset_rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "all".to_owned(),
            value: "unset".to_owned(),
        }],
    };
    let reset = resolve_style(&inherited, &current, &[], &[reset_rule]);
    assert_eq!(reset.color, inherited.color);
    assert_eq!(reset.family, inherited.family);
    assert_eq!(reset.font_size, inherited.font_size);
    assert_eq!(reset.font_variant_numeric, inherited.font_variant_numeric);
    assert_eq!(reset.weight, inherited.weight);
    assert_eq!(reset.font_style, inherited.font_style);
    assert_eq!(reset.background, [255, 255, 255]);
    assert_eq!(reset.text_decoration_color, None);
    assert_eq!(reset.decoration_thickness, None);
    assert_eq!(reset.cursor, inherited.cursor);

    let initial_rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "all".to_owned(),
            value: "initial".to_owned(),
        }],
    };
    let initial = resolve_style(&inherited, &current, &[], &[initial_rule]);
    assert_eq!(initial.color, [0, 0, 0, 255]);
    assert_eq!(initial.family, "serif");
    assert_eq!(initial.font_size, 16.0);
    assert!(initial.font_variant_numeric.is_empty());
    assert_eq!(initial.weight, Weight::NORMAL);
    assert_eq!(initial.background, [255, 255, 255]);

    let inherit_rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "all".to_owned(),
            value: "inherit".to_owned(),
        }],
    };
    let inherited_style = resolve_style(&inherited, &current, &[], &[inherit_rule]);
    assert_eq!(inherited_style.color, inherited.color);
    assert_eq!(inherited_style.family, inherited.family);
    assert_eq!(inherited_style.weight, inherited.weight);
    assert_eq!(
        inherited_style.font_variant_numeric,
        inherited.font_variant_numeric
    );
    assert_eq!(inherited_style.background, inherited.background);

    let variable_all = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "--reset".to_owned(),
                value: "var(--missing, initial)".to_owned(),
            },
            StyleDeclaration {
                property: "all".to_owned(),
                value: "var(--reset)".to_owned(),
            },
        ],
    };
    let initial = resolve_style(&inherited, &current, &[], &[variable_all]);
    assert_eq!(initial.color, [0, 0, 0, 255]);
    assert_eq!(initial.family, "serif");
    assert_eq!(initial.background, [255, 255, 255]);

    let all_before_color = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "all".to_owned(),
                value: "unset".to_owned(),
            },
            StyleDeclaration {
                property: "color".to_owned(),
                value: "red".to_owned(),
            },
        ],
    };
    let color = resolve_style(&inherited, &current, &[], &[all_before_color.clone()]);
    assert_eq!(color.color, [255, 0, 0, 255]);

    let color_before_all = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "color".to_owned(),
                value: "red".to_owned(),
            },
            StyleDeclaration {
                property: "all".to_owned(),
                value: "unset".to_owned(),
            },
        ],
    };
    let color = resolve_style(&inherited, &current, &[], &[color_before_all]);
    assert_eq!(color.color, inherited.color);

    let rules = [
        StyleRule {
            selectors: vec![".sample".to_owned()],
            layer: Some(0),
            declarations: vec![
                StyleDeclaration {
                    property: "color".to_owned(),
                    value: "red".to_owned(),
                },
                StyleDeclaration {
                    property: "font-weight".to_owned(),
                    value: "700".to_owned(),
                },
            ],
        },
        StyleRule {
            selectors: vec![".sample".to_owned()],
            layer: Some(1),
            declarations: vec![StyleDeclaration {
                property: "all".to_owned(),
                value: "revert-layer".to_owned(),
            }],
        },
    ];
    let reverted = resolve_style(&inherited, &current, &[], &rules);
    assert_eq!(reverted.color, [255, 0, 0, 255]);
    assert_eq!(reverted.weight, Weight(700));

    let important = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "all".to_owned(),
            value: "unset !important".to_owned(),
        }],
    };
    let important_result = resolve_style(&inherited, &current, &[], &[important, all_before_color]);
    assert_eq!(important_result.color, inherited.color);
}
