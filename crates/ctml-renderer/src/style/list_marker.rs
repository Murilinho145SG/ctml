pub(super) fn format_marker(
    list_style_type: Option<&str>,
    ordered_parent: bool,
    index: usize,
) -> Option<String> {
    let list_style_type =
        list_style_type.unwrap_or(if ordered_parent { "decimal" } else { "disc" });
    if let Some(marker) = list_style_type.strip_prefix("custom:") {
        return Some(format!("{marker} "));
    }
    Some(match list_style_type {
        "none" => return None,
        "disc" => "• ".to_owned(),
        "circle" => "◦ ".to_owned(),
        "square" => "▪ ".to_owned(),
        "decimal" => format!("{index}. "),
        "decimal-leading-zero" => format!("{index:02}. "),
        "lower-alpha" => format!("{}. ", alphabetic_marker(index, false)),
        "upper-alpha" => format!("{}. ", alphabetic_marker(index, true)),
        "lower-greek" => format!(
            "{}. ",
            alphabetic_marker_from(
                index,
                &[
                    'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'ι', 'κ', 'λ', 'μ', 'ν', 'ξ', 'ο', 'π',
                    'ρ', 'σ', 'τ', 'υ', 'φ', 'χ', 'ψ', 'ω',
                ]
            )
        ),
        "lower-roman" => format!("{}. ", roman_marker(index).to_ascii_lowercase()),
        "upper-roman" => format!("{}. ", roman_marker(index)),
        _ => return None,
    })
}

pub(super) fn format_counter_value(style: &str, value: i32) -> Option<String> {
    let positive = usize::try_from(value).ok().filter(|value| *value > 0);
    match style {
        "decimal" => Some(value.to_string()),
        "decimal-leading-zero" if (0..10).contains(&value) => Some(format!("0{value}")),
        "decimal-leading-zero" => Some(value.to_string()),
        "lower-alpha" => {
            Some(positive.map_or_else(|| value.to_string(), |n| alphabetic_marker(n, false)))
        }
        "upper-alpha" => {
            Some(positive.map_or_else(|| value.to_string(), |n| alphabetic_marker(n, true)))
        }
        "lower-greek" => Some(positive.map_or_else(
            || value.to_string(),
            |n| {
                alphabetic_marker_from(
                    n,
                    &[
                        'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'ι', 'κ', 'λ', 'μ', 'ν', 'ξ', 'ο',
                        'π', 'ρ', 'σ', 'τ', 'υ', 'φ', 'χ', 'ψ', 'ω',
                    ],
                )
            },
        )),
        "lower-roman" | "upper-roman" => Some(positive.map_or_else(
            || value.to_string(),
            |n| {
                let roman = roman_marker(n);
                if style == "lower-roman" {
                    roman.to_ascii_lowercase()
                } else {
                    roman
                }
            },
        )),
        _ => None,
    }
}

fn alphabetic_marker(index: usize, uppercase: bool) -> String {
    let alphabet = if uppercase {
        ('A'..='Z').collect::<Vec<_>>()
    } else {
        ('a'..='z').collect::<Vec<_>>()
    };
    alphabetic_marker_from(index, &alphabet)
}

fn alphabetic_marker_from(mut index: usize, alphabet: &[char]) -> String {
    let mut marker = Vec::new();
    while index > 0 {
        let digit = (index - 1) % alphabet.len();
        marker.push(alphabet[digit]);
        index = (index - 1) / alphabet.len();
    }
    marker.iter().rev().collect()
}

fn roman_marker(index: usize) -> String {
    if !(1..=3999).contains(&index) {
        return index.to_string();
    }
    let mut value = index;
    let mut marker = String::new();
    for (amount, symbol) in [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ] {
        while value >= amount {
            marker.push_str(symbol);
            value -= amount;
        }
    }
    marker
}
