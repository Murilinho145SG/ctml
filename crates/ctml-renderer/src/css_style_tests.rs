use cosmic_text::{Stretch, Style, Weight};
use winit::window::CursorIcon;

use super::{
    ElementIdentity, resolve_style, resolve_style_with_attributes, selector_matches,
    selector_specificity,
};
use crate::{
    StyleDeclaration, StyleRule,
    style::{TextDecorationStyle, TextStyle},
};

pub(super) fn style_for_declarations(declarations: &[(&str, &str)]) -> TextStyle {
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
fn resolves_typed_attr_values_in_supported_style_properties() {
    let rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "font-size".to_owned(),
                value: "attr(data-size type(<length>),)".to_owned(),
            },
            StyleDeclaration {
                property: "color".to_owned(),
                value: "attr(data-color type(<color>), red)".to_owned(),
            },
            StyleDeclaration {
                property: "text-decoration-color".to_owned(),
                value: "attr(data-invalid-color type(<color>), blue)".to_owned(),
            },
            StyleDeclaration {
                property: "opacity".to_owned(),
                value: "attr(data-opacity type(<number>), 0.5)".to_owned(),
            },
            StyleDeclaration {
                property: "letter-spacing".to_owned(),
                value: "attr(data-missing type(<length>), 3px)".to_owned(),
            },
        ],
    };
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };
    let attributes = [
        ("data-size".to_owned(), Some("22px".to_owned())),
        (
            "data-color".to_owned(),
            Some("rgb(255 0 0 / 50%)".to_owned()),
        ),
        ("data-opacity".to_owned(), Some("invalid".to_owned())),
        (
            "data-invalid-color".to_owned(),
            Some("not-a-color".to_owned()),
        ),
    ];

    let style =
        resolve_style_with_attributes(&TextStyle::default(), &paragraph, &[], &attributes, &[rule]);

    assert_eq!(style.font_size, 22.0);
    assert_eq!(style.color, [255, 0, 0, 128]);
    assert_eq!(style.text_decoration_color, Some([0, 0, 255, 255]));
    assert_eq!(style.opacity, 0.5);
    assert_eq!(style.letter_spacing, 3.0);
    assert!(style.letter_spacing_px);
}

#[test]
fn untyped_attr_is_a_css_string_not_an_injected_token_sequence() {
    let rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "color".to_owned(),
            value: "attr(data-color)".to_owned(),
        }],
    };
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };
    let attributes = [("data-color".to_owned(), Some("red".to_owned()))];

    let style =
        resolve_style_with_attributes(&TextStyle::default(), &paragraph, &[], &attributes, &[rule]);

    assert_eq!(style.color, [0, 0, 0, 255]);
}

