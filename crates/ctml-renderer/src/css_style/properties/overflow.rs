use crate::style::{Overflow, TextStyle};

pub(super) fn apply_overflow(style: &mut TextStyle, value: &str) {
    if let Some((overflow_x, overflow_y)) = parse_overflow_value(value) {
        style.overflow_x = overflow_x;
        style.overflow_y = overflow_y;
    }
}

pub(super) fn apply_overflow_axis(style: &mut TextStyle, value: &str, horizontal: bool) {
    let Some(overflow) = parse_overflow_keyword(value) else {
        return;
    };
    if horizontal {
        style.overflow_x = overflow;
    } else {
        style.overflow_y = overflow;
    }
}

fn parse_overflow_value(value: &str) -> Option<(Overflow, Overflow)> {
    let mut values = value.split_ascii_whitespace();
    let first = parse_overflow_keyword(values.next()?)?;
    let second = match values.next() {
        Some(value) => Some(parse_overflow_keyword(value)?),
        None => None,
    };
    if values.next().is_some() {
        return None;
    }
    Some((first, second.unwrap_or(first)))
}

fn parse_overflow_keyword(value: &str) -> Option<Overflow> {
    match value.to_ascii_lowercase().as_str() {
        "visible" => Some(Overflow::Visible),
        "hidden" => Some(Overflow::Hidden),
        "clip" => Some(Overflow::Clip),
        "auto" => Some(Overflow::Auto),
        "scroll" => Some(Overflow::Scroll),
        _ => None,
    }
}
