pub(super) const SUPPORTED_ALL_PROPERTIES: &[&str] = &[
    "color",
    "background-color",
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
    "line-height",
    "letter-spacing",
    "text-transform",
    "text-align",
    "text-decoration",
    "text-decoration-line",
    "text-decoration-style",
    "text-decoration-color",
    "text-decoration-skip-ink",
    "text-decoration-thickness",
    "text-underline-offset",
    "cursor",
];

pub(super) fn is_all_keyword(value: &str) -> bool {
    let value = value.trim();
    ["inherit", "initial", "unset", "revert", "revert-layer"]
        .iter()
        .any(|keyword| value.eq_ignore_ascii_case(keyword))
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
