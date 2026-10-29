use cosmic_text::{Align, Stretch, Style, Weight};
use winit::window::CursorIcon;

use super::{
    ElementIdentity, apply_matching_styles, apply_matching_styles_with_attributes, resolve_style,
    resolve_style_with_inline_context, resolve_style_with_tree_inline_context,
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

#[test]
fn covers_every_text_transform_value() {
    for (value, expected) in [
        ("none", TextTransform::None),
        ("uppercase", TextTransform::Uppercase),
        ("lowercase", TextTransform::Lowercase),
        ("capitalize", TextTransform::Capitalize),
    ] {
        assert_eq!(style_for("text-transform", value).text_transform, expected);
    }
}

#[test]
fn covers_every_text_alignment_value() {
    for (value, expected) in [
        ("left", Align::Left),
        ("start", Align::Left),
        ("right", Align::Right),
        ("end", Align::End),
        ("center", Align::Center),
        ("justify", Align::Justified),
    ] {
        assert_eq!(style_for("text-align", value).text_align, expected);
    }
}

#[test]
fn covers_every_text_decoration_line_and_style() {
    let none = style_for("text-decoration", "none");
    assert!(!none.underline && !none.overline && !none.strikethrough);
    let all = style_for("text-decoration", "underline overline line-through");
    assert!(all.underline && all.overline && all.strikethrough);
    for (value, expected) in [
        ("solid", TextDecorationStyle::Solid),
        ("double", TextDecorationStyle::Double),
        ("dotted", TextDecorationStyle::Dotted),
        ("dashed", TextDecorationStyle::Dashed),
        ("wavy", TextDecorationStyle::Wavy),
    ] {
        assert_eq!(
            style_for("text-decoration-style", value).decoration_style,
            expected,
            "{value}"
        );
    }
}

#[test]
fn covers_every_text_decoration_skip_ink_value() {
    for (value, expected) in [
        ("auto", TextDecorationSkipInk::Auto),
        ("none", TextDecorationSkipInk::None),
        ("all", TextDecorationSkipInk::All),
    ] {
        assert_eq!(
            style_for("text-decoration-skip-ink", value).decoration_skip_ink,
            expected,
            "{value}"
        );
    }
}

#[test]
fn covers_nonnegative_text_decoration_thickness_lengths() {
    for (value, expected) in [
        ("0", 0.0),
        ("0px", 0.0),
        ("2px", 2.0),
        ("0.5em", 8.0),
        ("25%", 4.0),
    ] {
        assert_eq!(
            style_for("text-decoration-thickness", value).decoration_thickness,
            Some(expected),
            "{value}"
        );
    }
    for value in ["auto", "from-font"] {
        assert_eq!(
            style_for("text-decoration-thickness", value).decoration_thickness,
            None,
            "{value}"
        );
    }
    assert_eq!(
        style_for("text-decoration-thickness", "-1px").decoration_thickness,
        None
    );
}

#[test]
fn covers_signed_text_underline_offset_lengths() {
    for (value, expected) in [
        ("0", 0.0),
        ("2px", 2.0),
        ("-2px", -2.0),
        ("1em", 16.0),
        ("50%", 8.0),
        ("-25%", -4.0),
    ] {
        assert_eq!(
            style_for("text-underline-offset", value).underline_offset,
            Some(expected),
            "{value}"
        );
    }
    assert_eq!(
        style_for("text-underline-offset", "auto").underline_offset,
        None
    );
    assert_eq!(
        style_for("text-underline-offset", "invalid").underline_offset,
        None
    );
}

#[test]
fn covers_every_font_feature_switch() {
    let none = style_for("font-variant-ligatures", "none");
    assert_eq!(none.common_ligatures, Some(false));
    assert_eq!(none.discretionary_ligatures, Some(false));
    assert_eq!(none.historical_ligatures, Some(false));
    assert_eq!(none.contextual_alternates, Some(false));
    let enabled = style_for(
        "font-variant-ligatures",
        "common-ligatures discretionary-ligatures historical-ligatures contextual",
    );
    assert_eq!(enabled.common_ligatures, Some(true));
    assert_eq!(enabled.discretionary_ligatures, Some(true));
    assert_eq!(enabled.historical_ligatures, Some(true));
    assert_eq!(enabled.contextual_alternates, Some(true));
    let normal = style_for("font-variant-ligatures", "normal");
    assert_eq!(normal.common_ligatures, None);
    assert_eq!(normal.discretionary_ligatures, None);
    assert_eq!(normal.historical_ligatures, None);
    assert_eq!(normal.contextual_alternates, None);
}

#[test]
fn covers_every_font_caps_and_position_value() {
    for (value, expected) in [
        ("normal", FontVariantCaps::Normal),
        ("small-caps", FontVariantCaps::SmallCaps),
        ("all-small-caps", FontVariantCaps::AllSmallCaps),
        ("petite-caps", FontVariantCaps::PetiteCaps),
        ("all-petite-caps", FontVariantCaps::AllPetiteCaps),
        ("unicase", FontVariantCaps::Unicase),
        ("titling-caps", FontVariantCaps::TitlingCaps),
    ] {
        assert_eq!(
            style_for("font-variant-caps", value).font_variant_caps,
            expected
        );
    }
    for (value, expected) in [
        ("normal", FontVariantPosition::Normal),
        ("sub", FontVariantPosition::Sub),
        ("super", FontVariantPosition::Super),
    ] {
        assert_eq!(
            style_for("font-variant-position", value).font_variant_position,
            expected
        );
    }
}

#[test]
fn covers_every_mapped_cursor_keyword() {
    let cases = [
        ("auto", CursorIcon::Text),
        ("text", CursorIcon::Text),
        ("default", CursorIcon::Default),
        ("pointer", CursorIcon::Pointer),
        ("crosshair", CursorIcon::Crosshair),
        ("move", CursorIcon::Move),
        ("wait", CursorIcon::Wait),
        ("progress", CursorIcon::Progress),
        ("help", CursorIcon::Help),
        ("not-allowed", CursorIcon::NotAllowed),
        ("no-drop", CursorIcon::NoDrop),
        ("grab", CursorIcon::Grab),
        ("grabbing", CursorIcon::Grabbing),
        ("copy", CursorIcon::Copy),
        ("alias", CursorIcon::Alias),
        ("context-menu", CursorIcon::ContextMenu),
        ("cell", CursorIcon::Cell),
        ("all-scroll", CursorIcon::AllScroll),
        ("vertical-text", CursorIcon::VerticalText),
        ("ew-resize", CursorIcon::EwResize),
        ("col-resize", CursorIcon::EwResize),
        ("ns-resize", CursorIcon::NsResize),
        ("row-resize", CursorIcon::NsResize),
        ("nesw-resize", CursorIcon::NeswResize),
        ("nwse-resize", CursorIcon::NwseResize),
        ("n-resize", CursorIcon::NResize),
        ("e-resize", CursorIcon::EResize),
        ("s-resize", CursorIcon::SResize),
        ("w-resize", CursorIcon::WResize),
        ("ne-resize", CursorIcon::NeResize),
        ("nw-resize", CursorIcon::NwResize),
        ("se-resize", CursorIcon::SeResize),
        ("sw-resize", CursorIcon::SwResize),
        ("zoom-in", CursorIcon::ZoomIn),
        ("zoom-out", CursorIcon::ZoomOut),
    ];
    for (value, expected) in cases {
        assert_eq!(style_for("cursor", value).cursor, expected, "{value}");
        assert!(!style_for("cursor", value).cursor_hidden, "{value}");
    }
    let hidden = style_for("cursor", "none");
    assert_eq!(hidden.cursor, CursorIcon::Default);
    assert!(hidden.cursor_hidden);
}

#[test]
fn cascade_uses_specificity_then_source_order_for_each_declaration() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: Some("target".to_owned()),
    };
    let rules = [
        ("p", "#0000ff"),
        (".sample", "#008000"),
        ("#target", "#ff0000"),
        ("p.sample", "#ffffff"),
    ]
    .map(|(selector, value)| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "color".to_owned(),
            value: value.to_owned(),
        }],
    });
    let mut style = TextStyle::default();
    apply_matching_styles(&mut style, &current, &[], &rules);
    assert_eq!(style.color, [255, 0, 0, 255]);
}

