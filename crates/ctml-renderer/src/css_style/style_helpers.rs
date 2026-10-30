use std::collections::HashMap;

pub(super) const SUPPORTED_ALL_PROPERTIES: &[&str] = &[
    "color",
    "opacity",
    "visibility",
    "content-visibility",
    "contain",
    "contain-intrinsic-size",
    "contain-intrinsic-width",
    "contain-intrinsic-height",
    "overflow",
    "overflow-x",
    "overflow-y",
    "display",
    "list-style-type",
    "width",
    "inline-size",
    "min-width",
    "min-inline-size",
    "max-width",
    "max-inline-size",
    "box-sizing",
    "border-radius",
    "border-top-left-radius",
    "border-top-right-radius",
    "border-bottom-right-radius",
    "border-bottom-left-radius",
    "border-image-source",
    "border-image-slice",
    "border-image-repeat",
    "border-image-width",
    "outline",
    "outline-width",
    "outline-style",
    "outline-color",
    "outline-offset",
    "aspect-ratio",
    "box-shadow",
    "box-decoration-break",
    "border",
    "border-top",
    "border-right",
    "border-bottom",
    "border-left",
    "border-inline",
    "border-inline-start",
    "border-inline-end",
    "border-block",
    "border-block-start",
    "border-block-end",
    "border-width",
    "border-top-width",
    "border-right-width",
    "border-bottom-width",
    "border-left-width",
    "border-inline-width",
    "border-inline-start-width",
    "border-inline-end-width",
    "border-block-width",
    "border-block-start-width",
    "border-block-end-width",
    "border-style",
    "border-top-style",
    "border-right-style",
    "border-bottom-style",
    "border-left-style",
    "border-inline-style",
    "border-inline-start-style",
    "border-inline-end-style",
    "border-block-style",
    "border-block-start-style",
    "border-block-end-style",
    "border-color",
    "border-top-color",
    "border-right-color",
    "border-bottom-color",
    "border-left-color",
    "border-inline-color",
    "border-inline-start-color",
    "border-inline-end-color",
    "border-block-color",
    "border-block-start-color",
    "border-block-end-color",
    "padding",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "padding-inline",
    "padding-inline-start",
    "padding-inline-end",
    "padding-block",
    "padding-block-start",
    "padding-block-end",
    "margin",
    "margin-top",
    "margin-right",
    "margin-bottom",
    "margin-left",
    "margin-inline",
    "margin-inline-start",
    "margin-inline-end",
    "margin-block",
    "margin-block-start",
    "margin-block-end",
    "height",
    "block-size",
    "min-height",
    "min-block-size",
    "max-height",
    "max-block-size",
    "background-color",
    "background-image",
    "background-repeat",
    "background-position",
    "background-size",
    "background-attachment",
    "background-origin",
    "background-clip",
    "font",
    "font-family",
    "font-size",
    "font-weight",
    "font-style",
    "font-stretch",
    "font-kerning",
    "font-variant",
    "font-variant-caps",
    "font-variant-ligatures",
    "font-variant-position",
    "font-variant-numeric",
    "font-variant-east-asian",
    "font-feature-settings",
    "font-optical-sizing",
    "font-variation-settings",
    "line-height",
    "vertical-align",
    "letter-spacing",
    "word-spacing",
    "text-transform",
    "text-align",
    "text-align-last",
    "text-justify",
    "text-indent",
    "white-space",
    "text-wrap",
    "text-wrap-mode",
    "overflow-wrap",
    "word-break",
    "line-break",
    "hyphens",
    "text-overflow",
    "line-clamp",
    "text-rendering",
    "text-orientation",
    "writing-mode",
    "direction",
    "unicode-bidi",
    "quotes",
    "text-decoration",
    "text-decoration-line",
    "text-decoration-style",
    "text-decoration-color",
    "text-decoration-skip-ink",
    "text-decoration-thickness",
    "text-underline-offset",
    "text-shadow",
    "text-emphasis",
    "text-emphasis-style",
    "text-emphasis-color",
    "text-emphasis-position",
    "cursor",
    "pointer-events",
    "user-select",
];

pub(super) fn is_all_keyword(value: &str) -> bool {
    let value = value.trim();
    ["inherit", "initial", "unset", "revert", "revert-layer"]
        .iter()
        .any(|keyword| value.eq_ignore_ascii_case(keyword))
}

pub(super) fn is_custom_property(property: &str) -> bool {
    property.starts_with("--")
}

pub(super) fn canonical_property_name(property: &str) -> String {
    if is_custom_property(property) {
        property.to_owned()
    } else {
        property.to_ascii_lowercase()
    }
}

pub(super) fn property_names_equal(left: &str, right: &str) -> bool {
    if is_custom_property(left) || is_custom_property(right) {
        left == right
    } else {
        left.eq_ignore_ascii_case(right)
    }
}

pub(super) fn layer_cascade_priority(layer: Option<usize>, important: bool) -> usize {
    match (important, layer) {
        (false, Some(layer)) => layer,
        (false, None) => usize::MAX,
        (true, Some(layer)) => usize::MAX - layer,
        (true, None) => 0,
    }
}

pub(super) fn declaration_value(value: &str) -> (&str, bool) {
    let value = value.trim_end();
    let Some(value_without_priority) = value.get(..value.len().saturating_sub("!important".len()))
    else {
        return (value, false);
    };
    let priority = value[value_without_priority.len()..].trim();
    if priority.eq_ignore_ascii_case("!important") {
        (value_without_priority.trim_end(), true)
    } else {
        (value, false)
    }
}

pub(super) fn reverted_layers(
    matches: &[super::MatchedDeclaration<'_>],
    property: &str,
) -> Vec<(bool, Option<usize>)> {
    let mut reverted = Vec::new();
    for (important, _, layer, _, _, _, declaration) in matches
        .iter()
        .rev()
        .filter(|entry| property_names_equal(&entry.6.property, property))
    {
        let layer_key = (*important, *layer);
        if reverted.contains(&layer_key) {
            continue;
        }
        let (value, _) = declaration_value(&declaration.value);
        if value.eq_ignore_ascii_case("revert-layer") {
            reverted.push(layer_key);
        } else {
            break;
        }
    }
    reverted
}

pub(super) fn cascade_custom_properties(
    inherited: &HashMap<String, String>,
    matches: &[super::MatchedDeclaration<'_>],
) -> HashMap<String, String> {
    let mut raw_properties = inherited.clone();
    for (important, _, layer, _, _, _, declaration) in matches
        .iter()
        .filter(|entry| is_custom_property(&entry.6.property))
    {
        let property = canonical_property_name(&declaration.property);
        if reverted_layers(matches, &property).contains(&(*important, *layer)) {
            continue;
        }
        let (value, _) = declaration_value(&declaration.value);
        match value.trim().to_ascii_lowercase().as_str() {
            "inherit" | "unset" => {
                if let Some(inherited) = inherited.get(&property) {
                    raw_properties.insert(property, inherited.clone());
                } else {
                    raw_properties.remove(&property);
                }
            }
            "initial" | "revert" => {
                raw_properties.remove(&property);
            }
            _ => {
                raw_properties.insert(property, value.to_owned());
            }
        }
    }
    super::variables::resolve_custom_properties(&raw_properties)
}
