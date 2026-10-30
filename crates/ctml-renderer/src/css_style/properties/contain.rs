use super::*;

pub(super) fn apply_contain(style: &mut TextStyle, value: &str) {
    if let Some((paint_contained, size_contained, inline_size_contained)) = parse_contain(value) {
        style.paint_contained = paint_contained;
        style.size_contained = size_contained;
        style.inline_size_contained = inline_size_contained;
    }
}

fn parse_contain(value: &str) -> Option<(bool, bool, bool)> {
    let values = value
        .split_ascii_whitespace()
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }
    if values.len() == 1 {
        return match values[0].as_str() {
            "none" | "layout" | "style" => Some((false, false, false)),
            "inline-size" => Some((false, false, true)),
            "size" => Some((false, true, true)),
            "paint" | "content" => Some((true, false, false)),
            "strict" => Some((true, true, true)),
            _ => None,
        };
    }
    if values
        .iter()
        .any(|value| matches!(value.as_str(), "none" | "content" | "strict"))
    {
        return None;
    }
    let mut size_kind = false;
    let mut unique = std::collections::HashSet::new();
    for value in &values {
        if !matches!(
            value.as_str(),
            "size" | "inline-size" | "layout" | "style" | "paint"
        ) || !unique.insert(value)
        {
            return None;
        }
        if matches!(value.as_str(), "size" | "inline-size") {
            if size_kind {
                return None;
            }
            size_kind = true;
        }
    }
    let contains_size = values.iter().any(|value| value == "size");
    Some((
        values.iter().any(|value| value == "paint"),
        contains_size,
        contains_size || values.iter().any(|value| value == "inline-size"),
    ))
}
