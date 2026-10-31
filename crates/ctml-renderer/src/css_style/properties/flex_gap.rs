use crate::style::TextStyle;
use ctml_css::{parse_math_expression, parse_nonnegative_font_relative_length_with_viewport};

pub(super) fn apply_flex_gap(style: &mut TextStyle, property: &str, value: &str) {
    let order = style.current_declaration_order.saturating_add(1);
    match property {
        "gap" => {
            let values = split_gap_values(value);
            if !(1..=2).contains(&values.len()) {
                return;
            }
            let Some(row_gap) = parse_gap_value(style, values[0]) else {
                return;
            };
            let column_gap = if values.len() == 2 {
                let Some(column_gap) = parse_gap_value(style, values[1]) else {
                    return;
                };
                column_gap
            } else {
                row_gap.clone()
            };
            set_row_gap(style, row_gap, order);
            set_column_gap(style, column_gap, order);
        }
        "row-gap" => {
            if let Some(gap) = parse_gap_value(style, value) {
                set_row_gap(style, gap, order);
            }
        }
        "column-gap" => {
            if let Some(gap) = parse_gap_value(style, value) {
                set_column_gap(style, gap, order);
            }
        }
        _ => {}
    }
}

pub(super) fn inherit_flex_gap(style: &mut TextStyle, inherited: &TextStyle, property: &str) {
    match property {
        "gap" => {
            style.row_gap.clone_from(&inherited.row_gap);
            style.column_gap.clone_from(&inherited.column_gap);
        }
        "row-gap" => style.row_gap.clone_from(&inherited.row_gap),
        "column-gap" => style.column_gap.clone_from(&inherited.column_gap),
        _ => {}
    }
}

pub(super) fn initial_flex_gap(style: &mut TextStyle, property: &str) {
    let order = style.current_declaration_order.saturating_add(1);
    match property {
        "gap" => {
            set_row_gap(style, "0".to_owned(), order);
            set_column_gap(style, "0".to_owned(), order);
        }
        "row-gap" => set_row_gap(style, "0".to_owned(), order),
        "column-gap" => set_column_gap(style, "0".to_owned(), order),
        _ => {}
    }
}

fn set_row_gap(style: &mut TextStyle, value: String, order: usize) {
    if order >= style.row_gap_order {
        style.row_gap = Some(value);
        style.row_gap_order = order;
    }
}

fn set_column_gap(style: &mut TextStyle, value: String, order: usize) {
    if order >= style.column_gap_order {
        style.column_gap = Some(value);
        style.column_gap_order = order;
    }
}

fn parse_gap_value(style: &TextStyle, value: &str) -> Option<String> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("normal") {
        return Some("0".to_owned());
    }
    let length_is_valid = |component: &str| {
        parse_nonnegative_font_relative_length_with_viewport(
            component,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
        .or_else(|| {
            ctml_css::parse_css_percentage(component).filter(|percentage| *percentage >= 0.0)
        })
    };
    if length_is_valid(value).is_some()
        || parse_math_expression(value, |component| length_is_valid(component)).is_some()
    {
        Some(value.to_ascii_lowercase())
    } else {
        None
    }
}

fn split_gap_values(value: &str) -> Vec<&str> {
    let mut values = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            character if character.is_ascii_whitespace() && depth == 0 => {
                if let Some(start) = start.take() {
                    values.push(&value[start..index]);
                }
            }
            _ => {
                start.get_or_insert(index);
            }
        }
    }
    if let Some(start) = start {
        values.push(&value[start..]);
    }
    values
}
