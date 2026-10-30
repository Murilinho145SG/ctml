pub(super) fn format(style: &str, index: usize) -> Option<String> {
    let mut parts = style.strip_prefix("symbols:")?.splitn(3, ':');
    let system = parts.next()?;
    let first_value = parts.next()?.parse::<i32>().ok()?;
    let encoded_symbols = parts.next()?;
    let symbols = decode_symbols(encoded_symbols)?;
    if symbols.is_empty() {
        return None;
    }
    let value = i32::try_from(index).ok()?;
    let marker = match system {
        "cyclic" => symbols[(index.saturating_sub(1)) % symbols.len()].clone(),
        "fixed" => {
            let offset = value.checked_sub(first_value)?;
            if offset < 0 || usize::try_from(offset).ok()? >= symbols.len() {
                return Some(format!("{index}. "));
            }
            symbols[usize::try_from(offset).ok()?].clone()
        }
        "symbolic" => {
            let offset = index.saturating_sub(1);
            symbols[offset % symbols.len()].repeat(offset / symbols.len() + 1)
        }
        "alphabetic" => alphabetic(index, &symbols)?,
        "numeric" => numeric(index, &symbols),
        _ => return None,
    };
    Some(format!("{marker}. "))
}

fn decode_symbols(mut encoded: &str) -> Option<Vec<String>> {
    let mut symbols = Vec::new();
    while !encoded.is_empty() {
        let (length, rest) = encoded.split_once(':')?;
        let length = length.parse::<usize>().ok()?;
        if rest.len() < length || !rest.is_char_boundary(length) {
            return None;
        }
        symbols.push(rest[..length].to_owned());
        encoded = &rest[length..];
    }
    Some(symbols)
}

fn alphabetic(mut value: usize, symbols: &[String]) -> Option<String> {
    if value == 0 {
        return None;
    }
    let base = symbols.len();
    let mut digits = Vec::new();
    while value > 0 {
        value -= 1;
        digits.push(symbols[value % base].as_str());
        value /= base;
    }
    Some(digits.into_iter().rev().collect())
}

fn numeric(mut value: usize, symbols: &[String]) -> String {
    if value == 0 {
        return symbols[0].clone();
    }
    let base = symbols.len();
    let mut digits = Vec::new();
    while value > 0 {
        digits.push(symbols[value % base].as_str());
        value /= base;
    }
    digits.into_iter().rev().collect()
}
