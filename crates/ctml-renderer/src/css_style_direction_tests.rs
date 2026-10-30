use super::{ElementIdentity, resolve_style, tests::style_for_declarations};
use crate::{
    StyleDeclaration, StyleRule,
    style::{Direction, TextStyle},
};

#[test]
fn direction_accepts_ltr_and_rtl_and_inherits() {
    assert_eq!(
        style_for_declarations(&[("direction", "ltr")]).direction,
        Direction::Ltr
    );
    assert_eq!(
        style_for_declarations(&[("direction", "rtl")]).direction,
        Direction::Rtl
    );

    let inherited = TextStyle {
        direction: Direction::Rtl,
        ..TextStyle::default()
    };
    let current = ElementIdentity {
        tag: "span".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };
    let rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "direction".to_owned(),
            value: "unset".to_owned(),
        }],
    };
    assert_eq!(
        resolve_style(&inherited, &current, &[], &[rule]).direction,
        Direction::Rtl
    );
}
