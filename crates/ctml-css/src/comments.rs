pub(crate) fn strip_utf8_bom_and_comments(source: &str) -> Result<String, String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let mut output = String::with_capacity(source.len());
    let mut index = 0;
    let mut quote = None;
    let mut quote_start = 0;

    while index < source.len() {
        let character = source[index..]
            .chars()
            .next()
            .expect("valid character boundary");
        if let Some(active_quote) = quote {
            if character == '\\' {
                let end = escape_end(source, index);
                output.push_str(&source[index..end]);
                index = end;
            } else {
                output.push(character);
                index += character.len_utf8();
                if character == active_quote {
                    quote = None;
                }
            }
        } else if source[index..].starts_with("/*") {
            let Some(comment_end) = source[index + 2..].find("*/") else {
                return Err(source_error(source, index, "unterminated CSS comment"));
            };
            index += comment_end + 4;
        } else if matches!(character, '\'' | '"') {
            quote = Some(character);
            quote_start = index;
            output.push(character);
            index += character.len_utf8();
        } else if character == '\\' {
            let end = escape_end(source, index);
            output.push_str(&source[index..end]);
            index = end;
        } else {
            output.push(character);
            index += character.len_utf8();
        }
    }

    if quote.is_some() {
        return Err(source_error(source, quote_start, "unterminated CSS string"));
    }
    Ok(output)
}

pub(crate) fn source_error(source: &str, position: usize, message: &str) -> String {
    let prefix = &source[..position];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column_start = prefix.rfind('\n').map_or(0, |newline| newline + 1);
    let column = source[column_start..position].encode_utf16().count() + 1;
    format!("line {line}, column {column}: {message}")
}

fn escape_end(source: &str, start: usize) -> usize {
    let mut index = start + '\\'.len_utf8();
    if index == source.len() {
        return index;
    }
    let first = source[index..]
        .chars()
        .next()
        .expect("valid character boundary");
    if first.is_ascii_hexdigit() {
        let mut digits = 0;
        while index < source.len() && digits < 6 {
            let character = source[index..]
                .chars()
                .next()
                .expect("valid character boundary");
            if !character.is_ascii_hexdigit() {
                break;
            }
            index += character.len_utf8();
            digits += 1;
        }
        if index < source.len() {
            let character = source[index..]
                .chars()
                .next()
                .expect("valid character boundary");
            if character.is_whitespace() {
                index += character.len_utf8();
            }
        }
    } else {
        index += first.len_utf8();
    }
    index
}
