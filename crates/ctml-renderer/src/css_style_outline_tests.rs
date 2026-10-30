use super::{ElementIdentity, resolve_style};
use crate::{
    StyleDeclaration, StyleRule,
    style::{BorderLineStyle, TextStyle},
};

fn style_for(declarations: &[(&str, &str)]) -> TextStyle {
    let rule = StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: declarations
            .iter()
            .map(|(property, value)| StyleDeclaration {
                property: (*property).to_owned(),
                value: (*value).to_owned(),
            })
            .collect(),
    };
    let current = ElementIdentity {
        tag: "div".to_owned(),
        class: Some("case".to_owned()),
        id: None,
    };
    resolve_style(&TextStyle::default(), &current, &[], &[rule])
}

#[test]
fn outline_shorthand_accepts_components_in_any_order_and_longhands_override() {
    let style = style_for(&[
        ("outline", "blue dashed 4px"),
        ("outline-width", "2px"),
        ("outline-offset", "-3px"),
    ]);
    assert_eq!(style.outline_width.as_deref(), Some("2px"));
    assert_eq!(style.outline_style, BorderLineStyle::Dashed);
    assert_eq!(style.outline_color, Some([0, 0, 255, 255]));
    assert_eq!(style.outline_offset.as_deref(), Some("-3px"));
}

#[test]
fn outline_css_wide_values_and_all_reset_components() {
    let initial = style_for(&[("outline", "4px solid red"), ("outline-style", "initial")]);
    assert_eq!(initial.outline_style, BorderLineStyle::None);
    assert_eq!(initial.outline_width.as_deref(), Some("4px"));
    assert_eq!(initial.outline_color, Some([255, 0, 0, 255]));

    let inherited = TextStyle {
        outline_width: Some("7px".into()),
        outline_style: BorderLineStyle::Dotted,
        outline_color: Some([1, 2, 3, 255]),
        outline_offset: Some("2px".into()),
        ..TextStyle::default()
    };
    let rule = StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "outline".into(),
            value: "inherit".into(),
        }],
    };
    let current = ElementIdentity {
        tag: "div".into(),
        class: Some("case".into()),
        id: None,
    };
    let child = resolve_style(&inherited, &current, &[], &[rule]);
    assert_eq!(child.outline_width, inherited.outline_width);
    assert_eq!(child.outline_style, inherited.outline_style);
    assert_eq!(child.outline_color, inherited.outline_color);

    let reset = style_for(&[("outline", "4px solid red"), ("all", "initial")]);
    assert_eq!(reset.outline_style, BorderLineStyle::None);
    assert!(reset.outline_offset.is_none());
}

#[test]
fn outline_ignores_invalid_width_style_and_offset_values() {
    let style = style_for(&[
        ("outline-width", "-2px"),
        ("outline-style", "hidden"),
        ("outline-offset", "20%"),
    ]);
    assert!(style.outline_width.is_none());
    assert_eq!(style.outline_style, BorderLineStyle::None);
    assert!(style.outline_offset.is_none());
}
