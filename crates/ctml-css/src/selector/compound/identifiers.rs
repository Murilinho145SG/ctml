pub(super) fn identifier_end(selector: &str, start: usize) -> Option<usize> {
    let mut cursor = start;
    while cursor < selector.len() {
        let character = selector[cursor..].chars().next()?;
        if character == '\\' {
            cursor = css_escape_end(selector, cursor)?;
        } else if character.is_ascii_whitespace() || matches!(character, '.' | '#' | '[' | ':') {
            break;
        } else {
            cursor += character.len_utf8();
        }
    }
    Some(cursor)
}

pub(crate) fn css_escape_end(selector: &str, start: usize) -> Option<usize> {
    if selector.as_bytes().get(start) != Some(&b'\\') {
        return None;
    }
    let mut cursor = start + 1;
    let first = *selector.as_bytes().get(cursor)?;
    if first.is_ascii_hexdigit() {
        let mut digits = 0;
        while digits < 6
            && selector
                .as_bytes()
                .get(cursor)
                .is_some_and(u8::is_ascii_hexdigit)
        {
            cursor += 1;
            digits += 1;
        }
        if selector[cursor..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            let whitespace = selector[cursor..].chars().next()?;
            cursor += whitespace.len_utf8();
            if whitespace == '\r' && selector[cursor..].starts_with('\n') {
                cursor += 1;
            }
        }
    } else {
        cursor += selector[cursor..].chars().next()?.len_utf8();
    }
    Some(cursor)
}

pub(crate) fn decode_css_identifier(value: &str) -> Option<String> {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0;
    while cursor < value.len() {
        let character = value[cursor..].chars().next()?;
        if character != '\\' {
            output.push(character);
            cursor += character.len_utf8();
            continue;
        }
        let end = css_escape_end(value, cursor)?;
        let escaped = &value[cursor + 1..end];
        if escaped
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_hexdigit)
        {
            let hex_end = escaped.find(char::is_whitespace).unwrap_or(escaped.len());
            let codepoint = u32::from_str_radix(&escaped[..hex_end], 16).ok()?;
            output.push(
                char::from_u32(codepoint)
                    .filter(|_| codepoint != 0)
                    .unwrap_or('\u{fffd}'),
            );
        } else {
            output.push(escaped.chars().next()?);
        }
        cursor = end;
    }
    Some(output)
}
