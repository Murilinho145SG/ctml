use super::{
    FontVariantCaps, TextOrientation, TextRendering, TextRun, TextStyle, WhiteSpace, append_text,
    compile_runs_with_font_system, synthesize_font_variant_caps,
};
use crate::{StyleDeclaration, StyleRule, UiNode};

fn compile_runs(
    nodes: &[UiNode],
    stylesheet: &[StyleRule],
    hovered_path: &[usize],
    active_path: &[usize],
    focused_path: &[usize],
    keyboard_focus: bool,
) -> (Vec<TextRun>, TextStyle) {
    compile_runs_with_font_system(
        nodes,
        stylesheet,
        hovered_path,
        active_path,
        focused_path,
        keyboard_focus,
        None,
        &[],
        ctml_css::ViewportSize::default(),
    )
}

#[test]
fn resolves_custom_icc_colors_in_native_text_styles() {
    let profile = ctml_css::CssColorProfile {
        name: "--screen".to_owned(),
        source: "url(test.icc)".to_owned(),
        rendering_intent: "relative-colorimetric".to_owned(),
        components: vec!["red".to_owned(), "green".to_owned(), "blue".to_owned()],
        profile_data: Some(lcms2::Profile::new_srgb().icc().unwrap()),
        stylesheet_path: None,
    };
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("profile color".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        declarations: vec![
            StyleDeclaration {
                property: "color".to_owned(),
                value: "color-mix(in --screen, color(--screen 1 0 0), blue)".to_owned(),
            },
            StyleDeclaration {
                property: "background-color".to_owned(),
                value: "color(--screen 1 0 0)".to_owned(),
            },
        ],
        layer: None,
    }];

    let (runs, _) = compile_runs_with_font_system(
        &nodes,
        &stylesheet,
        &[],
        &[],
        &[],
        false,
        None,
        &[profile],
        ctml_css::ViewportSize::default(),
    );

    assert_eq!(runs[0].style.color, [127, 0, 128, 255]);
    assert_eq!(runs[0].style.background_color, Some([255, 0, 0, 255]));
}

#[test]
fn preserves_small_caps_for_font_feature_shaping() {
    let style = TextStyle {
        font_size: 12.0,
        line_height: 20.0,
        font_variant_caps: FontVariantCaps::SmallCaps,
        ..TextStyle::default()
    };
    let mut runs = Vec::<TextRun>::new();
    let (mut last_was_space, mut capitalize_next) = (false, false);

    append_text(
        "aB",
        &style,
        &[],
        &mut runs,
        &mut last_was_space,
        &mut capitalize_next,
    );

    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].text, "aB");
    assert_eq!(runs[0].style.font_scale, 1.0);
    assert_eq!(runs[0].style.font_size, 12.0);
    assert_eq!(runs[0].style.line_height, 20.0);
    assert_eq!(runs[0].style.font_variant_caps, FontVariantCaps::SmallCaps);
}

#[test]
fn white_space_modes_collapse_or_preserve_source_text() {
    let mut runs = Vec::<TextRun>::new();
    let (mut last_was_space, mut capitalize_next) = (true, false);
    append_text(
        "  one \t two\nthree  ",
        &TextStyle::default(),
        &[],
        &mut runs,
        &mut last_was_space,
        &mut capitalize_next,
    );
    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "one two three "
    );

    let pre = TextStyle {
        white_space: WhiteSpace::Pre,
        ..TextStyle::default()
    };
    runs.clear();
    last_was_space = false;
    append_text(
        "  one \t two\nthree  ",
        &pre,
        &[],
        &mut runs,
        &mut last_was_space,
        &mut capitalize_next,
    );
    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "  one \t two\nthree  "
    );

    let pre_line = TextStyle {
        white_space: WhiteSpace::PreLine,
        ..TextStyle::default()
    };
    runs.clear();
    last_was_space = false;
    append_text(
        "one \t two\nthree",
        &pre_line,
        &[],
        &mut runs,
        &mut last_was_space,
        &mut capitalize_next,
    );
    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "one two\nthree"
    );
}

#[test]
fn generated_open_and_close_quotes_follow_nested_depth() {
    let nested_quote = UiNode::Element {
        tag: "q".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("inner".to_owned())],
    };
    let nodes = [UiNode::Element {
        tag: "q".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("outer ".to_owned()), nested_quote],
    }];
    let stylesheet = [
        ("q", "quotes", "\"«\" \"»\" \"‹\" \"›\""),
        ("q::before", "content", "open-quote"),
        ("q::after", "content", "close-quote"),
    ]
    .map(|(selector, property, value)| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    });

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "«outer ‹inner›»"
    );
}

