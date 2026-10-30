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
fn aspect_ratio_accepts_positive_ratio_forms_and_auto_reset() {
    assert_eq!(
        style_for(&[("aspect-ratio", "16 / 9")]).aspect_ratio,
        Some(16.0 / 9.0)
    );
    assert_eq!(
        style_for(&[("aspect-ratio", "1.5")]).aspect_ratio,
        Some(1.5)
    );
    assert_eq!(
        style_for(&[("aspect-ratio", "4/3"), ("aspect-ratio", "auto")]).aspect_ratio,
        None
    );
}

#[test]
fn aspect_ratio_rejects_zero_negative_nonfinite_and_malformed_values() {
    for value in ["0", "1/0", "-1/2", "1 / -2", "NaN", "1/2/3", "16px"] {
        assert_eq!(
            style_for(&[("aspect-ratio", value)]).aspect_ratio,
            None,
            "{value}"
        );
    }
}

#[test]
fn aspect_ratio_supports_css_wide_inherit_initial_and_all() {
    let inherited = TextStyle {
        aspect_ratio: Some(1.5),
        ..TextStyle::default()
    };
    let rule = StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "aspect-ratio".into(),
            value: "inherit".into(),
        }],
    };
    let current = ElementIdentity {
        tag: "div".into(),
        class: Some("case".into()),
        id: None,
    };
    assert_eq!(
        resolve_style(&inherited, &current, &[], &[rule]).aspect_ratio,
        Some(1.5)
    );
    assert_eq!(
        style_for(&[("aspect-ratio", "2/1"), ("aspect-ratio", "initial")]).aspect_ratio,
        None
    );
    assert_eq!(
        style_for(&[("aspect-ratio", "2/1"), ("all", "initial")]).aspect_ratio,
        None
    );
}
