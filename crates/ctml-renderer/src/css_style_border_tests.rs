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
fn border_width_shorthand_expands_and_side_longhands_override_by_source_order() {
    let style = style_for(&[
        ("border-width", "1px 2px 3px 4px"),
        ("border-left-width", "8px"),
    ]);
    assert_eq!(
        style.border_width,
        [
            Some("1px".into()),
            Some("2px".into()),
            Some("3px".into()),
            Some("8px".into())
        ]
    );
}

#[test]
fn logical_border_widths_follow_writing_mode_direction_and_source_order() {
    let style = style_for(&[
        ("border-inline-width", "4px"),
        ("border-left-width", "2px"),
        ("border-inline-start-width", "6px"),
    ]);
    assert_eq!(style.border_width[1].as_deref(), Some("4px"));
    assert_eq!(style.border_width[3].as_deref(), Some("6px"));

    let vertical = style_for(&[
        ("writing-mode", "vertical-rl"),
        ("direction", "rtl"),
        ("border-inline-start-width", "7px"),
        ("border-block-end-width", "9px"),
    ]);
    assert_eq!(vertical.border_width[2].as_deref(), Some("7px"));
    assert_eq!(vertical.border_width[3].as_deref(), Some("9px"));
}

#[test]
fn border_style_shorthand_parses_supported_styles_and_resets_with_none() {
    let style = style_for(&[
        ("border-style", "solid dashed"),
        ("border-left-style", "none"),
    ]);
    assert_eq!(style.border_style[0], BorderLineStyle::Solid);
    assert_eq!(style.border_style[1], BorderLineStyle::Dashed);
    assert_eq!(style.border_style[3], BorderLineStyle::None);
}

#[test]
fn border_widths_support_css_wide_initial_inherit_and_all_resets() {
    let initial = style_for(&[("border-width", "4px"), ("border-left-width", "initial")]);
    assert_eq!(initial.border_width[0].as_deref(), Some("4px"));
    assert_eq!(initial.border_width[3], None);

    let inherited = TextStyle {
        border_width: std::array::from_fn(|_| Some("7px".into())),
        border_style: [BorderLineStyle::Solid; 4],
        ..TextStyle::default()
    };
    let rule = StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "border-top-width".to_owned(),
            value: "inherit".to_owned(),
        }],
    };
    let current = ElementIdentity {
        tag: "div".to_owned(),
        class: Some("case".to_owned()),
        id: None,
    };
    let inherited_child = resolve_style(&inherited, &current, &[], &[rule]);
    assert_eq!(inherited_child.border_width[0].as_deref(), Some("7px"));
    assert_eq!(inherited_child.border_width[1], None);

    let all_reset = style_for(&[("border-width", "4px"), ("all", "initial")]);
    assert!(
        all_reset
            .border_width
            .iter()
            .all(|width| width.as_deref().is_none_or(|width| width == "medium"))
    );
    assert!(
        all_reset
            .border_style
            .iter()
            .all(|style| *style == BorderLineStyle::None)
    );
}