#[test]
fn generated_counters_follow_nested_scopes_and_sibling_increments() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("root".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![
            counter_item("A", None),
            counter_item(
                "B",
                Some(UiNode::Element {
                    tag: "span".to_owned(),
                    class: Some("nested".to_owned()),
                    id: None,
                    attributes: Vec::new(),
                    children: Vec::new(),
                }),
            ),
            counter_item("C", None),
        ],
    }];
    let stylesheet = [
        (".root", "counter-reset", "chapter"),
        (".item", "counter-increment", "chapter"),
        (".item::before", "content", "counter(chapter) \"/\""),
        (".nested", "counter-reset", "chapter 5"),
        (".nested::before", "content", "counters(chapter, \".\")"),
    ]
    .map(|(selector, property, value)| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    });

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "1/A2/B2.53/C"
    );
}

#[test]
fn generated_counters_support_alpha_greek_and_roman_styles() {
    let nodes = [UiNode::Element {
        tag: "span".to_owned(),
        class: Some("numbered".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: Vec::new(),
    }];
    let stylesheet = [
        (".numbered", "counter-set", "chapter 27"),
        (
            ".numbered::before",
            "content",
            "counter(chapter, lower-alpha) \"/\" counter(chapter, upper-roman) \"/\" counter(chapter, lower-greek)",
        ),
    ]
    .map(|(selector, property, value)| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    });

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "aa/XXVII/αγ"
    );
}

#[test]
fn first_letter_applies_font_optical_sizing_and_variation_settings() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Variable font".to_owned())],
    }];
    let stylesheet = [
        ("p::first-letter", "font-style", "oblique 18deg"),
        ("p::first-letter", "font-optical-sizing", "none"),
        ("p::first-letter", "text-rendering", "optimizeSpeed"),
        (
            "p::first-letter",
            "font-variation-settings",
            "\"wght\" 725, \"wdth\" 90",
        ),
    ]
    .map(|(selector, property, value)| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    });

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);
    let first_letter = runs.iter().find(|run| run.text == "V").unwrap();

    assert!(!first_letter.style.font_optical_sizing);
    assert_eq!(first_letter.style.font_oblique_angle, Some(18.0));
    assert!(
        first_letter
            .style
            .to_attrs("sans-serif")
            .font_variations
            .iter()
            .any(|setting| setting.tag() == *b"slnt" && setting.value() == -18.0)
    );
    assert_eq!(
        first_letter.style.text_rendering,
        TextRendering::OptimizeSpeed
    );
    assert_eq!(
        first_letter.style.font_variation_settings,
        vec![(*b"wght", 725.0), (*b"wdth", 90.0)]
    );
}

#[test]
fn first_letter_applies_text_orientation() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Orientation".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-letter".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "text-orientation".to_owned(),
            value: "sideways".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);
    let first_letter = runs.iter().find(|run| run.text == "O").unwrap();

    assert_eq!(
        first_letter.style.text_orientation,
        TextOrientation::Sideways
    );
}

#[test]
fn first_line_resolves_font_optical_sizing_and_variation_settings() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Variable font".to_owned())],
    }];
    let stylesheet = [
        ("p::first-line", "font-optical-sizing", "none"),
        ("p::first-line", "text-rendering", "optimizeSpeed"),
        (
            "p::first-line",
            "font-variation-settings",
            "\"wght\" 650, \"opsz\" 13",
        ),
    ]
    .map(|(selector, property, value)| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    });

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);
    let first_line = runs[0].first_line_font_style.as_ref().unwrap();

    assert!(!first_line.font_optical_sizing);
    assert_eq!(first_line.text_rendering, TextRendering::OptimizeSpeed);
    assert_eq!(
        first_line.font_variation_settings,
        vec![(*b"wght", 650.0), (*b"opsz", 13.0)]
    );
}

