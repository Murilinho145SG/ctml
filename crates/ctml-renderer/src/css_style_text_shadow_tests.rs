use super::{ElementIdentity, resolve_style, tests::style_for_declarations};
use crate::{
    StyleDeclaration, StyleRule,
    style::{TextShadow, TextStyle},
};

#[test]
fn resolves_text_shadow_values_and_ignores_malformed_declarations() {
    let style = style_for_declarations(&[("color", "red"), ("text-shadow", "1px 2px 3px blue")]);
    assert_eq!(
        style.text_shadows,
        [TextShadow {
            offset_x: 1.0,
            offset_y: 2.0,
            blur: 3.0,
            color: [0, 0, 255, 255],
        }]
    );
    let current_color = style_for_declarations(&[("color", "red"), ("text-shadow", "0 1em")]);
    assert_eq!(current_color.text_shadows[0].color, [255, 0, 0, 255]);
    assert_eq!(current_color.text_shadows[0].offset_y, 16.0);

    let inherited = TextStyle {
        text_shadows: vec![TextShadow {
            offset_x: 0.0,
            offset_y: 1.0,
            blur: 2.0,
            color: [1, 2, 3, 255],
        }],
        ..TextStyle::default()
    };
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };
    let invalid = StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "text-shadow".to_owned(),
            value: "1px 2px -3px red".to_owned(),
        }],
    };
    assert_eq!(
        resolve_style(&inherited, &current, &[], &[invalid]).text_shadows,
        inherited.text_shadows
    );
}

#[test]
fn text_shadow_inherits_and_supports_css_wide_resets() {
    let inherited = TextStyle {
        text_shadows: vec![TextShadow {
            offset_x: 1.0,
            offset_y: 2.0,
            blur: 0.0,
            color: [10, 20, 30, 255],
        }],
        ..TextStyle::default()
    };
    let current = ElementIdentity {
        tag: "span".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };
    let rule = |value: &str| StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "text-shadow".to_owned(),
            value: value.to_owned(),
        }],
    };

    assert_eq!(
        resolve_style(&inherited, &current, &[], &[]).text_shadows,
        inherited.text_shadows
    );
    assert!(
        resolve_style(&inherited, &current, &[], &[rule("none")])
            .text_shadows
            .is_empty()
    );
    assert_eq!(
        resolve_style(&inherited, &current, &[], &[rule("initial")]).text_shadows,
        Vec::<TextShadow>::new()
    );
    assert_eq!(
        resolve_style(&inherited, &current, &[], &[rule("inherit")]).text_shadows,
        inherited.text_shadows
    );
}
