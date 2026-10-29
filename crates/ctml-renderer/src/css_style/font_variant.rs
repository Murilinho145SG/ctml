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
    let numeric = tokens
        .iter()
        .copied()
        .filter(|token| is_font_variant_numeric_token(token))
        .collect::<Vec<_>>();
    let numeric_values = if numeric.is_empty() {
        Some(Vec::new())
    } else {
        parse_font_variant_numeric(&numeric.join(" "))
    };
    let Some(numeric_values) = numeric_values else {
        return;
    };
    let east_asian = tokens
        .iter()
        .copied()
        .filter(|token| is_font_variant_east_asian_token(token))
        .collect::<Vec<_>>();
    let east_asian_values = if east_asian.is_empty() {
        Some(Vec::new())
    } else {
        parse_font_variant_east_asian(&east_asian.join(" "))
    };
    let Some(east_asian_values) = east_asian_values else {
        return;
    };

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
    style.font_variant_numeric = numeric_values;
    style.font_variant_east_asian = east_asian_values;
}

fn reset_font_variant(style: &mut TextStyle) {
    style.font_variant_caps = FontVariantCaps::Normal;
    style.font_variant_position = FontVariantPosition::Normal;
    style.common_ligatures = None;
    style.discretionary_ligatures = None;
    style.historical_ligatures = None;
    style.contextual_alternates = None;
    style.font_variant_numeric.clear();
    style.font_variant_east_asian.clear();
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
            | "lining-nums"
            | "oldstyle-nums"
            | "proportional-nums"
            | "tabular-nums"
            | "diagonal-fractions"
            | "stacked-fractions"
            | "ordinal"
            | "slashed-zero"
            | "jis78"
            | "jis83"
            | "jis90"
            | "jis04"
            | "simplified"
            | "traditional"
            | "full-width"
            | "proportional-width"
            | "ruby"
            | "none"
    )
}

pub(super) fn apply_font_variant_numeric(style: &mut TextStyle, value: &str) {
    if let Some(values) = parse_font_variant_numeric(value) {
        style.font_variant_numeric = values;
    }
}

fn parse_font_variant_numeric(value: &str) -> Option<Vec<String>> {
    if value.eq_ignore_ascii_case("normal") {
        return Some(Vec::new());
    }
    let mut groups = [false; 5];
    let mut values = Vec::new();
    for token in value.split_ascii_whitespace() {
        let group = match token.to_ascii_lowercase().as_str() {
            "lining-nums" | "oldstyle-nums" => 0,
            "proportional-nums" | "tabular-nums" => 1,
            "diagonal-fractions" | "stacked-fractions" => 2,
            "ordinal" => 3,
            "slashed-zero" => 4,
            _ => return None,
        };
        if groups[group] {
            return None;
        }
        groups[group] = true;
        values.push(token.to_ascii_lowercase());
    }
    (!values.is_empty()).then_some(values)
}

fn is_font_variant_numeric_token(token: &str) -> bool {
    matches!(
        token.to_ascii_lowercase().as_str(),
        "lining-nums"
            | "oldstyle-nums"
            | "proportional-nums"
            | "tabular-nums"
            | "diagonal-fractions"
            | "stacked-fractions"
            | "ordinal"
            | "slashed-zero"
    )
}

pub(super) fn apply_font_variant_east_asian(style: &mut TextStyle, value: &str) {
    if let Some(values) = parse_font_variant_east_asian(value) {
        style.font_variant_east_asian = values;
    }
}

fn parse_font_variant_east_asian(value: &str) -> Option<Vec<String>> {
    if value.eq_ignore_ascii_case("normal") {
        return Some(Vec::new());
    }
    let mut groups = [false; 4];
    let mut values = Vec::new();
    for token in value.split_ascii_whitespace() {
        let group = match token.to_ascii_lowercase().as_str() {
            "jis78" | "jis83" | "jis90" | "jis04" => 0,
            "simplified" | "traditional" => 1,
            "full-width" | "proportional-width" => 2,
            "ruby" => 3,
            _ => return None,
        };
        if groups[group] {
            return None;
        }
        groups[group] = true;
        values.push(token.to_ascii_lowercase());
    }
    (!values.is_empty()).then_some(values)
}

fn is_font_variant_east_asian_token(token: &str) -> bool {
    matches!(
        token.to_ascii_lowercase().as_str(),
        "jis78"
            | "jis83"
            | "jis90"
            | "jis04"
            | "simplified"
            | "traditional"
            | "full-width"
            | "proportional-width"
            | "ruby"
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