#[test]
fn first_line_and_first_letter_resolve_text_emphasis_side() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Emphasis side".to_owned())],
    }];
    let stylesheet = [
        ("p::first-line", "text-emphasis-position", "under left"),
        ("p::first-letter", "text-emphasis-position", "over right"),
    ]
    .map(|(selector, property, value)| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    });

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);
    let first_letter = runs.iter().find(|run| run.text == "E").unwrap();
    let first_line = first_letter.first_line_font_style.as_ref().unwrap();

    assert_eq!(
        first_letter.style.text_emphasis_side,
        Some(crate::style::TextEmphasisSide::Right)
    );
    assert_eq!(
        first_line.text_emphasis_side,
        Some(crate::style::TextEmphasisSide::Left)
    );
}

fn counter_item(text: &str, nested: Option<UiNode>) -> UiNode {
    let mut children = vec![UiNode::Text(text.to_owned())];
    children.extend(nested);
    UiNode::Element {
        tag: "span".to_owned(),
        class: Some("item".to_owned()),
        id: None,
        attributes: Vec::new(),
        children,
    }
}

#[test]
fn preserves_all_small_caps_for_font_feature_shaping() {
    let style = TextStyle {
        font_variant_caps: FontVariantCaps::AllSmallCaps,
        ..TextStyle::default()
    };
    let mut runs = Vec::<TextRun>::new();
    let (mut last_was_space, mut capitalize_next) = (false, false);

    append_text(
        "aB",
        &style,
        &[],
        &mut runs,
        &mut last_was_space,
        &mut capitalize_next,
    );

    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].text, "aB");
    assert_eq!(runs[0].style.font_scale, 1.0);
    assert_eq!(
        runs[0].style.font_variant_caps,
        FontVariantCaps::AllSmallCaps
    );
}

#[test]
fn synthesizes_small_caps_when_the_font_has_no_feature() {
    let style = TextStyle {
        font_size: 12.0,
        line_height: 20.0,
        font_variant_caps: FontVariantCaps::SmallCaps,
        ..TextStyle::default()
    };
    let runs = synthesize_font_variant_caps(TextRun {
        text: "aB".to_owned(),
        source_first_letter_character: Some('a'),
        style,
        element_path: Vec::new(),
        selection_background: Some([255, 0, 0, 255]),
        selection_foreground: Some([0, 255, 0, 255]),
        first_line_block: Some(3),
        first_line_foreground: Some([0, 0, 255, 255]),
        first_line_background: Some([255, 255, 0, 255]),
        first_line_decoration: None,
        first_line_opacity: None,
        first_line_font_style: None,
        first_letter_block: Some(3),
        first_letter_foreground: Some([255, 0, 255, 255]),
        first_letter_background: Some([0, 255, 255, 255]),
        first_letter_decoration: None,
        first_letter_opacity: None,
        first_letter_font_style: None,
    });

    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].text, "A");
    assert_eq!(runs[0].style.font_scale, 2.0 / 3.0);
    assert_eq!(runs[0].style.font_variant_caps, FontVariantCaps::Normal);
    assert_eq!(runs[0].style.line_height, 20.0);
    assert_eq!(runs[0].selection_background, Some([255, 0, 0, 255]));
    assert_eq!(runs[0].selection_foreground, Some([0, 255, 0, 255]));
    assert_eq!(runs[0].first_line_block, Some(3));
    assert_eq!(runs[0].first_line_foreground, Some([0, 0, 255, 255]));
    assert_eq!(runs[0].first_line_background, Some([255, 255, 0, 255]));
    assert_eq!(runs[0].first_letter_block, Some(3));
    assert_eq!(runs[0].first_letter_foreground, Some([255, 0, 255, 255]));
    assert_eq!(runs[0].first_letter_background, Some([0, 255, 255, 255]));
    assert_eq!(runs[1].text, "B");
    assert_eq!(runs[1].style.font_scale, 1.0);
    assert_eq!(runs[1].selection_background, Some([255, 0, 0, 255]));
    assert_eq!(runs[1].selection_foreground, Some([0, 255, 0, 255]));
    assert_eq!(runs[1].first_line_block, Some(3));
    assert_eq!(runs[1].first_line_foreground, Some([0, 0, 255, 255]));
    assert_eq!(runs[1].first_line_background, Some([255, 255, 0, 255]));
    assert_eq!(runs[1].first_letter_block, Some(3));
    assert_eq!(runs[1].first_letter_foreground, Some([255, 0, 255, 255]));
    assert_eq!(runs[1].first_letter_background, Some([0, 255, 255, 255]));
}

