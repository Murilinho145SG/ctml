use super::{ElementIdentity, resolve_style};
use crate::{
    StyleDeclaration, StyleRule,
    style::{BoxShadow, TextStyle},
};

fn style_for(value: &str) -> TextStyle {
    style_with(&[("box-shadow", value)])
}

fn style_with(declarations: &[(&str, &str)]) -> TextStyle {
    let rule = StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: declarations
            .iter()
            .map(|(property, value)| StyleDeclaration {
                property: (*property).into(),
                value: (*value).into(),
            })
            .collect(),
    };
    let current = ElementIdentity {
        tag: "div".into(),
        class: Some("case".into()),
        id: None,
    };
    resolve_style(&TextStyle::default(), &current, &[], &[rule])
}

#[test]
fn box_shadow_parses_multiple_outer_and_inset_shadows() {
    let style = style_for("2px 3px 4px -1px red, inset 0 1px 2px currentColor");
    assert_eq!(style.box_shadows.len(), 2);
    assert_eq!(
        style.box_shadows[0],
        BoxShadow {
            offset_x: 2.0,
            offset_y: 3.0,
            blur: 4.0,
            spread: -1.0,
            color: Some([255, 0, 0, 255]),
            inset: false,
        }
    );
    assert_eq!(
        style.box_shadows[1],
        BoxShadow {
            offset_x: 0.0,
            offset_y: 1.0,
            blur: 2.0,
            spread: 0.0,
            color: None,
            inset: true,
        }
    );
}

#[test]
fn box_shadow_accepts_length_math_expressions() {
    let shadow = style_for("calc(2px + 3px) 4px 0 red").box_shadows[0];
    assert_eq!(shadow.offset_x, 5.0);
    assert_eq!(shadow.offset_y, 4.0);
}

#[test]
fn box_shadow_none_and_css_wide_keywords_reset_or_inherit() {
    assert!(
        style_with(&[("box-shadow", "2px 2px red"), ("box-shadow", "none")])
            .box_shadows
            .is_empty()
    );
    assert!(style_for("2px 2px red").box_shadows.len() == 1);
    let inherited = TextStyle {
        box_shadows: style_for("1px 2px 3px blue").box_shadows,
        ..TextStyle::default()
    };
    let rule = StyleRule {
        selectors: vec![".case".into()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "box-shadow".into(),
            value: "inherit".into(),
        }],
    };
    let current = ElementIdentity {
        tag: "div".into(),
        class: Some("case".into()),
        id: None,
    };
    assert_eq!(
        resolve_style(&inherited, &current, &[], &[rule]).box_shadows,
        inherited.box_shadows
    );
    assert!(
        style_with(&[("box-shadow", "1px 2px red"), ("box-shadow", "initial")])
            .box_shadows
            .is_empty()
    );
}

#[test]
fn box_shadow_rejects_missing_offsets_duplicate_components_and_negative_blur() {
    for value in [
        "red",
        "1px red",
        "1px 2px -3px red",
        "1px 2px 3px 4px 5px red",
        "inset inset 1px 2px",
    ] {
        assert!(style_for(value).box_shadows.is_empty(), "{value}");
    }
}

#[test]
fn all_initial_resets_box_shadow() {
    assert!(
        style_with(&[("box-shadow", "1px 2px red"), ("all", "initial")])
            .box_shadows
            .is_empty()
    );
}
