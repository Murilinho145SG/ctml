use super::{ElementIdentity, resolve_style};
use crate::{StyleDeclaration, StyleRule, style::TextStyle};

fn style_for(property: &str, value: &str) -> TextStyle {
    let rule = StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    };
    let current = ElementIdentity {
        tag: "div".to_owned(),
        class: Some("case".to_owned()),
        id: None,
    };
    resolve_style(&TextStyle::default(), &current, &[], &[rule])
}

#[test]
fn box_sizing_accepts_content_box_and_border_box() {
    assert!(!style_for("box-sizing", "content-box").border_box);
    assert!(style_for("box-sizing", "border-box").border_box);
    assert!(style_for("box-sizing", "BORDER-BOX").border_box);
    assert!(!style_for("box-sizing", "unsupported").border_box);
}

#[test]
fn box_sizing_is_not_inherited_unless_requested() {
    let inherited = TextStyle {
        border_box: true,
        ..TextStyle::default()
    };
    let current = ElementIdentity {
        tag: "div".to_owned(),
        class: Some("child".to_owned()),
        id: None,
    };
    assert!(!resolve_style(&inherited, &current, &[], &[]).border_box);
    let rule = StyleRule {
        selectors: vec![".child".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "box-sizing".to_owned(),
            value: "inherit".to_owned(),
        }],
    };
    assert!(resolve_style(&inherited, &current, &[], &[rule]).border_box);
}

#[test]
fn padding_shorthand_expands_in_physical_side_order_and_longhands_override_it() {
    let rule = StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "padding".to_owned(),
                value: "1px 2px 3px 4px".to_owned(),
            },
            StyleDeclaration {
                property: "padding-left".to_owned(),
                value: "8px".to_owned(),
            },
        ],
    };
    let current = ElementIdentity {
        tag: "div".to_owned(),
        class: Some("case".to_owned()),
        id: None,
    };
    let style = resolve_style(&TextStyle::default(), &current, &[], &[rule]);

    assert_eq!(
        style.padding,
        [
            Some("1px".to_owned()),
            Some("2px".to_owned()),
            Some("3px".to_owned()),
            Some("8px".to_owned()),
        ]
    );
    assert_eq!(
        style_for("padding", "2px").padding,
        std::array::from_fn(|_| Some("2px".to_owned()))
    );
    assert_eq!(
        style_for("padding", "2px 4px").padding,
        [
            Some("2px".to_owned()),
            Some("4px".to_owned()),
            Some("2px".to_owned()),
            Some("4px".to_owned()),
        ]
    );
    assert_eq!(
        style_for("padding", "2px 4px 6px").padding,
        [
            Some("2px".to_owned()),
            Some("4px".to_owned()),
            Some("6px".to_owned()),
            Some("4px".to_owned()),
        ]
    );
    assert_eq!(
        style_for("padding", "10% 5%").padding,
        [
            Some("10%".to_owned()),
            Some("5%".to_owned()),
            Some("10%".to_owned()),
            Some("5%".to_owned()),
        ]
    );
    assert_eq!(
        style_for("padding", "-1px").padding,
        std::array::from_fn(|_| None)
    );
}

#[test]
fn logical_padding_maps_after_writing_mode_and_respects_cascade_order() {
    let current = ElementIdentity {
        tag: "div".to_owned(),
        class: Some("case".to_owned()),
        id: None,
    };
    let stylesheet = [StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "writing-mode".to_owned(),
                value: "vertical-rl".to_owned(),
            },
            StyleDeclaration {
                property: "direction".to_owned(),
                value: "rtl".to_owned(),
            },
            StyleDeclaration {
                property: "padding-inline-start".to_owned(),
                value: "3px".to_owned(),
            },
            StyleDeclaration {
                property: "padding-block".to_owned(),
                value: "4px 5px".to_owned(),
            },
        ],
    }];
    let style = resolve_style(&TextStyle::default(), &current, &[], &stylesheet);
    assert_eq!(
        style.padding,
        [
            None,
            Some("4px".to_owned()),
            Some("3px".to_owned()),
            Some("5px".to_owned()),
        ]
    );

    let stylesheet = [StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "padding-inline".to_owned(),
                value: "1px 2px".to_owned(),
            },
            StyleDeclaration {
                property: "padding-left".to_owned(),
                value: "8px".to_owned(),
            },
            StyleDeclaration {
                property: "padding-inline-start".to_owned(),
                value: "6px".to_owned(),
            },
        ],
    }];
    let style = resolve_style(&TextStyle::default(), &current, &[], &stylesheet);
    assert_eq!(style.padding[3].as_deref(), Some("6px"));
    assert_eq!(style.padding[1].as_deref(), Some("2px"));
}

#[test]
fn margins_accept_signed_lengths_and_logical_properties_follow_source_order() {
    let style = style_for("margin", "-2px 5% 1em");
    assert_eq!(
        style.margin,
        [
            Some("-2px".to_owned()),
            Some("5%".to_owned()),
            Some("1em".to_owned()),
            Some("5%".to_owned()),
        ]
    );
    assert_eq!(
        style_for("margin-inline", "auto").logical_margin[0].as_deref(),
        Some("auto")
    );
    let current = ElementIdentity {
        tag: "div".to_owned(),
        class: Some("case".to_owned()),
        id: None,
    };
    let stylesheet = [StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "margin-inline".to_owned(),
                value: "1px 2px".to_owned(),
            },
            StyleDeclaration {
                property: "margin-left".to_owned(),
                value: "8px".to_owned(),
            },
            StyleDeclaration {
                property: "margin-inline-start".to_owned(),
                value: "6px".to_owned(),
            },
        ],
    }];
    let style = resolve_style(&TextStyle::default(), &current, &[], &stylesheet);
    assert_eq!(style.margin[3].as_deref(), Some("6px"));
    assert_eq!(style.margin[1].as_deref(), Some("2px"));

    let vertical = [StyleRule {
        selectors: vec![".case".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "writing-mode".to_owned(),
                value: "vertical-rl".to_owned(),
            },
            StyleDeclaration {
                property: "direction".to_owned(),
                value: "rtl".to_owned(),
            },
            StyleDeclaration {
                property: "margin-inline-start".to_owned(),
                value: "3px".to_owned(),
            },
            StyleDeclaration {
                property: "margin-block".to_owned(),
                value: "4px 5px".to_owned(),
            },
        ],
    }];
    let style = resolve_style(&TextStyle::default(), &current, &[], &vertical);
    assert_eq!(
        style.margin,
        [
            None,
            Some("4px".to_owned()),
            Some("3px".to_owned()),
            Some("5px".to_owned()),
        ]
    );
}
