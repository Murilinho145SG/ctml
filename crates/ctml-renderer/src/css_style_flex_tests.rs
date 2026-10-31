use crate::{StyleDeclaration, StyleRule, style::TextStyle};

use super::{ElementIdentity, resolve_style};

fn style_with_order(value: &str) -> i32 {
    resolve_style(
        &TextStyle::default(),
        &ElementIdentity {
            tag: "p".to_owned(),
            class: None,
            id: None,
        },
        &[],
        &[rule("order", value)],
    )
    .flex_order
}

fn rule(property: &str, value: &str) -> StyleRule {
    StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    }
}

fn style_with_declarations(declarations: &[(&str, &str)]) -> TextStyle {
    let rule = StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: declarations
            .iter()
            .map(|(property, value)| StyleDeclaration {
                property: (*property).to_owned(),
                value: (*value).to_owned(),
            })
            .collect(),
    };
    resolve_style(
        &TextStyle::default(),
        &ElementIdentity {
            tag: "p".to_owned(),
            class: None,
            id: None,
        },
        &[],
        &[rule],
    )
}

#[test]
fn order_parses_integers_and_clamps_values_outside_the_style_range() {
    assert_eq!(style_with_order("-3"), -3);
    assert_eq!(style_with_order("2"), 2);
    assert_eq!(style_with_order("1.5"), 0);
    assert_eq!(style_with_order("999999999999999999999"), i32::MAX);
    assert_eq!(style_with_order("-999999999999999999999"), i32::MIN);
}

#[test]
fn order_is_non_inherited_and_supports_css_wide_resets() {
    let inherited = TextStyle {
        flex_order: 7,
        ..TextStyle::default()
    };
    let element = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };

    assert_eq!(resolve_style(&inherited, &element, &[], &[]).flex_order, 0);
    assert_eq!(
        resolve_style(&inherited, &element, &[], &[rule("order", "inherit")]).flex_order,
        7
    );
    for keyword in ["initial", "unset"] {
        assert_eq!(
            resolve_style(&inherited, &element, &[], &[rule("order", keyword)]).flex_order,
            0
        );
    }
    assert_eq!(
        resolve_style(
            &inherited,
            &element,
            &[],
            &[rule("order", "3"), rule("all", "initial")]
        )
        .flex_order,
        0
    );
}

#[test]
fn flex_gaps_parse_shorthand_longhands_and_source_order() {
    let single = style_with_declarations(&[("gap", "4px")]);
    assert_eq!(single.row_gap.as_deref(), Some("4px"));
    assert_eq!(single.column_gap.as_deref(), Some("4px"));

    let pair = style_with_declarations(&[("gap", "2px 8px")]);
    assert_eq!(pair.row_gap.as_deref(), Some("2px"));
    assert_eq!(pair.column_gap.as_deref(), Some("8px"));

    let math = style_with_declarations(&[("gap", "calc(1px + 2px)")]);
    assert_eq!(math.row_gap.as_deref(), Some("calc(1px + 2px)"));

    let ordered = style_with_declarations(&[
        ("gap", "3px 6px"),
        ("row-gap", "5px"),
        ("column-gap", "9px"),
    ]);
    assert_eq!(ordered.row_gap.as_deref(), Some("5px"));
    assert_eq!(ordered.column_gap.as_deref(), Some("9px"));

    let later_shorthand = style_with_declarations(&[("column-gap", "9px"), ("gap", "3px 6px")]);
    assert_eq!(later_shorthand.column_gap.as_deref(), Some("6px"));

    let invalid = style_with_declarations(&[("gap", "-2px")]);
    assert_eq!(invalid.row_gap, None);
    assert_eq!(
        style_with_declarations(&[("gap", "normal")])
            .column_gap
            .as_deref(),
        Some("0")
    );

    let reset = style_with_declarations(&[("gap", "3px"), ("all", "initial")]);
    assert_eq!(reset.row_gap.as_deref(), Some("0"));
    assert_eq!(reset.column_gap.as_deref(), Some("0"));
}
