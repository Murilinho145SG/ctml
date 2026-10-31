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
