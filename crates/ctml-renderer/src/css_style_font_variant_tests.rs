use crate::style::TextStyle;
use crate::style::{FontVariantCaps, FontVariantPosition};

use super::{ElementIdentity, resolve_style, tests::style_for_declarations};
use crate::{StyleDeclaration, StyleRule};

#[test]
fn resolves_font_variant_ligatures() {
    let style = style_for_declarations(&[(
        "font-variant-ligatures",
        "no-common-ligatures discretionary-ligatures no-contextual",
    )]);

    assert_eq!(style.common_ligatures, Some(false));
    assert_eq!(style.discretionary_ligatures, Some(true));
    assert_eq!(style.contextual_alternates, Some(false));
    assert_eq!(style.historical_ligatures, None);
    let historical = style_for_declarations(&[(
        "font-variant-ligatures",
        "historical-ligatures no-historical-ligatures",
    )]);
    assert_eq!(historical.historical_ligatures, Some(false));
}

#[test]
fn resolves_font_variant_position_values() {
    assert_eq!(
        style_for_declarations(&[("font-variant-position", "sub")]).font_variant_position,
        FontVariantPosition::Sub
    );
    assert_eq!(
        style_for_declarations(&[("font-variant-position", "super")]).font_variant_position,
        FontVariantPosition::Super
    );
}

#[test]
fn resolves_small_caps_variants() {
    let style = style_for_declarations(&[("font-variant-caps", "all-small-caps")]);

    assert_eq!(style.font_variant_caps, FontVariantCaps::AllSmallCaps);
}

#[test]
fn resolves_additional_font_variant_caps_values() {
    for (value, expected) in [
        ("petite-caps", FontVariantCaps::PetiteCaps),
        ("all-petite-caps", FontVariantCaps::AllPetiteCaps),
        ("unicase", FontVariantCaps::Unicase),
        ("titling-caps", FontVariantCaps::TitlingCaps),
    ] {
        assert_eq!(
            style_for_declarations(&[("font-variant-caps", value)]).font_variant_caps,
            expected,
            "font-variant-caps: {value}"
        );
    }
}

#[test]
fn resolves_font_variant_shorthand_and_resets_its_subproperties() {
    let mixed = style_for_declarations(&[(
        "font-variant",
        "small-caps sub no-common-ligatures historical-ligatures",
    )]);
    assert_eq!(mixed.font_variant_caps, FontVariantCaps::SmallCaps);
    assert_eq!(mixed.font_variant_position, FontVariantPosition::Sub);
    assert_eq!(mixed.common_ligatures, Some(false));
    assert_eq!(mixed.historical_ligatures, Some(true));

    let reset_to_defaults = style_for_declarations(&[
        ("font-variant-position", "super"),
        ("font-variant-ligatures", "none"),
        ("font-variant", "small-caps"),
    ]);
    assert_eq!(
        reset_to_defaults.font_variant_position,
        FontVariantPosition::Normal
    );
    assert_eq!(reset_to_defaults.common_ligatures, None);
    assert_eq!(reset_to_defaults.discretionary_ligatures, None);

    let reset = style_for_declarations(&[
        ("font-variant-caps", "petite-caps"),
        ("font-variant-position", "super"),
        ("font-variant-ligatures", "none"),
        ("font-variant", "normal"),
    ]);
    assert_eq!(reset.font_variant_caps, FontVariantCaps::Normal);
    assert_eq!(reset.font_variant_position, FontVariantPosition::Normal);
    assert_eq!(reset.common_ligatures, None);
    assert_eq!(reset.discretionary_ligatures, None);
    assert_eq!(reset.historical_ligatures, None);
    assert_eq!(reset.contextual_alternates, None);

    let ignored_invalid = style_for_declarations(&[
        ("font-variant-caps", "petite-caps"),
        ("font-variant", "small-caps unsupported-value"),
    ]);
    assert_eq!(
        ignored_invalid.font_variant_caps,
        FontVariantCaps::PetiteCaps
    );
}

#[test]
fn resolves_font_variant_numeric_features_and_rejects_conflicting_groups() {
    let style = style_for_declarations(&[(
        "font-variant-numeric",
        "lining-nums tabular-nums diagonal-fractions ordinal slashed-zero",
    )]);
    assert_eq!(
        style.font_variant_numeric,
        [
            "lining-nums",
            "tabular-nums",
            "diagonal-fractions",
            "ordinal",
            "slashed-zero"
        ]
    );

    let reset = style_for_declarations(&[
        ("font-variant-numeric", "oldstyle-nums proportional-nums"),
        ("font-variant-numeric", "normal"),
    ]);
    assert!(reset.font_variant_numeric.is_empty());

    let invalid = style_for_declarations(&[
        ("font-variant-numeric", "tabular-nums"),
        ("font-variant-numeric", "lining-nums oldstyle-nums"),
    ]);
    assert_eq!(invalid.font_variant_numeric, ["tabular-nums"]);
}