#[test]
fn parses_exponential_numeric_font_weights() {
    assert_eq!(
        style_for_declarations(&[("font-weight", "1e2")]).weight,
        Weight(100)
    );
    assert_eq!(
        style_for_declarations(&[("font", "1e2 16px sans-serif")]).weight,
        Weight(100)
    );
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
        layer: None,
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
fn scalar_css_values_require_single_number_or_percentage_tokens() {
    assert_eq!(style_for_declarations(&[("opacity", "+.25")]).opacity, 0.25);
    assert_eq!(style_for_declarations(&[("opacity", "1 %")]).opacity, 1.0);
    assert_eq!(style_for_declarations(&[("opacity", "NaN")]).opacity, 1.0);
    assert_eq!(
        style_for_declarations(&[("line-height", "1.5")]).line_height,
        24.0
    );
    assert_eq!(
        style_for_declarations(&[("font-stretch", "75 %")]).stretch,
        Stretch::Normal
    );
    assert_eq!(
        style_for_declarations(&[("vertical-align", "50 %")]).vertical_align_offset,
        0.0
    );
    assert_eq!(
        style_for_declarations(&[("word-spacing", "25 %")]).word_spacing_percent,
        None
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
        layer: None,
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
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-size".to_owned(),
                value: "1.5em".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".rem".to_owned()],
            layer: None,
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
    assert_eq!(
        super::properties::parse_font_size("larger", 16.0),
        Some(19.2)
    );
    assert_eq!(
        super::properties::parse_font_size("smaller", 16.0),
        Some(16.0 / 1.2)
    );
    assert_eq!(
        super::properties::parse_font_size("larger", 20.0),
        Some(24.0)
    );
    assert_eq!(
        super::properties::parse_font_size("smaller", 20.0),
        Some(20.0 / 1.2)
    );
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
        layer: None,
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
fn resolves_system_colors_for_text_and_backgrounds() {
    let style =
        style_for_declarations(&[("color", "CanvasText"), ("background-color", "Highlight")]);

    assert_eq!(style.color, [0, 0, 0, 255]);
    assert_eq!(style.background_color, Some([0, 76, 192, 255]));
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
fn resolves_srgb_color_mix_for_text_and_page_background() {
    let style = style_for_declarations(&[
        ("color", "color-mix(in srgb, transparent, red)"),
        (
            "background-color",
            "color-mix(in srgb, #0000ff 25%, white 75%)",
        ),
    ]);

    assert_eq!(style.color, [255, 0, 0, 128]);
    assert_eq!(style.background, [191, 191, 255]);

    let linear_style =
        style_for_declarations(&[("background-color", "color-mix(in srgb-linear, red, blue)")]);
    assert_eq!(linear_style.background, [188, 0, 188]);

    let oklab_style =
        style_for_declarations(&[("background-color", "color-mix(in oklab, red, blue)")]);
    assert_eq!(oklab_style.background, [140, 83, 162]);

    let relative_style = style_for_declarations(&[("color", "rgb(from #102030 b g r / 50%)")]);
    assert_eq!(relative_style.color, [48, 32, 16, 128]);

    let hsl_style = style_for_declarations(&[("background-color", "color-mix(in hsl, red, blue)")]);
    assert_eq!(hsl_style.background, [255, 0, 255]);

    let hue_method_style = style_for_declarations(&[(
        "background-color",
        "color-mix(in hsl longer hue, red, blue)",
    )]);
    assert_eq!(hue_method_style.background, [0, 255, 0]);

    let hwb_style = style_for_declarations(&[("background-color", "color-mix(in hwb, red, blue)")]);
    assert_eq!(hwb_style.background, [255, 0, 255]);

    let relative_hsl_style = style_for_declarations(&[("color", "hsl(from red 120deg s l)")]);
    assert_eq!(relative_hsl_style.color, [0, 255, 0, 255]);

    let relative_oklab_style = style_for_declarations(&[("color", "oklab(from red l a b)")]);
    assert_eq!(relative_oklab_style.color, [255, 0, 0, 255]);

    let relative_lab_style = style_for_declarations(&[("color", "lab(from red l a b)")]);
    assert_eq!(relative_lab_style.color, [255, 0, 0, 255]);

    let relative_hwb_style = style_for_declarations(&[("color", "hwb(from red 120deg w b)")]);
    assert_eq!(relative_hwb_style.color, [0, 255, 0, 255]);

    let relative_color_style = style_for_declarations(&[("color", "color(from red srgb 0 1 0)")]);
    assert_eq!(relative_color_style.color, [0, 255, 0, 255]);

    let calculated_relative_color_style =
        style_for_declarations(&[("color", "rgb(from red calc(r - 20) g b)")]);
    assert_eq!(calculated_relative_color_style.color, [235, 0, 0, 255]);

    for (space, expected) in [
        ("display-p3", [128, 10, 145]),
        ("display-p3-linear", [188, 0, 188]),
        ("xyz", [188, 0, 188]),
        ("xyz-d65", [188, 0, 188]),
        ("xyz-d50", [188, 0, 188]),
        ("lab", [193, 0, 136]),
        ("lch", [205, 0, 126]),
        ("oklch", [183, 0, 190]),
    ] {
        let style = style_for_declarations(&[(
            "background-color",
            &format!("color-mix(in {space}, red, blue)"),
        )]);
        assert_eq!(
            style.background, expected,
            "unexpected background in {space}"
        );
    }
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
fn collects_background_layers_without_inheriting_background_color() {
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let parent = ElementIdentity {
        tag: "section".to_owned(),
        class: Some("parent".to_owned()),
        id: None,
    };
    let child = ElementIdentity {
        tag: "span".to_owned(),
        class: Some("child".to_owned()),
        id: None,
    };
    let stylesheet = [
        StyleRule {
            selectors: vec![".parent".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "background-color".to_owned(),
                value: "#ff000080".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".child".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "background-color".to_owned(),
                value: "blue".to_owned(),
            }],
        },
    ];

    let parent_style = resolve_style(&TextStyle::default(), &parent, &[body.clone()], &stylesheet);
    let inherited_child =
        resolve_style(&parent_style, &child, &[body.clone(), parent.clone()], &[]);
    let styled_child = resolve_style(&parent_style, &child, &[body, parent], &stylesheet);

    assert_eq!(inherited_child.background_color, None);
    assert_eq!(inherited_child.background_layers, [[255, 0, 0, 128]]);
    assert_eq!(styled_child.background_color, Some([0, 0, 255, 255]));
    assert_eq!(
        styled_child.background_layers,
        [[255, 0, 0, 128], [0, 0, 255, 255]]
    );
}

#[test]
fn current_color_for_color_uses_the_inherited_color() {
    let rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
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
    assert_eq!(
        style_for_declarations(&[("text-align", "start")]).text_align,
        cosmic_text::Align::Start
    );
    assert_eq!(
        style_for_declarations(&[("text-align-last", "justify")]).text_align_last,
        Some(cosmic_text::Align::JustifiedLast)
    );
    assert_eq!(
        style_for_declarations(&[("text-align-last", "auto")]).text_align_last,
        None
    );
}

#[test]
fn resolves_text_justify_methods_and_legacy_distribute_alias() {
    for (value, expected) in [
        ("auto", cosmic_text::TextJustify::Auto),
        ("none", cosmic_text::TextJustify::None),
        ("inter-word", cosmic_text::TextJustify::InterWord),
        ("inter-character", cosmic_text::TextJustify::InterCharacter),
        ("distribute", cosmic_text::TextJustify::InterCharacter),
    ] {
        assert_eq!(
            style_for_declarations(&[("text-justify", value)]).text_justify,
            expected,
            "{value}"
        );
    }
    assert_eq!(
        style_for_declarations(&[("text-justify", "none"), ("text-justify", "invalid")])
            .text_justify,
        cosmic_text::TextJustify::None
    );
}

#[test]
fn resolves_white_space_and_wrapping_modes() {
    use super::super::style::{OverflowWrap, WhiteSpace, WordBreak};

    for (value, expected) in [
        ("normal", WhiteSpace::Normal),
        ("nowrap", WhiteSpace::NoWrap),
        ("pre", WhiteSpace::Pre),
        ("pre-wrap", WhiteSpace::PreWrap),
        ("pre-line", WhiteSpace::PreLine),
        ("break-spaces", WhiteSpace::BreakSpaces),
    ] {
        assert_eq!(
            style_for_declarations(&[("white-space", value)]).white_space,
            expected,
            "{value}"
        );
    }

    assert_eq!(
        style_for_declarations(&[("overflow-wrap", "break-word")]).wrap_mode(),
        cosmic_text::Wrap::WordOrGlyph
    );
    assert_eq!(
        style_for_declarations(&[("overflow-wrap", "anywhere")]).wrap_mode(),
        cosmic_text::Wrap::WordOrGlyph
    );
    assert_eq!(
        style_for_declarations(&[("word-break", "break-all")]).wrap_mode(),
        cosmic_text::Wrap::Glyph
    );
    assert_eq!(
        style_for_declarations(&[("word-break", "keep-all")]).word_break,
        WordBreak::KeepAll
    );
    assert_eq!(
        style_for_declarations(&[("white-space", "pre")]).wrap_mode(),
        cosmic_text::Wrap::None
    );
    assert_eq!(
        style_for_declarations(&[("white-space", "nowrap")]).wrap_mode(),
        cosmic_text::Wrap::None
    );
    assert_eq!(
        style_for_declarations(&[("overflow-wrap", "invalid")]).overflow_wrap,
        OverflowWrap::Normal
    );
}

#[test]
fn resolves_line_break_modes() {
    use super::super::style::LineBreak;

    for (value, expected) in [
        ("auto", LineBreak::Auto),
        ("loose", LineBreak::Loose),
        ("normal", LineBreak::Normal),
        ("strict", LineBreak::Strict),
        ("anywhere", LineBreak::Anywhere),
    ] {
        let style = style_for_declarations(&[("line-break", value)]);
        assert_eq!(style.line_break, expected, "{value}");
        assert_eq!(
            style.wrap_mode(),
            if value == "anywhere" {
                cosmic_text::Wrap::Glyph
            } else {
                cosmic_text::Wrap::Word
            },
            "{value} wrapping"
        );
    }
}

#[test]
fn resolves_hyphens_modes() {
    use super::super::style::Hyphens;

    for (value, expected) in [
        ("none", Hyphens::None),
        ("manual", Hyphens::Manual),
        ("auto", Hyphens::Auto),
    ] {
        assert_eq!(
            style_for_declarations(&[("hyphens", value)]).hyphens,
            expected,
            "{value}"
        );
    }
}

#[test]
fn resolves_text_overflow_and_line_clamp() {
    use super::super::style::{TextOverflow, WhiteSpace};

    let ellipsis =
        style_for_declarations(&[("white-space", "nowrap"), ("text-overflow", "ellipsis")]);
    assert_eq!(ellipsis.text_overflow, TextOverflow::Ellipsis);
    assert_eq!(
        ellipsis.ellipsize_mode(),
        cosmic_text::Ellipsize::End(cosmic_text::EllipsizeHeightLimit::Lines(1))
    );

    let clamp = style_for_declarations(&[("line-clamp", "3")]);
    assert_eq!(clamp.line_clamp, Some(3));
    assert_eq!(
        clamp.ellipsize_mode(),
        cosmic_text::Ellipsize::End(cosmic_text::EllipsizeHeightLimit::Lines(3))
    );
    assert_eq!(
        style_for_declarations(&[("line-clamp", "0")]).line_clamp,
        None
    );
    let reset = style_for_declarations(&[("line-clamp", "3"), ("line-clamp", "none")]);
    assert_eq!(reset.line_clamp, None);
    assert_eq!(
        style_for_declarations(&[("text-overflow", "ellipsis")]).ellipsize_mode(),
        cosmic_text::Ellipsize::None,
        "text-overflow only truncates nowrap/pre lines"
    );
    assert_eq!(
        style_for_declarations(&[("white-space", "pre")]).white_space,
        WhiteSpace::Pre
    );
}

#[test]
fn resolves_text_rendering_features_and_text_orientation() {
    use super::super::style::{TextOrientation, TextRendering};

    for (value, expected) in [
        ("auto", TextRendering::Auto),
        ("optimizeSpeed", TextRendering::OptimizeSpeed),
        ("optimizeLegibility", TextRendering::OptimizeLegibility),
        ("geometricPrecision", TextRendering::GeometricPrecision),
    ] {
        assert_eq!(
            style_for_declarations(&[("text-rendering", value)]).text_rendering,
            expected,
            "{value}"
        );
    }

    let speed = style_for_declarations(&[("text-rendering", "optimizeSpeed")]);
    let features = speed.font_features();
    assert!(
        features.features.iter().any(|feature| {
            feature.tag == cosmic_text::FeatureTag::KERNING && feature.value == 0
        })
    );
    assert!(features.features.iter().any(|feature| {
        feature.tag == cosmic_text::FeatureTag::STANDARD_LIGATURES && feature.value == 0
    }));

    let explicit_kerning = style_for_declarations(&[
        ("text-rendering", "optimizeSpeed"),
        ("font-kerning", "normal"),
    ]);
    assert!(
        explicit_kerning
            .font_features()
            .features
            .iter()
            .any(|feature| {
                feature.tag == cosmic_text::FeatureTag::KERNING && feature.value == 1
            })
    );

    for (value, expected) in [
        ("mixed", TextOrientation::Mixed),
        ("upright", TextOrientation::Upright),
        ("sideways", TextOrientation::Sideways),
    ] {
        assert_eq!(
            style_for_declarations(&[("text-orientation", value)]).text_orientation,
            expected,
            "{value}"
        );
    }
}

#[test]
fn resolves_custom_css_quote_pairs() {
    let quotes = style_for_declarations(&[("quotes", "\"«\" \"»\" \"‹\" \"›\"")]);
    assert_eq!(
        quotes.quotes,
        [
            ("«".to_owned(), "»".to_owned()),
            ("‹".to_owned(), "›".to_owned())
        ]
    );

    assert!(
        style_for_declarations(&[("quotes", "none")])
            .quotes
            .is_empty()
    );
    assert_eq!(
        style_for_declarations(&[("quotes", "auto")]).quotes,
        [
            ("“".to_owned(), "”".to_owned()),
            ("‘".to_owned(), "’".to_owned())
        ]
    );
    assert_eq!(
        style_for_declarations(&[("quotes", "\"one\" \"two\" \"odd\"")]).quotes,
        [
            ("“".to_owned(), "”".to_owned()),
            ("‘".to_owned(), "’".to_owned())
        ]
    );
}

#[test]
fn resolves_counter_reset_increment_and_set_descriptors() {
    let style = style_for_declarations(&[
        ("counter-reset", "chapter 2 section"),
        ("counter-increment", "chapter -1 section"),
        ("counter-set", "section 4"),
    ]);
    assert_eq!(
        style.counter_reset,
        [("chapter".to_owned(), 2), ("section".to_owned(), 0)]
    );
    assert_eq!(
        style.counter_increment,
        [("chapter".to_owned(), -1), ("section".to_owned(), 1)]
    );
    assert_eq!(style.counter_set, [("section".to_owned(), 4)]);
    assert!(
        style_for_declarations(&[("counter-reset", "none")])
            .counter_reset
            .is_empty()
    );
}

#[test]
fn resolves_text_indent_lengths_percentages_and_flags() {
    let em_indent = style_for_declarations(&[("text-indent", "2em hanging each-line")]);
    assert_eq!(em_indent.text_indent, 32.0);
    assert_eq!(em_indent.text_indent_percent, 0.0);
    assert!(em_indent.text_indent_hanging);
    assert!(em_indent.text_indent_each_line);

    let percent_indent = style_for_declarations(&[("text-indent", "12.5% hanging")]);
    assert_eq!(percent_indent.text_indent, 0.0);
    assert_eq!(percent_indent.text_indent_percent, 12.5);
    assert!(percent_indent.text_indent_hanging);
    assert!(!percent_indent.text_indent_each_line);

    let negative_indent = style_for_declarations(&[("text-indent", "-8px")]);
    assert_eq!(negative_indent.text_indent, -8.0);
    assert_eq!(negative_indent.text_indent_percent, 0.0);
    assert!(!negative_indent.text_indent_hanging);
    assert!(!negative_indent.text_indent_each_line);

    let invalid_after_valid =
        style_for_declarations(&[("text-indent", "10px hanging"), ("text-indent", "auto")]);
    assert_eq!(invalid_after_valid.text_indent, 10.0);
    assert!(invalid_after_valid.text_indent_hanging);
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
fn resolves_optical_sizing_and_variation_settings() {
    let style = style_for_declarations(&[
        ("font-size", "18px"),
        ("font-optical-sizing", "auto"),
        ("font-variation-settings", "\"wght\" 620, \"opsz\" 16.5"),
    ]);
    assert!(style.font_optical_sizing);
    assert_eq!(
        style.font_variation_settings,
        vec![(*b"wght", 620.0), (*b"opsz", 16.5)]
    );
    let attrs = style.to_attrs("sans-serif");
    assert_eq!(attrs.font_variations.len(), 2);
    assert_eq!(attrs.font_variations[0].tag(), *b"wght");
    assert_eq!(attrs.font_variations[1].tag(), *b"opsz");
    assert_eq!(attrs.font_variations[1].value(), 16.5);

    let oblique = style_for_declarations(&[("font-style", "oblique")]).to_attrs("sans-serif");
    assert!(
        oblique
            .font_variations
            .iter()
            .any(|setting| setting.tag() == *b"slnt" && setting.value() == -14.0)
    );
    let italic = style_for_declarations(&[("font-style", "italic")]).to_attrs("sans-serif");
    assert!(
        italic
            .font_variations
            .iter()
            .any(|setting| setting.tag() == *b"ital" && setting.value() == 1.0)
    );
}

#[test]
fn optical_sizing_defaults_to_auto_and_variation_settings_reset() {
    let default = TextStyle::default();
    assert!(default.font_optical_sizing);
    assert!(default.font_variation_settings.is_empty());

    let style = style_for_declarations(&[
        ("font-optical-sizing", "none"),
        ("font-variation-settings", "\"wght\" 500"),
        ("font", "italic 700 20px serif"),
    ]);
    assert!(style.font_optical_sizing);
    assert!(style.font_variation_settings.is_empty());
}

#[test]
fn rejects_invalid_font_variation_settings_without_changing_previous_value() {
    let style = style_for_declarations(&[
        ("font-variation-settings", "\"wght\" 620"),
        ("font-variation-settings", "\"wght\" 620,"),
    ]);
    assert_eq!(style.font_variation_settings, vec![(*b"wght", 620.0)]);
}

#[path = "css_style_decoration_inherit_tests.rs"]
mod decoration_inherit;