#[test]
fn important_declarations_precede_specificity_and_source_order() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: Some("target".to_owned()),
    };
    let rules = [
        ("#target", "#ff0000"),
        ("p", "#0000ff !IMPORTANT"),
        ("#target", "#008000"),
        (".sample", "#ffffff !important"),
    ]
    .map(|(selector, value)| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "color".to_owned(),
            value: value.to_owned(),
        }],
    });
    let mut style = TextStyle::default();

    apply_matching_styles(&mut style, &current, &[], &rules);

    assert_eq!(style.color, [255, 255, 255, 255]);
}

#[test]
fn cascade_layers_order_normal_and_important_declarations() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: Some("target".to_owned()),
    };
    let stylesheet = ctml_css::parse_stylesheet(
        "@layer low, high; @layer low { #target { color: red; } p { color: red !important; } } @layer high { p { color: blue; } #target { color: blue !important; } } p { color: green !important; }",
    )
    .unwrap();
    let mut style = TextStyle::default();

    apply_matching_styles(&mut style, &current, &[], &stylesheet.rules);

    assert_eq!(style.color, [255, 0, 0, 255]);
}

#[test]
fn later_layers_outrank_earlier_layers_before_specificity() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: Some("target".to_owned()),
    };
    let stylesheet = ctml_css::parse_stylesheet(
        "@layer low, high; @layer low { #target { color: red; } } @layer high { p { color: blue; } }",
    )
    .unwrap();
    let mut style = TextStyle::default();

    apply_matching_styles(&mut style, &current, &[], &stylesheet.rules);

    assert_eq!(style.color, [0, 0, 255, 255]);
}

