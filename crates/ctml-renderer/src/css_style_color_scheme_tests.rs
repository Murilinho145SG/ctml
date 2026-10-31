use super::{ElementIdentity, resolve_style};
use crate::{StyleDeclaration, StyleRule, style::TextStyle};

fn style_for(declarations: &[(&str, &str)]) -> TextStyle {
    let rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: declarations
            .iter()
            .map(|(property, value)| StyleDeclaration {
                property: (*property).to_owned(),
                value: (*value).to_owned(),
            })
            .collect(),
    };
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let element = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };
    resolve_style(&TextStyle::default(), &element, &[body], &[rule])
}

#[test]
fn system_colors_use_the_selected_scheme_independent_of_declaration_order() {
    for declarations in [
        [
            ("color", "CanvasText"),
            ("background-color", "Canvas"),
            ("color-scheme", "dark"),
        ],
        [
            ("color-scheme", "dark"),
            ("color", "CanvasText"),
            ("background-color", "Canvas"),
        ],
    ] {
        let style = style_for(&declarations);
        assert!(style.dark_color_scheme);
        assert_eq!(style.color, [245, 245, 245, 255]);
        assert_eq!(style.background_color, Some([24, 24, 24, 255]));
    }
}

#[test]
fn color_scheme_uses_the_first_supported_entry_and_defaults_to_light() {
    assert!(style_for(&[("color-scheme", "dark light")]).dark_color_scheme);
    assert!(!style_for(&[("color-scheme", "light dark")]).dark_color_scheme);
    assert!(style_for(&[("color-scheme", "future dark")]).dark_color_scheme);
    assert!(!style_for(&[("color-scheme", "normal")]).dark_color_scheme);
    assert!(!style_for(&[("color-scheme", "future")]).dark_color_scheme);
    assert!(style_for(&[("color-scheme", "dark only")]).dark_color_scheme);
}

#[test]
fn color_scheme_is_inherited_and_can_be_reset_to_initial() {
    let dark_parent = style_for(&[("color-scheme", "dark")]);
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let child = ElementIdentity {
        tag: "span".to_owned(),
        class: None,
        id: None,
    };
    let rule = StyleRule {
        selectors: vec!["span".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "color".to_owned(),
            value: "CanvasText".to_owned(),
        }],
    };
    let inherited = resolve_style(&dark_parent, &child, &[body.clone()], &[rule.clone()]);
    assert!(inherited.dark_color_scheme);
    assert_eq!(inherited.color, [245, 245, 245, 255]);

    let reset_rule = StyleRule {
        selectors: vec!["span".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "color-scheme".to_owned(),
            value: "initial".to_owned(),
        }],
    };
    let reset = resolve_style(&dark_parent, &child, &[body], &[rule, reset_rule]);
    assert!(!reset.dark_color_scheme);
    assert_eq!(reset.color, [0, 0, 0, 255]);
}

#[test]
fn color_functions_resolve_system_colors_in_the_selected_scheme() {
    let style = style_for(&[
        ("color-scheme", "dark"),
        ("color", "color-mix(in srgb, CanvasText 50%, Canvas)"),
    ]);
    assert_eq!(style.color, [135, 135, 135, 255]);
}

#[test]
fn dark_system_colors_reach_borders_shadows_and_text_decorations() {
    let style = style_for(&[
        ("color-scheme", "dark"),
        ("border-color", "CanvasText"),
        ("box-shadow", "1px 1px CanvasText"),
        ("text-shadow", "1px 1px CanvasText"),
        ("text-decoration-color", "CanvasText"),
    ]);
    let light_ink = [245, 245, 245, 255];

    assert_eq!(style.border_color, [Some(light_ink); 4]);
    assert_eq!(style.box_shadows[0].color, Some(light_ink));
    assert_eq!(style.text_shadows[0].color, light_ink);
    assert_eq!(style.text_decoration_color, Some(light_ink));
}
