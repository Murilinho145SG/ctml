use cosmic_text::{Stretch, Style, Weight};
use winit::window::CursorIcon;

use super::{ElementIdentity, resolve_style, selector_matches, selector_specificity};
use crate::{
    StyleDeclaration, StyleRule,
    style::{TextDecorationStyle, TextStyle},
};

pub(super) fn style_for_declarations(declarations: &[(&str, &str)]) -> TextStyle {
    let rule = StyleRule {
        selectors: vec![".sample".to_owned()],
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
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };
    resolve_style(&TextStyle::default(), &paragraph, &[body], &[rule])
}

#[test]
fn resolves_numeric_weight_font_style_and_stretch() {
    let style = style_for_declarations(&[
        ("font-weight", "350"),
        ("font-style", "oblique"),
        ("font-stretch", "condensed"),
    ]);

    assert_eq!(style.weight, Weight(350));
    assert_eq!(style.font_style, Style::Oblique);
    assert_eq!(style.stretch, Stretch::Condensed);
}

#[test]
fn resolves_relative_font_weight_keywords() {
    assert_eq!(
        style_for_declarations(&[("font-weight", "bolder")]).weight,
        Weight(700)
    );
    assert_eq!(
        style_for_declarations(&[("font-weight", "lighter")]).weight,
        Weight(100)
    );

    let rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        declarations: vec![StyleDeclaration {
            property: "font-weight".to_owned(),
            value: "lighter".to_owned(),
        }],
    };
    let inherited = TextStyle {
        weight: Weight(700),
        ..TextStyle::default()
    };
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };
    assert_eq!(
        resolve_style(&inherited, &paragraph, &[body], &[rule]).weight,
        Weight(400)
    );
}

#[test]
fn maps_font_stretch_percentages_to_available_widths() {
    assert_eq!(
        style_for_declarations(&[("font-stretch", "75%")]).stretch,
        Stretch::Condensed
    );
    assert_eq!(
        style_for_declarations(&[("font-stretch", "100%")]).stretch,
        Stretch::Normal
    );
}

#[test]
fn resolves_font_size_percentages_against_the_inherited_size() {
    let inherited = TextStyle {
        font_size: 20.0,
        line_height: 28.0,
        line_height_multiplier: None,
        ..TextStyle::default()
    };
    let rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        declarations: vec![StyleDeclaration {
            property: "font-size".to_owned(),
            value: "125%".to_owned(),
        }],
    };
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };

    let style = resolve_style(&inherited, &paragraph, &[body], &[rule]);

    assert_eq!(style.font_size, 25.0);
    assert_eq!(style.line_height, 28.0);
}

#[test]
fn resolves_font_relative_em_and_rem_sizes() {
    assert_eq!(super::parse_length("1.5em", 20.0), Some(30.0));
    assert_eq!(super::parse_length("1.5rem", 20.0), Some(24.0));

    let inherited = TextStyle {
        font_size: 20.0,
        ..TextStyle::default()
    };
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("em".to_owned()),
        id: None,
    };
    let stylesheet = [
        StyleRule {
            selectors: vec![".em".to_owned()],
            declarations: vec![StyleDeclaration {
                property: "font-size".to_owned(),
                value: "1.5em".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".rem".to_owned()],
            declarations: vec![StyleDeclaration {
                property: "font-size".to_owned(),
                value: "1.5rem".to_owned(),
            }],
        },
    ];

    assert_eq!(
        resolve_style(&inherited, &paragraph, &[body], &stylesheet).font_size,
        30.0
    );
    let rem_paragraph = ElementIdentity {
        class: Some("rem".to_owned()),
        ..paragraph
    };
    assert_eq!(
        resolve_style(
            &inherited,
            &rem_paragraph,
            &[ElementIdentity {
                tag: "body".to_owned(),
                class: None,
                id: None,
            }],
            &stylesheet
        )
        .font_size,
        24.0
    );
}

