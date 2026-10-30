use super::{ElementIdentity, resolve_style};
use crate::{StyleDeclaration, StyleRule, style::TextStyle};

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
fn border_radius_shorthand_expands_four_corners_and_elliptical_axes() {
    let style = style_for(&[("border-radius", "1px 2px 3px / 4px 5px")]);
    assert_eq!(
        style.border_radius[0],
        [Some("1px".into()), Some("4px".into())]
    );
    assert_eq!(
        style.border_radius[1],
        [Some("2px".into()), Some("5px".into())]
    );
    assert_eq!(
        style.border_radius[2],
        [Some("3px".into()), Some("4px".into())]
    );
    assert_eq!(
        style.border_radius[3],
        [Some("2px".into()), Some("5px".into())]
    );
}

#[test]
fn corner_longhands_override_shorthand_in_source_order_and_reject_negative_values() {
    let style = style_for(&[
        ("border-radius", "8px"),
        ("border-top-left-radius", "3px 5px"),
        ("border-bottom-right-radius", "-2px"),
    ]);
    assert_eq!(
        style.border_radius[0],
        [Some("3px".into()), Some("5px".into())]
    );
    assert_eq!(
        style.border_radius[2],
        [Some("8px".into()), Some("8px".into())]
    );
}

#[test]
fn border_radius_supports_initial_inherit_and_all() {
    let initial = style_for(&[
        ("border-radius", "8px"),
        ("border-top-left-radius", "initial"),
    ]);
    assert_eq!(initial.border_radius[0], [None, None]);
    assert_eq!(initial.border_radius[1][0].as_deref(), Some("8px"));

    let inherited = TextStyle {
        border_radius: std::array::from_fn(|_| [Some("6px".into()), Some("9px".into())]),
        ..TextStyle::default()
    };
    let rule = StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "border-top-right-radius".into(),
            value: "inherit".into(),
        }],
    };
    let current = ElementIdentity {
        tag: "div".into(),
        class: Some("case".into()),
        id: None,
    };
    let child = resolve_style(&inherited, &current, &[], &[rule]);
    assert_eq!(
        child.border_radius[1],
        [Some("6px".into()), Some("9px".into())]
    );

    let reset = style_for(&[("border-radius", "8px"), ("all", "initial")]);
    assert!(reset.border_radius.iter().flatten().all(Option::is_none));
}
