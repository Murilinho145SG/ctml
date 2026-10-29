use crate::style::{FontVariantCaps, FontVariantPosition};

use super::tests::style_for_declarations;

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
