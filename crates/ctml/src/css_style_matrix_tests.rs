use cosmic_text::{Align, Stretch, Style, Weight};
use winit::window::CursorIcon;

use super::{ElementIdentity, apply_matching_styles, resolve_style};
use crate::{
    StyleDeclaration, StyleRule,
    style::{
        FontVariantCaps, FontVariantPosition, TextDecorationSkipInk, TextDecorationStyle,
        TextStyle, TextTransform,
    },
};

fn style_for(property: &str, value: &str) -> TextStyle {
    let rule = StyleRule {
        selectors: vec![".case".to_owned()],
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
        declarations: vec![StyleDeclaration {
            property: "font-weight".to_owned(),
            value: "700".to_owned(),
        }],
    };
    let style = resolve_style(&TextStyle::default(), &current, &[], &[rule]);
    assert_eq!(style.weight, Weight::BOLD);
}
