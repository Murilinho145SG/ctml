use cosmic_text::{Align, Stretch, Style, Weight};
use winit::window::CursorIcon;

use super::{
    ElementIdentity, apply_matching_styles, apply_matching_styles_with_attributes, resolve_style,
    resolve_style_with_inline_context, resolve_style_with_tree_inline_context,
    tests::style_for_declarations,
};
use crate::{
    StyleDeclaration, StyleRule,
    style::{
        FontVariantCaps, FontVariantPosition, TextDecorationSkipInk, TextDecorationStyle,
        TextStyle, TextTransform,
    },
};
use ctml_css::{SelectorPosition, SelectorSibling};

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
        tag: "p".to_owned(),
        class: Some("case".to_owned()),
        id: None,
    };
    resolve_style(&TextStyle::default(), &current, &[], &[rule])
}

#[test]
fn ignores_unknown_properties_without_affecting_computed_text_style() {
    assert_eq!(
        style_for("future-property", "anything"),
        TextStyle::default()
    );
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
fn inline_style_uses_author_cascade_priority_and_importance() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("target".to_owned()),
        id: Some("target".to_owned()),
    };
    let stylesheet = [
        StyleRule {
            selectors: vec!["#target".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-family".to_owned(),
                value: "monospace".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".target".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "color".to_owned(),
                    value: "blue !important".to_owned(),
                },
                StyleDeclaration {
                    property: "font-weight".to_owned(),
                    value: "400 !important".to_owned(),
                },
                StyleDeclaration {
                    property: "text-transform".to_owned(),
                    value: "lowercase !important".to_owned(),
                },
            ],
        },
    ];

    let style = resolve_style_with_inline_context(
        &TextStyle::default(),
        &current,
        &[],
        &[],
        &[],
        Some(
            "color: red; font-family: serif; font-weight: 700; text-transform: uppercase !important; cursor: help;",
        ),
        &stylesheet,
    );

    assert_eq!(style.color, [0, 0, 255, 255]);
    assert_eq!(style.family, "serif");
    assert_eq!(style.weight, Weight(400));
    assert_eq!(style.text_transform, TextTransform::Uppercase);
    assert_eq!(style.cursor, CursorIcon::Help);
}

#[test]
fn sibling_selectors_apply_styles_to_elements_in_document_order() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("target".to_owned()),
        id: None,
    };
    let previous = [
        SelectorSibling {
            element: ElementIdentity {
                tag: "h1".to_owned(),
                class: Some("title".to_owned()),
                id: None,
            },
            attributes: Vec::new(),
            position: SelectorPosition::default(),
            node_id: 0,
        },
        SelectorSibling {
            element: ElementIdentity {
                tag: "div".to_owned(),
                class: None,
                id: None,
            },
            attributes: Vec::new(),
            position: SelectorPosition::default(),
            node_id: 0,
        },
    ];
    let stylesheet = [
        StyleRule {
            selectors: vec!["div + p.target".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "color".to_owned(),
                value: "red".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["h1 ~ p.target".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-weight".to_owned(),
                value: "700".to_owned(),
            }],
        },
    ];

    let style = resolve_style_with_tree_inline_context(
        &TextStyle::default(),
        &current,
        &[],
        &[],
        &[],
        &previous,
        &[],
        SelectorPosition::default(),
        &[],
        None,
        &stylesheet,
    );

    assert_eq!(style.color, [255, 0, 0, 255]);
    assert_eq!(style.weight, Weight(700));
}

#[test]
fn inherited_language_and_direction_selectors_apply_styles() {
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };
    let stylesheet = [StyleRule {
        selectors: vec![":lang(en):dir(rtl)".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "color".to_owned(),
            value: "blue".to_owned(),
        }],
    }];
    let style = resolve_style_with_tree_inline_context(
        &TextStyle::default(),
        &paragraph,
        &[body],
        &[],
        &[vec![
            ("lang".to_owned(), Some("en-GB".to_owned())),
            ("dir".to_owned(), Some("rtl".to_owned())),
        ]],
        &[],
        &[vec![]],
        SelectorPosition::default(),
        &[SelectorPosition::default()],
        None,
        &stylesheet,
    );

    assert_eq!(style.color, [0, 0, 255, 255]);
}

#[test]
fn revert_uses_the_supported_initial_user_agent_values() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("reset".to_owned()),
        id: None,
    };
    let inherited = TextStyle {
        weight: Weight::BOLD,
        ..TextStyle::default()
    };
    let stylesheet = [StyleRule {
        selectors: vec![".reset".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "color".to_owned(),
                value: "red".to_owned(),
            },
            StyleDeclaration {
                property: "font-weight".to_owned(),
                value: "revert".to_owned(),
            },
            StyleDeclaration {
                property: "color".to_owned(),
                value: "revert".to_owned(),
            },
        ],
    }];
    let style = resolve_style(&inherited, &current, &[], &stylesheet);

    assert_eq!(style.color, [0, 0, 0, 255]);
    assert_eq!(style.weight, Weight::NORMAL);
}

#[test]
fn structural_pseudo_classes_apply_styles_from_element_positions() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };
    let position = SelectorPosition {
        child_index: 2,
        child_count: 3,
        type_index: 1,
        type_count: 2,
        is_empty: true,
        is_root: false,
        ..SelectorPosition::default()
    };
    let stylesheet = [
        StyleRule {
            selectors: vec!["p:nth-child(even)".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "color".to_owned(),
                value: "red".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p:first-of-type:empty".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-weight".to_owned(),
                value: "700".to_owned(),
            }],
        },
    ];

    let style = resolve_style_with_tree_inline_context(
        &TextStyle::default(),
        &current,
        &[],
        &[],
        &[],
        &[],
        &[],
        position,
        &[],
        None,
        &stylesheet,
    );

    assert_eq!(style.color, [255, 0, 0, 255]);
    assert_eq!(style.weight, Weight(700));
}