#[test]
fn rounds_synthetic_caps_to_a_pixel_size() {
    for caps in [FontVariantCaps::SmallCaps, FontVariantCaps::PetiteCaps] {
        let style = TextStyle {
            font_size: 8.0,
            font_variant_caps: caps,
            ..TextStyle::default()
        };
        let runs = synthesize_font_variant_caps(TextRun {
            text: "a".to_owned(),
            source_first_letter_character: Some('a'),
            style,
            element_path: Vec::new(),
            selection_background: None,
            selection_foreground: None,
            first_line_block: None,
            first_line_foreground: None,
            first_line_background: None,
            first_line_decoration: None,
            first_line_opacity: None,
            first_line_font_style: None,
            first_letter_block: None,
            first_letter_foreground: None,
            first_letter_background: None,
            first_letter_decoration: None,
            first_letter_opacity: None,
            first_letter_font_style: None,
        });

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "A");
        assert_eq!(runs[0].style.font_size * runs[0].style.font_scale, 6.0);
    }
}

#[test]
fn generates_before_and_after_text_with_pseudo_element_styles() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("body".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p::before".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "content".to_owned(),
                    value: "\"before\"".to_owned(),
                },
                StyleDeclaration {
                    property: "color".to_owned(),
                    value: "blue".to_owned(),
                },
            ],
        },
        StyleRule {
            selectors: vec!["p:after".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "content".to_owned(),
                    value: "'after'".to_owned(),
                },
                StyleDeclaration {
                    property: "font-weight".to_owned(),
                    value: "700".to_owned(),
                },
            ],
        },
    ];
    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "beforebodyafter"
    );
    assert_eq!(runs[0].style.color, [0, 0, 255, 255]);
    assert_eq!(
        runs.iter()
            .find(|run| run.text == "after")
            .unwrap()
            .style
            .weight,
        cosmic_text::Weight(700)
    );
}

