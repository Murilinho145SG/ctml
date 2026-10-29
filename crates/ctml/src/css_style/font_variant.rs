use crate::style::{FontVariantCaps, FontVariantPosition, TextStyle};

pub(super) fn apply_font_variant_ligatures(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("normal") {
        style.common_ligatures = None;
        style.discretionary_ligatures = None;
        style.historical_ligatures = None;
        style.contextual_alternates = None;
        return;
    }
    if value.eq_ignore_ascii_case("none") {
        style.common_ligatures = Some(false);
        style.discretionary_ligatures = Some(false);
        style.historical_ligatures = Some(false);
        style.contextual_alternates = Some(false);
        return;
    }
    for feature in value.split_ascii_whitespace() {
        match feature.to_ascii_lowercase().as_str() {
            "common-ligatures" => style.common_ligatures = Some(true),
            "no-common-ligatures" => style.common_ligatures = Some(false),
            "discretionary-ligatures" => style.discretionary_ligatures = Some(true),
            "no-discretionary-ligatures" => style.discretionary_ligatures = Some(false),
            "historical-ligatures" => style.historical_ligatures = Some(true),
            "no-historical-ligatures" => style.historical_ligatures = Some(false),
            "contextual" => style.contextual_alternates = Some(true),
            "no-contextual" => style.contextual_alternates = Some(false),
            _ => {}
        }
    }
}

pub(super) fn apply_font_variant(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("normal") {
        reset_font_variant(style);
        return;
    }
    let tokens = value.split_ascii_whitespace().collect::<Vec<_>>();
    if tokens.is_empty() || tokens.iter().any(|token| !is_font_variant_token(token)) {
        return;
    }

    reset_font_variant(style);
    for token in tokens {
        apply_font_variant_caps(style, token);
        match token.to_ascii_lowercase().as_str() {
            "sub" => style.font_variant_position = FontVariantPosition::Sub,
            "super" => style.font_variant_position = FontVariantPosition::Super,
            _ => {}
        }
    }
    apply_font_variant_ligatures(style, value);
}

fn reset_font_variant(style: &mut TextStyle) {
    style.font_variant_caps = FontVariantCaps::Normal;
    style.font_variant_position = FontVariantPosition::Normal;
    style.common_ligatures = None;
    style.discretionary_ligatures = None;
    style.historical_ligatures = None;
    style.contextual_alternates = None;
}

fn is_font_variant_token(token: &str) -> bool {
    matches!(
        token.to_ascii_lowercase().as_str(),
        "small-caps"
            | "all-small-caps"
            | "petite-caps"
            | "all-petite-caps"
            | "unicase"
            | "titling-caps"
            | "sub"
            | "super"
            | "common-ligatures"
            | "no-common-ligatures"
            | "discretionary-ligatures"
            | "no-discretionary-ligatures"
            | "historical-ligatures"
            | "no-historical-ligatures"
            | "contextual"
            | "no-contextual"
            | "none"
    )
}

pub(super) fn apply_font_variant_caps(style: &mut TextStyle, value: &str) {
    style.font_variant_caps = match value.to_ascii_lowercase().as_str() {
        "normal" => FontVariantCaps::Normal,
        "small-caps" => FontVariantCaps::SmallCaps,
        "all-small-caps" => FontVariantCaps::AllSmallCaps,
        "petite-caps" => FontVariantCaps::PetiteCaps,
        "all-petite-caps" => FontVariantCaps::AllPetiteCaps,
        "unicase" => FontVariantCaps::Unicase,
        "titling-caps" => FontVariantCaps::TitlingCaps,
        _ => style.font_variant_caps,
    };
}
