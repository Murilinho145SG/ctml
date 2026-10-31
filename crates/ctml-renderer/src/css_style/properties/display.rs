use crate::style::{FlexBasis, TextStyle};

pub(super) fn apply_flex_direction(style: &mut TextStyle, value: &str) {
    match value.trim().to_ascii_lowercase().as_str() {
        "row" => {
            style.flex_direction_column = false;
            style.flex_direction_reverse = false;
        }
        "row-reverse" => {
            style.flex_direction_column = false;
            style.flex_direction_reverse = true;
        }
        "column" => {
            style.flex_direction_column = true;
            style.flex_direction_reverse = false;
        }
        "column-reverse" => {
            style.flex_direction_column = true;
            style.flex_direction_reverse = true;
        }
        _ => {}
    }
}

pub(super) fn apply_flex_wrap(style: &mut TextStyle, value: &str) {
    match value.trim().to_ascii_lowercase().as_str() {
        "nowrap" => style.flex_wrap = false,
        "wrap" => style.flex_wrap = true,
        _ => {}
    }
}

pub(super) fn apply_flex_flow(style: &mut TextStyle, value: &str) {
    let mut direction = None;
    let mut wrap = None;
    for token in value.split_ascii_whitespace() {
        match token.to_ascii_lowercase().as_str() {
            "row" | "row-reverse" | "column" | "column-reverse" if direction.is_none() => {
                direction = Some(token.to_ascii_lowercase());
            }
            "wrap" | "nowrap" if wrap.is_none() => {
                wrap = Some(token.to_ascii_lowercase());
            }
            _ => return,
        }
    }
    if direction.is_none() && wrap.is_none() {
        return;
    }

    apply_flex_direction(style, direction.as_deref().unwrap_or("row"));
    apply_flex_wrap(style, wrap.as_deref().unwrap_or("nowrap"));
}

pub(super) fn apply_justify_content(style: &mut TextStyle, value: &str) {
    style.justify_content = match value.trim().to_ascii_lowercase().as_str() {
        "normal" | "flex-start" => crate::style::FlexAlignment::FlexStart,
        "start" => crate::style::FlexAlignment::Start,
        "center" => crate::style::FlexAlignment::Center,
        "flex-end" => crate::style::FlexAlignment::FlexEnd,
        "end" => crate::style::FlexAlignment::End,
        "space-between" => crate::style::FlexAlignment::SpaceBetween,
        "space-around" => crate::style::FlexAlignment::SpaceAround,
        "space-evenly" => crate::style::FlexAlignment::SpaceEvenly,
        _ => return,
    };
}

pub(super) fn apply_align_items(style: &mut TextStyle, value: &str) {
    style.align_items = match value.trim().to_ascii_lowercase().as_str() {
        "normal" | "stretch" => Some(crate::style::FlexAlignment::Stretch),
        "flex-start" => Some(crate::style::FlexAlignment::FlexStart),
        "start" => Some(crate::style::FlexAlignment::Start),
        "center" => Some(crate::style::FlexAlignment::Center),
        "flex-end" => Some(crate::style::FlexAlignment::FlexEnd),
        "end" => Some(crate::style::FlexAlignment::End),
        _ => return,
    };
}

pub(super) fn apply_align_content(style: &mut TextStyle, value: &str) {
    style.align_content = match value.trim().to_ascii_lowercase().as_str() {
        "normal" | "stretch" => Some(crate::style::FlexAlignment::Stretch),
        "flex-start" => Some(crate::style::FlexAlignment::FlexStart),
        "start" => Some(crate::style::FlexAlignment::Start),
        "center" => Some(crate::style::FlexAlignment::Center),
        "flex-end" => Some(crate::style::FlexAlignment::FlexEnd),
        "end" => Some(crate::style::FlexAlignment::End),
        "space-between" => Some(crate::style::FlexAlignment::SpaceBetween),
        "space-around" => Some(crate::style::FlexAlignment::SpaceAround),
        "space-evenly" => Some(crate::style::FlexAlignment::SpaceEvenly),
        _ => return,
    };
}

pub(super) fn apply_align_self(style: &mut TextStyle, value: &str) {
    style.align_self = match value.trim().to_ascii_lowercase().as_str() {
        "auto" => None,
        "normal" | "stretch" => Some(crate::style::FlexAlignment::Stretch),
        "flex-start" => Some(crate::style::FlexAlignment::FlexStart),
        "start" => Some(crate::style::FlexAlignment::Start),
        "center" => Some(crate::style::FlexAlignment::Center),
        "flex-end" => Some(crate::style::FlexAlignment::FlexEnd),
        "end" => Some(crate::style::FlexAlignment::End),
        _ => return,
    };
}