#[test]
fn generated_content_concatenates_strings_and_attribute_values() {
    let nodes = [UiNode::Element {
        tag: "a".to_owned(),
        class: None,
        id: None,
        attributes: vec![("title".to_owned(), Some("documentation".to_owned()))],
        children: vec![UiNode::Text("link".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["a::before".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "content".to_owned(),
            value: "\"Open \" attr(title) \" now\"".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "Open documentation nowlink"
    );
}

#[test]
fn missing_generated_content_attribute_contributes_an_empty_string() {
    let nodes = [UiNode::Element {
        tag: "a".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("link".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["a::before".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "content".to_owned(),
            value: "\"Open \" attr(title)".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "Open link"
    );
}

#[test]
fn generated_content_attr_uses_tokenized_fallbacks_only_when_missing() {
    let nodes = [
        UiNode::Element {
            tag: "a".to_owned(),
            class: None,
            id: None,
            attributes: vec![("title".to_owned(), Some("documentation".to_owned()))],
            children: Vec::new(),
        },
        UiNode::Element {
            tag: "span".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: Vec::new(),
        },
    ];
    let stylesheet = [
        ("a::before", "attr(title, \"unused\")"),
        ("span::before", "attr(title, \"missing \" label)"),
    ]
    .map(|(selector, value)| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "content".to_owned(),
            value: value.to_owned(),
        }],
    });

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "documentation missing label"
    );
}

#[test]
fn generated_content_attr_converts_string_number_and_integer_types() {
    let nodes = [UiNode::Element {
        tag: "span".to_owned(),
        class: None,
        id: None,
        attributes: vec![
            ("count".to_owned(), Some("0012".to_owned())),
            ("ratio".to_owned(), Some("2.50".to_owned())),
            ("invalid".to_owned(), Some("not-a-number".to_owned())),
        ],
        children: Vec::new(),
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["span::before".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "content".to_owned(),
            value: "attr(count type(<integer>),) \"/\" attr(ratio type(<number>),) \"/\" attr(invalid type(<integer>), \"N/A\") \"/\" attr(missing type(<integer>), 7)".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "12/2.5/ N/A/ 7"
    );
}

#[test]
fn generated_content_attr_converts_supported_css_dimension_types() {
    let nodes = [UiNode::Element {
        tag: "span".to_owned(),
        class: None,
        id: None,
        attributes: vec![
            ("length".to_owned(), Some("1.5REM".to_owned())),
            ("percent".to_owned(), Some("25%".to_owned())),
            ("angle".to_owned(), Some(".5turn".to_owned())),
            ("invalid".to_owned(), Some("2s".to_owned())),
        ],
        children: Vec::new(),
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["span::before".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "content".to_owned(),
            value: r#"attr(length type(<length>), "fallback") "/" attr(percent type(<length-percentage>),) "/" attr(angle type(<angle>),) "/" attr(invalid type(<length>), "invalid")"#.to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "1.5rem/25%/0.5turn/ invalid"
    );
}

#[test]
fn generated_content_attr_converts_custom_ident_values_and_uses_fallbacks() {
    let nodes = [UiNode::Element {
        tag: "span".to_owned(),
        class: None,
        id: None,
        attributes: vec![
            ("valid".to_owned(), Some("north-star".to_owned())),
            ("reserved".to_owned(), Some("default".to_owned())),
            ("multiple".to_owned(), Some("two words".to_owned())),
        ],
        children: Vec::new(),
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["span::before".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "content".to_owned(),
            value: r#"attr(valid type(<custom-ident>), fallback) "/" attr(reserved type(<custom-ident>), fallback) "/" attr(multiple type(<custom-ident>), fallback)"#.to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "north-star/ fallback/ fallback"
    );
}

#[test]
fn display_none_hides_element_descendants_and_generated_content() {
    let nodes = [
        UiNode::Element {
            tag: "div".to_owned(),
            class: Some("hidden".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("hidden child".to_owned())],
        },
        UiNode::Element {
            tag: "p".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("visible".to_owned())],
        },
    ];
    let stylesheet = [
        StyleRule {
            selectors: vec![".hidden".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "display".to_owned(),
                value: "none".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".hidden::before".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "content".to_owned(),
                value: "'hidden generated'".to_owned(),
            }],
        },
    ];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "visible"
    );
}

#[test]
fn display_none_on_body_hides_all_rendered_content() {
    let nodes = [UiNode::Text("hidden body".to_owned())];
    let stylesheet = [StyleRule {
        selectors: vec!["body".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "display".to_owned(),
            value: "none".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert!(runs.is_empty());
}

#[test]
fn display_block_and_inline_override_default_text_flow() {
    let nodes = [
        UiNode::Text("before".to_owned()),
        UiNode::Element {
            tag: "span".to_owned(),
            class: Some("block".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("middle".to_owned())],
        },
    ];
    let stylesheet = [StyleRule {
        selectors: vec![".block".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "display".to_owned(),
            value: "block".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "before\nmiddle"
    );
}

#[test]
fn display_inline_overrides_block_tag_line_separation() {
    let nodes = [
        UiNode::Text("before".to_owned()),
        UiNode::Element {
            tag: "p".to_owned(),
            class: Some("inline".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("middle".to_owned())],
        },
        UiNode::Text("after".to_owned()),
    ];
    let stylesheet = [StyleRule {
        selectors: vec![".inline".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "display".to_owned(),
            value: "inline".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "beforemiddleafter"
    );
}

#[test]
fn display_contents_removes_tag_line_separation_but_keeps_children() {
    let nodes = [
        UiNode::Text("before".to_owned()),
        UiNode::Element {
            tag: "div".to_owned(),
            class: Some("contents".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("middle".to_owned())],
        },
        UiNode::Text("after".to_owned()),
    ];
    let stylesheet = [StyleRule {
        selectors: vec![".contents".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "display".to_owned(),
            value: "contents".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "beforemiddleafter"
    );
}

#[test]
fn display_flow_root_uses_block_text_flow() {
    assert_display_flow("span", "flow-root", "before\nmiddle\nafter");
}

#[test]
fn display_inline_block_uses_inline_text_flow() {
    assert_display_flow("p", "inline-block", "beforemiddleafter");
}

#[test]
fn explicit_list_item_uses_a_bullet_and_block_flow() {
    let nodes = [UiNode::Element {
        tag: "span".to_owned(),
        class: Some("item".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("task".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec![".item".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "display".to_owned(),
            value: "list-item".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "• task"
    );
}

fn assert_display_flow(tag: &str, display: &str, expected: &str) {
    let nodes = [
        UiNode::Text("before".to_owned()),
        UiNode::Element {
            tag: tag.to_owned(),
            class: Some("target".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("middle".to_owned())],
        },
        UiNode::Text("after".to_owned()),
    ];
    let stylesheet = [StyleRule {
        selectors: vec![".target".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "display".to_owned(),
            value: display.to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        expected
    );
}