#[test]
fn hover_and_active_pseudo_classes_follow_pointer_state() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("target".to_owned()),
        id: None,
    };
    let hovered = SelectorPosition {
        is_hovered: true,
        ..SelectorPosition::default()
    };
    let active = SelectorPosition {
        is_hovered: true,
        is_active: true,
        ..SelectorPosition::default()
    };
    let stylesheet = [
        StyleRule {
            selectors: vec!["p:hover".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "color".to_owned(),
                value: "red".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p:active".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-weight".to_owned(),
                value: "700".to_owned(),
            }],
        },
    ];

    let hover_style = resolve_style_with_tree_inline_context(
        &TextStyle::default(),
        &current,
        &[],
        &[],
        &[],
        &[],
        &[],
        hovered,
        &[],
        None,
        &stylesheet,
    );
    let active_style = resolve_style_with_tree_inline_context(
        &TextStyle::default(),
        &current,
        &[],
        &[],
        &[],
        &[],
        &[],
        active,
        &[],
        None,
        &stylesheet,
    );

    assert_eq!(hover_style.color, [255, 0, 0, 255]);
    assert_eq!(hover_style.weight, Weight(400));
    assert_eq!(active_style.color, [255, 0, 0, 255]);
    assert_eq!(active_style.weight, Weight(700));
}

#[test]
fn covers_every_font_style_keyword() {
    for (value, expected) in [
        ("normal", Style::Normal),
        ("italic", Style::Italic),
        ("oblique", Style::Oblique),
    ] {
        assert_eq!(style_for("font-style", value).font_style, expected);
    }
}

#[test]
fn covers_font_family_first_choice_and_fallback_list() {
    assert_eq!(style_for("font-family", "serif").family, "serif");
    assert_eq!(
        style_for("font-family", "\"DejaVu Sans\", sans-serif").family,
        "\"DejaVu Sans\", sans-serif"
    );
}

#[test]
fn covers_font_weight_keywords_and_numeric_boundaries() {
    for (value, expected) in [
        ("normal", Weight::NORMAL),
        ("bold", Weight::BOLD),
        ("1", Weight(1)),
        ("100", Weight(100)),
        ("400", Weight(400)),
        ("900", Weight(900)),
        ("1000", Weight(1000)),
        ("bolder", Weight::BOLD),
        ("lighter", Weight(100)),
    ] {
        assert_eq!(style_for("font-weight", value).weight, expected, "{value}");
    }
    for invalid in ["0", "1001", "boldish", "NaN"] {
        assert_eq!(style_for("font-weight", invalid).weight, Weight::NORMAL);
    }
}

#[test]
fn covers_every_font_kerning_keyword() {
    for (value, expected) in [
        ("auto", Some(true)),
        ("normal", Some(true)),
        ("none", Some(false)),
    ] {
        assert_eq!(style_for("font-kerning", value).kerning, expected);
    }
}

#[test]
fn covers_every_font_stretch_keyword_and_mapping_bucket() {
    for (value, expected) in [
        ("ultra-condensed", Stretch::UltraCondensed),
        ("extra-condensed", Stretch::ExtraCondensed),
        ("condensed", Stretch::Condensed),
        ("semi-condensed", Stretch::SemiCondensed),
        ("normal", Stretch::Normal),
        ("semi-expanded", Stretch::SemiExpanded),
        ("expanded", Stretch::Expanded),
        ("extra-expanded", Stretch::ExtraExpanded),
        ("ultra-expanded", Stretch::UltraExpanded),
        ("50%", Stretch::UltraCondensed),
        ("62.5%", Stretch::ExtraCondensed),
        ("75%", Stretch::Condensed),
        ("87.5%", Stretch::SemiCondensed),
        ("100%", Stretch::Normal),
        ("112.5%", Stretch::SemiExpanded),
        ("125%", Stretch::Expanded),
        ("150%", Stretch::ExtraExpanded),
        ("200%", Stretch::UltraExpanded),
    ] {
        assert_eq!(
            style_for("font-stretch", value).stretch,
            expected,
            "{value}"
        );
    }
}

#[test]
fn covers_line_height_number_percentage_and_length_values() {
    for (value, expected_height, expected_multiplier) in [
        ("1", 16.0, Some(1.0)),
        ("1.5", 24.0, Some(1.5)),
        ("150%", 24.0, None),
        ("24px", 24.0, None),
        ("18pt", 24.0, None),
        ("1.5em", 24.0, None),
        ("1.5rem", 24.0, None),
    ] {
        let style = style_for("line-height", value);
        assert_eq!(style.line_height, expected_height, "{value}");
        assert_eq!(style.line_height_multiplier, expected_multiplier, "{value}");
    }
    let normal = style_for("line-height", "normal");
    assert!(normal.line_height_normal);
    assert_eq!(normal.line_height_multiplier, None);
}

#[test]
fn covers_letter_spacing_units_and_reset() {
    for (value, expected, is_pixels) in [
        ("normal", 0.0, false),
        ("2px", 2.0, true),
        ("-1px", -1.0, true),
        ("0.1em", 0.1, false),
        ("-0.05em", -0.05, false),
    ] {
        let style = style_for("letter-spacing", value);
        assert_eq!(style.letter_spacing, expected, "{value}");
        assert_eq!(style.letter_spacing_px, is_pixels, "{value}");
    }
}

#[path = "css_style_all_tests.rs"]
mod all;
#[path = "css_style_cascade_matrix_tests.rs"]
mod cascade;
#[path = "css_style_decoration_matrix_tests.rs"]
mod decoration;
#[path = "css_style_variables_tests.rs"]
mod variables;