#[test]
fn unlayered_normal_rules_outrank_layered_normal_rules() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: Some("target".to_owned()),
    };
    let stylesheet =
        ctml_css::parse_stylesheet("@layer theme { #target { color: red; } } p { color: blue; }")
            .unwrap();
    let mut style = TextStyle::default();

    apply_matching_styles(&mut style, &current, &[], &stylesheet.rules);

    assert_eq!(style.color, [0, 0, 255, 255]);
}

#[test]
fn attribute_selectors_apply_through_the_style_cascade() {
    let current = ElementIdentity {
        tag: "input".to_owned(),
        class: None,
        id: None,
    };
    let attributes = [("type".to_owned(), Some("TeXt".to_owned()))];
    let rules = [
        StyleRule {
            selectors: vec!["input[type=text i]".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "color".to_owned(),
                value: "#008000".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["input[type=password]".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-weight".to_owned(),
                value: "700".to_owned(),
            }],
        },
    ];
    let mut style = TextStyle::default();

    apply_matching_styles_with_attributes(&mut style, &current, &[], &attributes, &rules);

    assert_eq!(style.color, [0, 128, 0, 255]);
    assert_eq!(style.weight, Weight::NORMAL);
}

#[test]
fn css_wide_inherit_initial_and_unset_keywords_resolve_supported_properties() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };
    let rules = [
        StyleRule {
            selectors: vec![".sample".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "color".to_owned(),
                    value: "#ff0000".to_owned(),
                },
                StyleDeclaration {
                    property: "font-weight".to_owned(),
                    value: "700".to_owned(),
                },
                StyleDeclaration {
                    property: "font-size".to_owned(),
                    value: "32px".to_owned(),
                },
                StyleDeclaration {
                    property: "background-color".to_owned(),
                    value: "#000000".to_owned(),
                },
                StyleDeclaration {
                    property: "text-decoration-line".to_owned(),
                    value: "underline".to_owned(),
                },
            ],
        },
        StyleRule {
            selectors: vec![".sample".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "color".to_owned(),
                    value: "initial".to_owned(),
                },
                StyleDeclaration {
                    property: "font-weight".to_owned(),
                    value: "inherit".to_owned(),
                },
                StyleDeclaration {
                    property: "font-size".to_owned(),
                    value: "unset".to_owned(),
                },
                StyleDeclaration {
                    property: "background-color".to_owned(),
                    value: "unset".to_owned(),
                },
                StyleDeclaration {
                    property: "text-decoration-line".to_owned(),
                    value: "unset".to_owned(),
                },
            ],
        },
    ];
    let inherited = TextStyle {
        color: [0, 0, 255, 255],
        weight: Weight(300),
        font_size: 24.0,
        background: [12, 34, 56],
        ..TextStyle::default()
    };

    let style = resolve_style(&inherited, &current, &[], &rules);

    assert_eq!(style.color, [0, 0, 0, 255]);
    assert_eq!(style.weight, Weight(300));
    assert_eq!(style.font_size, 24.0);
    assert_eq!(style.background, [255, 255, 255]);
    assert!(!style.underline);
}

#[test]
fn selector_matrix_covers_type_class_id_universal_compound_and_descendant() {
    use super::selector_matches;

    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let main = ElementIdentity {
        tag: "main".to_owned(),
        class: Some("content".to_owned()),
        id: Some("root".to_owned()),
    };
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("notice highlighted".to_owned()),
        id: Some("message".to_owned()),
    };
    let ancestors = [body, main];

    for selector in [
        "p",
        ".notice",
        "#message",
        "*",
        "p.notice",
        "p.notice.highlighted",
        "body main p",
    ] {
        assert!(
            selector_matches(selector, &paragraph, &ancestors),
            "{selector}"
        );
    }
    assert!(!selector_matches("#other", &paragraph, &ancestors));
    assert!(!selector_matches("article", &paragraph, &ancestors));
}

#[test]
fn selector_group_matches_the_most_specific_member_only_once() {
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: Some("target".to_owned()),
    };
    let rule = StyleRule {
        selectors: vec!["p".to_owned(), ".sample".to_owned(), "#target".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "font-weight".to_owned(),
            value: "700".to_owned(),
        }],
    };
    let style = resolve_style(&TextStyle::default(), &current, &[], &[rule]);
    assert_eq!(style.weight, Weight::BOLD);
}