#[test]
fn resolves_absolute_css_font_size_units() {
    for value in [
        "16px",
        "0.16666667in",
        "0.42333334cm",
        "4.233333mm",
        "16.933334Q",
        "12pt",
        "1pc",
    ] {
        let style = style_for_declarations(&[("font-size", value)]);
        assert!(
            (style.font_size - 16.0).abs() < 0.001,
            "{value} should resolve to 16 CSS pixels, got {}",
            style.font_size
        );
    }
}

#[test]
fn resolves_absolute_font_size_keywords() {
    let cases = [
        ("xx-small", 9.0),
        ("x-small", 10.0),
        ("small", 13.0),
        ("medium", 16.0),
        ("large", 18.0),
        ("x-large", 24.0),
        ("xx-large", 32.0),
        ("xxx-large", 48.0),
    ];

    for (keyword, expected_size) in cases {
        let style = style_for_declarations(&[("font-size", keyword)]);
        assert_eq!(style.font_size, expected_size, "font-size: {keyword}");
    }
}

#[test]
fn preserves_the_font_family_fallback_list() {
    let style = style_for_declarations(&[("font-family", "\"Missing Font\", serif")]);
    assert_eq!(style.family, "\"Missing Font\", serif");
}

#[test]
fn resolves_relative_font_size_keywords() {
    assert_eq!(super::parse_font_size("larger", 16.0), Some(19.2));
    assert_eq!(super::parse_font_size("smaller", 16.0), Some(16.0 / 1.2));
    assert_eq!(super::parse_font_size("larger", 20.0), Some(24.0));
    assert_eq!(super::parse_font_size("smaller", 20.0), Some(20.0 / 1.2));
}

#[test]
fn scales_inherited_unitless_line_height_with_font_size() {
    let inherited = TextStyle {
        font_size: 20.0,
        line_height: 30.0,
        line_height_multiplier: Some(1.5),
        ..TextStyle::default()
    };
    let rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        declarations: vec![StyleDeclaration {
            property: "font-size".to_owned(),
            value: "125%".to_owned(),
        }],
    };
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };

    let style = resolve_style(&inherited, &paragraph, &[body], &[rule]);

    assert_eq!(style.font_size, 25.0);
    assert_eq!(style.line_height, 37.5);
}

#[test]
fn resolves_font_kerning_values() {
    assert_eq!(
        style_for_declarations(&[("font-kerning", "none")]).kerning,
        Some(false)
    );
    assert_eq!(
        style_for_declarations(&[("font-kerning", "normal")]).kerning,
        Some(true)
    );
}

#[test]
fn resolves_standard_named_css_colors() {
    let style = style_for_declarations(&[("color", "rebeccapurple")]);

    assert_eq!(style.color, [102, 51, 153, 255]);
}

#[test]
fn resolves_alpha_text_color_and_blends_page_background_with_white() {
    let style = style_for_declarations(&[
        ("color", "#00000080"),
        ("background-color", "rgba(0, 0, 0, 50%)"),
    ]);

    assert_eq!(style.color, [0, 0, 0, 128]);
    assert_eq!(style.background, [128, 128, 128]);
}

#[test]
fn resolves_transparent_text_and_page_background() {
    let style = style_for_declarations(&[
        ("color", "transparent"),
        ("background-color", "transparent"),
    ]);

    assert_eq!(style.color, [0, 0, 0, 0]);
    assert_eq!(style.background, [255, 255, 255]);
}

#[test]
fn resolves_current_color_after_cascade() {
    let style =
        style_for_declarations(&[("background-color", "currentColor"), ("color", "#123456")]);

    assert_eq!(style.color, [18, 52, 86, 255]);
    assert_eq!(style.background, [18, 52, 86]);
}

#[test]
fn preserves_fractional_opacity_when_color_becomes_current_color() {
    let style = style_for_declarations(&[
        ("color", "rgb(51 102 153 / 50%)"),
        ("background-color", "currentColor"),
    ]);

    assert_eq!(style.color, [51, 102, 153, 128]);
    assert_eq!(style.color_opacity, 0.5);
    assert_eq!(style.background, [153, 179, 204]);
}

