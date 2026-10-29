pub(super) fn format_marker(
    list_style_type: Option<&str>,
    ordered_parent: bool,
    index: usize,
) -> Option<String> {
    let list_style_type =
        list_style_type.unwrap_or(if ordered_parent { "decimal" } else { "disc" });
    Some(match list_style_type {
        "none" => return None,
        "disc" => "• ".to_owned(),
        "circle" => "◦ ".to_owned(),
        "square" => "▪ ".to_owned(),
        "decimal" => format!("{index}. "),
        "lower-alpha" => format!("{}. ", alphabetic_marker(index, false)),
        "upper-alpha" => format!("{}. ", alphabetic_marker(index, true)),
        _ => return None,
    })
}

fn alphabetic_marker(mut index: usize, uppercase: bool) -> String {
    let mut marker = Vec::new();
    while index > 0 {
        let digit = (index - 1) % 26;
        let base = if uppercase { b'A' } else { b'a' };
        marker.push((base + digit as u8) as char);
        index = (index - 1) / 26;
    }
    marker.iter().rev().collect()
}