pub(super) fn apply_flex_order(style: &mut TextStyle, value: &str) {
    if let Some(order) = ctml_css::parse_css_integer(value) {
        style.flex_order = order.parse().unwrap_or_else(|_| {
            if order.starts_with('-') {
                i32::MIN
            } else {
                i32::MAX
            }
        });
    }
}

pub(super) fn apply_flex_grow(style: &mut TextStyle, value: &str) {
    if let Some(grow) = value
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|grow| grow.is_finite() && *grow >= 0.0)
    {
        style.flex_grow = grow;
    }
}

pub(super) fn apply_flex_shrink(style: &mut TextStyle, value: &str) {
    if let Some(shrink) = value
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|shrink| shrink.is_finite() && *shrink >= 0.0)
    {
        style.flex_shrink = shrink;
    }
}

pub(super) fn apply_flex_basis(style: &mut TextStyle, value: &str) {
    let value = value.trim();
    let basis = if value.eq_ignore_ascii_case("auto") {
        Some(FlexBasis::Auto)
    } else if value.eq_ignore_ascii_case("content") {
        Some(FlexBasis::Content)
    } else if value.eq_ignore_ascii_case("max-content") {
        Some(FlexBasis::MaxContent)
    } else if value.eq_ignore_ascii_case("min-content") {
        Some(FlexBasis::MinContent)
    } else if value.eq_ignore_ascii_case("fit-content") {
        Some(FlexBasis::FitContent)
    } else if let Some(percentage) = ctml_css::parse_css_percentage(value) {
        (percentage >= 0.0).then_some(FlexBasis::Percentage(percentage / 100.0))
    } else if ctml_css::parse_math_expression(value, |component| {
        ctml_css::parse_css_percentage(component)
            .map(|percentage| percentage / 100.0)
            .or_else(|| {
                ctml_css::parse_nonnegative_font_relative_length_with_viewport(
                    component,
                    style.font_size,
                    style.line_height,
                    style.font_relative_units,
                    style.viewport_size,
                )
            })
    })
    .is_some()
    {
        Some(FlexBasis::Calculation(value.to_ascii_lowercase()))
    } else {
        ctml_css::parse_nonnegative_font_relative_length_with_viewport(
            value,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
        .filter(|length| length.is_finite())
        .map(FlexBasis::Length)
    };
    if let Some(basis) = basis {
        style.flex_basis = Some(basis);
    }
}

pub(super) fn apply_layout_property(style: &mut TextStyle, property: &str, value: &str) {
    match property {
        "display" => apply_display(style, value),
        "flex-direction" => apply_flex_direction(style, value),
        "flex-wrap" => apply_flex_wrap(style, value),
        "flex-flow" => apply_flex_flow(style, value),
        "justify-content" => apply_justify_content(style, value),
        "align-content" => apply_align_content(style, value),
        "align-items" => apply_align_items(style, value),
        "align-self" => apply_align_self(style, value),
        "order" => apply_flex_order(style, value),
        "flex-grow" => apply_flex_grow(style, value),
        "flex-shrink" => apply_flex_shrink(style, value),
        "flex-basis" => apply_flex_basis(style, value),
        _ => super::flex_gap::apply_flex_gap(style, property, value),
    }
}

pub(super) fn is_layout_property(property: &str) -> bool {
    matches!(
        property,
        "display"
            | "flex-direction"
            | "flex-wrap"
            | "flex-flow"
            | "justify-content"
            | "align-content"
            | "align-items"
            | "align-self"
            | "order"
            | "flex-grow"
            | "flex-shrink"
            | "flex-basis"
            | "gap"
            | "row-gap"
            | "column-gap"
    )
}

pub(super) fn apply_display(style: &mut TextStyle, value: &str) {
    let value = value
        .split_ascii_whitespace()
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    let value = value.iter().map(String::as_str).collect::<Vec<_>>();
    if value == ["run-in"] {
        style.display_none = false;
        style.display_block = None;
        style.display_list_item = Some(false);
        style.display_run_in = true;
        style.display_flex = false;
        return;
    }
    if let Some(is_block) = parse_flex_display(&value) {
        style.display_none = false;
        style.display_block = Some(is_block);
        style.display_list_item = Some(false);
        style.display_run_in = false;
        style.display_flex = true;
        return;
    }
    if let Some(is_block) = parse_list_item(&value) {
        style.display_none = false;
        style.display_block = Some(is_block);
        style.display_list_item = Some(true);
        style.display_run_in = false;
        style.display_flex = false;
        return;
    }
    match parse_flow_display(&value) {
        Some((none, block)) => {
            style.display_none = none;
            style.display_block = block;
            style.display_list_item = Some(false);
            style.display_run_in = false;
            style.display_flex = false;
        }
        None => {}
    }
}

fn parse_flex_display(tokens: &[&str]) -> Option<bool> {
    match tokens {
        ["flex"] | ["block", "flex"] | ["flex", "block"] => Some(true),
        ["inline-flex"] | ["inline", "flex"] | ["flex", "inline"] => Some(false),
        _ => None,
    }
}

fn parse_list_item(tokens: &[&str]) -> Option<bool> {
    let mut remaining = tokens.to_vec();
    let item_position = remaining.iter().position(|token| *token == "list-item")?;
    remaining.remove(item_position);
    if remaining.len() > 2 {
        return None;
    }

    let mut outside = None;
    let mut inside = None;
    for token in remaining {
        match token {
            "block" | "inline" if outside.is_none() => outside = Some(token),
            "flow" | "flow-root" if inside.is_none() => inside = Some(token),
            _ => return None,
        }
    }
    Some(outside.unwrap_or("block") == "block")
}

fn parse_flow_display(tokens: &[&str]) -> Option<(bool, Option<bool>)> {
    match tokens {
        ["none"] => Some((true, None)),
        ["contents"] => Some((false, Some(false))),
        ["flow-root"] => Some((false, Some(true))),
        ["inline-block"] => Some((false, Some(false))),
        ["block"]
        | ["block", "flow"]
        | ["flow", "block"]
        | ["block", "flow-root"]
        | ["flow-root", "block"] => Some((false, Some(true))),
        ["inline"]
        | ["inline", "flow"]
        | ["flow", "inline"]
        | ["inline", "flow-root"]
        | ["flow-root", "inline"] => Some((false, Some(false))),
        _ => None,
    }
}

pub(super) fn inherit_display(style: &mut TextStyle, inherited: &TextStyle) {
    style.display_none = inherited.display_none;
    style.display_block = inherited.display_block;
    style.display_list_item = inherited.display_list_item;
    style.display_run_in = inherited.display_run_in;
    style.display_flex = inherited.display_flex;
}

pub(super) fn reset_display(style: &mut TextStyle) {
    style.display_none = false;
    style.display_block = Some(false);
    style.display_list_item = Some(false);
    style.display_run_in = false;
    style.display_flex = false;
}

pub(super) fn revert_display(style: &mut TextStyle) {
    style.display_none = false;
    style.display_block = None;
    style.display_list_item = None;
    style.display_run_in = false;
    style.display_flex = false;
}

#[cfg(test)]
mod tests {
    use super::{parse_flex_display, parse_flow_display, parse_list_item};

    #[test]
    fn parses_flex_display_outer_forms_in_any_order() {
        for tokens in [
            &(["flex"][..]),
            &(["block", "flex"][..]),
            &(["flex", "block"][..]),
        ] {
            assert_eq!(parse_flex_display(tokens), Some(true));
        }
        for tokens in [
            &(["inline-flex"][..]),
            &(["inline", "flex"][..]),
            &(["flex", "inline"][..]),
        ] {
            assert_eq!(parse_flex_display(tokens), Some(false));
        }
        assert_eq!(parse_flex_display(&["grid"]), None);
        assert_eq!(parse_flex_display(&["inline", "block", "flex"]), None);
    }

    #[test]
    fn parses_list_item_outer_and_inner_values_in_any_order() {
        for tokens in [
            &["list-item"][..],
            &["inline", "list-item"],
            &["flow-root", "block", "list-item"],
            &["list-item", "flow", "inline"],
        ] {
            assert!(parse_list_item(tokens).is_some(), "{tokens:?}");
        }
        assert_eq!(parse_list_item(&["inline", "list-item"]), Some(false));
        assert_eq!(
            parse_list_item(&["flow-root", "block", "list-item"]),
            Some(true)
        );
    }

    #[test]
    fn rejects_duplicate_or_unknown_list_item_components() {
        assert_eq!(parse_list_item(&["inline", "block", "list-item"]), None);
        assert_eq!(parse_list_item(&["flow", "flow-root", "list-item"]), None);
        assert_eq!(parse_list_item(&["grid", "list-item"]), None);
    }

    #[test]
    fn accepts_multi_keyword_flow_components_in_any_order() {
        assert_eq!(
            parse_flow_display(&["inline", "flow-root"]),
            Some((false, Some(false)))
        );
        assert_eq!(
            parse_flow_display(&["flow-root", "inline"]),
            Some((false, Some(false)))
        );
        assert_eq!(
            parse_flow_display(&["flow", "block"]),
            Some((false, Some(true)))
        );
    }
}