#[test]
fn current_color_for_color_uses_the_inherited_color() {
    let rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        declarations: vec![StyleDeclaration {
            property: "color".to_owned(),
            value: "currentColor".to_owned(),
        }],
    };
    let inherited = TextStyle {
        color: [12, 34, 56, 255],
        color_opacity: 1.0,
        ..TextStyle::default()
    };
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };

    let style = resolve_style(&inherited, &paragraph, &[body], &[rule]);

    assert_eq!(style.color, [12, 34, 56, 255]);
}

#[test]
fn resolves_supported_text_alignment_values() {
    assert_eq!(
        style_for_declarations(&[("text-align", "center")]).text_align,
        cosmic_text::Align::Center
    );
    assert_eq!(
        style_for_declarations(&[("text-align", "justify")]).text_align,
        cosmic_text::Align::Justified
    );
}

#[test]
fn resolves_solid_double_and_patterned_text_decoration_styles() {
    for (value, expected) in [
        ("solid", super::super::style::TextDecorationStyle::Solid),
        ("double", super::super::style::TextDecorationStyle::Double),
        ("dotted", super::super::style::TextDecorationStyle::Dotted),
        ("dashed", super::super::style::TextDecorationStyle::Dashed),
        ("wavy", super::super::style::TextDecorationStyle::Wavy),
    ] {
        assert_eq!(
            style_for_declarations(&[("text-decoration-style", value)]).decoration_style,
            expected,
            "{value}"
        );
    }
    assert_eq!(
        style_for_declarations(&[("text-decoration", "underline solid")]).decoration_style,
        super::super::style::TextDecorationStyle::Solid
    );
}

#[test]
fn resolves_hidden_cursor_value() {
    let style = style_for_declarations(&[("cursor", "none")]);

    assert_eq!(style.cursor, CursorIcon::Default);
    assert!(style.cursor_hidden);
}

#[test]
fn matches_direct_child_combinators_and_specificity() {
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };
    let span = ElementIdentity {
        tag: "span".to_owned(),
        class: Some("accent".to_owned()),
        id: None,
    };

    assert!(selector_matches(
        "p.sample > span.accent",
        &span,
        &[body.clone(), paragraph.clone()]
    ));
    assert!(selector_matches(
        "p.sample>span.accent",
        &span,
        &[body.clone(), paragraph.clone()]
    ));
    assert!(!selector_matches(
        "body > span.accent",
        &span,
        &[body, paragraph]
    ));
    assert_eq!(selector_specificity("p.sample > span.accent"), (0, 2, 2));
}

#[test]
fn ignores_invalid_numeric_font_weight() {
    let style = style_for_declarations(&[("font-weight", "1001")]);

    assert_eq!(style.weight, Weight::NORMAL);
}

#[test]
fn applies_text_decoration_color_to_active_lines() {
    let style = style_for_declarations(&[
        ("text-decoration", "underline line-through"),
        ("text-decoration-color", "#ff008080"),
    ]);

    assert!(style.underline);
    assert!(style.strikethrough);
    assert_eq!(style.text_decoration_color, Some([255, 0, 128, 128]));
}

#[test]
fn text_decoration_shorthand_resets_inherited_style_and_color() {
    let rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        declarations: vec![StyleDeclaration {
            property: "text-decoration".to_owned(),
            value: "overline".to_owned(),
        }],
    };
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };
    let inherited = TextStyle {
        decoration_style: TextDecorationStyle::Wavy,
        decoration_thickness: Some(3.0),
        text_decoration_color: Some([51, 102, 204, 255]),
        ..TextStyle::default()
    };

    let style = resolve_style(&inherited, &paragraph, &[body], &[rule]);

    assert_eq!(style.decoration_style, TextDecorationStyle::Solid);
    assert_eq!(style.decoration_thickness, None);
    assert_eq!(style.text_decoration_color, None);
    assert!(style.overline);
}
