use super::{ElementIdentity, resolve_style};
use crate::{
    StyleDeclaration, StyleRule,
    style::{BoxDecorationBreak, TextStyle},
};

fn style_for(value: &str) -> TextStyle {
    let rule = StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "box-decoration-break".into(),
            value: value.into(),
        }],
    };
    let current = ElementIdentity {
        tag: "span".to_owned(),
        class: Some("case".to_owned()),
        id: None,
    };
    resolve_style(&TextStyle::default(), &current, &[], &[rule])
}

#[test]
fn box_decoration_break_accepts_slice_and_clone() {
    assert_eq!(
        style_for("clone").box_decoration_break,
        BoxDecorationBreak::Clone
    );
    assert_eq!(
        style_for("slice").box_decoration_break,
        BoxDecorationBreak::Slice
    );
    assert_eq!(
        TextStyle::default().box_decoration_break,
        BoxDecorationBreak::Slice
    );
}

#[test]
fn box_decoration_break_supports_initial_inherit_and_all() {
    let initial = StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "box-decoration-break".into(),
                value: "clone".into(),
            },
            StyleDeclaration {
                property: "box-decoration-break".into(),
                value: "initial".into(),
            },
        ],
    };
    let current = ElementIdentity {
        tag: "span".into(),
        class: Some("case".into()),
        id: None,
    };
    assert_eq!(
        resolve_style(&TextStyle::default(), &current, &[], &[initial]).box_decoration_break,
        BoxDecorationBreak::Slice
    );

    let parent = TextStyle {
        box_decoration_break: BoxDecorationBreak::Clone,
        ..TextStyle::default()
    };
    let inherited = StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "box-decoration-break".into(),
            value: "inherit".into(),
        }],
    };
    assert_eq!(
        resolve_style(&parent, &current, &[], &[inherited]).box_decoration_break,
        BoxDecorationBreak::Clone
    );

    let all = StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "box-decoration-break".into(),
                value: "clone".into(),
            },
            StyleDeclaration {
                property: "all".into(),
                value: "initial".into(),
            },
        ],
    };
    assert_eq!(
        resolve_style(&TextStyle::default(), &current, &[], &[all]).box_decoration_break,
        BoxDecorationBreak::Slice
    );
}
