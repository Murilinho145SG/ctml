use super::*;

pub(super) fn apply_contain(style: &mut TextStyle, value: &str) {
    if let Some(paint_contained) = parse_contain(value) {
        style.paint_contained = paint_contained;
    }
}

fn parse_contain(value: &str) -> Option<bool> {
    let values = value
        .split_ascii_whitespace()
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }
    if values.len() == 1 {
        return match values[0].as_str() {
            "none" | "size" | "inline-size" | "layout" | "style" => Some(false),
            "paint" | "content" | "strict" => Some(true),
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
    Some(values.iter().any(|value| value == "paint"))
}