#[test]
fn font_variant_shorthand_sets_and_resets_numeric_features() {
    let style = style_for_declarations(&[("font-variant", "lining-nums ordinal small-caps")]);
    assert_eq!(style.font_variant_caps, FontVariantCaps::SmallCaps);
    assert_eq!(style.font_variant_numeric, ["lining-nums", "ordinal"]);

    let reset = style_for_declarations(&[
        ("font-variant", "tabular-nums slashed-zero"),
        ("font-variant", "normal"),
    ]);
    assert!(reset.font_variant_numeric.is_empty());
}

#[test]
fn font_variant_numeric_inherits_and_unset_uses_the_inherited_value() {
    let inherited = TextStyle {
        font_variant_numeric: vec!["oldstyle-nums".to_owned(), "ordinal".to_owned()],
        font_variant_east_asian: vec!["jis04".to_owned(), "ruby".to_owned()],
        font_feature_settings: vec![(*b"sinf", 1)],
        ..TextStyle::default()
    };
    let rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "font-variant-numeric".to_owned(),
                value: "unset".to_owned(),
            },
            StyleDeclaration {
                property: "font-variant-east-asian".to_owned(),
                value: "unset".to_owned(),
            },
            StyleDeclaration {
                property: "font-feature-settings".to_owned(),
                value: "unset".to_owned(),
            },
        ],
    };
    let current = ElementIdentity {
        tag: "span".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };

    let style = resolve_style(&inherited, &current, &[], &[rule]);

    assert_eq!(style.font_variant_numeric, inherited.font_variant_numeric);
    assert_eq!(
        style.font_variant_east_asian,
        inherited.font_variant_east_asian
    );
    assert_eq!(style.font_feature_settings, inherited.font_feature_settings);
}

#[test]
fn resolves_font_variant_east_asian_features_and_conflicts() {
    let style = style_for_declarations(&[(
        "font-variant-east-asian",
        "jis83 simplified proportional-width ruby",
    )]);
    assert_eq!(
        style.font_variant_east_asian,
        ["jis83", "simplified", "proportional-width", "ruby"]
    );

    let invalid = style_for_declarations(&[
        ("font-variant-east-asian", "jis90 ruby"),
        ("font-variant-east-asian", "jis78 jis04"),
    ]);
    assert_eq!(invalid.font_variant_east_asian, ["jis90", "ruby"]);
}

#[test]
fn font_variant_shorthand_sets_and_resets_east_asian_features() {
    let style = style_for_declarations(&[("font-variant", "jis90 traditional full-width ruby")]);
    assert_eq!(
        style.font_variant_east_asian,
        ["jis90", "traditional", "full-width", "ruby"]
    );

    let reset = style_for_declarations(&[
        ("font-variant", "jis83 simplified proportional-width"),
        ("font-variant", "normal"),
    ]);
    assert!(reset.font_variant_east_asian.is_empty());
}

#[test]
fn parses_font_feature_settings_and_ignores_invalid_values() {
    let style = style_for_declarations(&[(
        "font-feature-settings",
        "\"sinf\" 1, \"liga\" off, \"ABCD\"",
    )]);
    assert_eq!(
        style.font_feature_settings,
        [(*b"sinf", 1), (*b"liga", 0), (*b"ABCD", 1)]
    );

    let invalid = style_for_declarations(&[
        ("font-feature-settings", "\"sinf\" 1"),
        ("font-feature-settings", "\"abc\" on"),
    ]);
    assert_eq!(invalid.font_feature_settings, [(*b"sinf", 1)]);

    let reset = style_for_declarations(&[
        ("font-feature-settings", "\"sinf\" on"),
        ("font-feature-settings", "normal"),
    ]);
    assert!(reset.font_feature_settings.is_empty());
}

#[test]
fn font_shorthand_resets_font_feature_settings() {
    let style = style_for_declarations(&[
        ("font-feature-settings", "\"sinf\" 1"),
        ("font", "italic 18px sans-serif"),
    ]);
    assert!(style.font_feature_settings.is_empty());
}
