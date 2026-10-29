use super::{
    compound::{attribute_selector_end, css_escape_end, function_selector_end},
    list::split_selector_list,
    selector_compounds,
};

pub fn selector_specificity(selector: &str) -> (usize, usize, usize) {
    let Some(compounds) = selector_compounds(selector) else {
        return (0, 0, 0);
    };
    let mut ids = 0;
    let mut classes = 0;
    let mut tags = 0;
    for (compound, _) in compounds {
        if !compound.starts_with(['.', '#', '*', '[', ':']) {
            tags += 1;
        }
        let mut cursor = 0;
        while cursor < compound.len() {
            let character = compound[cursor..].chars().next().unwrap();
            if character == '\\' {
                let Some(end) = css_escape_end(&compound, cursor) else {
                    break;
                };
                cursor = end;
                continue;
            }
            if character == '[' {
                classes += 1;
                let Some(end) = attribute_selector_end(&compound, cursor) else {
                    break;
                };
                cursor = end + 1;
                continue;
            }
            if character == ':' {
                if compound.as_bytes().get(cursor + 1) == Some(&b':') {
                    tags += 1;
                    cursor += 2;
                    while cursor < compound.len()
                        && !matches!(
                            compound.as_bytes()[cursor],
                            b'(' | b'.' | b'#' | b'[' | b':'
                        )
                    {
                        if compound.as_bytes()[cursor] == b'\\' {
                            if let Some(end) = css_escape_end(&compound, cursor) {
                                cursor = end;
                            } else {
                                break;
                            }
                        } else {
                            cursor += compound[cursor..].chars().next().unwrap().len_utf8();
                        }
                    }
                    continue;
                }
                cursor += 1;
                let name_start = cursor;
                while cursor < compound.len() && compound.as_bytes()[cursor] != b'(' {
                    if compound.as_bytes()[cursor] == b'\\' {
                        if let Some(end) = css_escape_end(&compound, cursor) {
                            cursor = end;
                        } else {
                            break;
                        }
                    } else if matches!(compound.as_bytes()[cursor], b'.' | b'#' | b'[' | b':') {
                        break;
                    } else {
                        cursor += compound[cursor..].chars().next().unwrap().len_utf8();
                    }
                }
                let Some(name) =
                    super::compound::decode_css_identifier(&compound[name_start..cursor])
                else {
                    break;
                };
                let name = name.to_ascii_lowercase();
                let legacy_pseudo_element = matches!(
                    name.as_str(),
                    "before" | "after" | "first-line" | "first-letter"
                );
                if cursor < compound.len() && compound.as_bytes()[cursor] == b'(' {
                    if let Some(end) = function_selector_end(&compound, cursor) {
                        let arguments = split_selector_list(&compound[cursor + 1..end]);
                        if name == "is" || name == "not" || name == "has" {
                            let specificity = arguments
                                .into_iter()
                                .map(|argument| {
                                    if name == "has" {
                                        let argument = argument.trim_start();
                                        let relative_selector = argument
                                            .strip_prefix(['>', '+', '~'])
                                            .map(str::trim_start)
                                            .unwrap_or(argument);
                                        selector_specificity(relative_selector)
                                    } else {
                                        selector_specificity(argument)
                                    }
                                })
                                .max()
                                .unwrap_or_default();
                            ids += specificity.0;
                            classes += specificity.1;
                            tags += specificity.2;
                        } else if name == "nth-child" || name == "nth-last-child" {
                            classes += 1;
                            if let Some(selectors) = nth_of_selector(&compound[cursor + 1..end]) {
                                let specificity = split_selector_list(selectors)
                                    .into_iter()
                                    .map(selector_specificity)
                                    .max()
                                    .unwrap_or_default();
                                ids += specificity.0;
                                classes += specificity.1;
                                tags += specificity.2;
                            }
                        } else if name != "where" {
                            classes += 1;
                        }
                        cursor = end + 1;
                        continue;
                    }
                }
                if legacy_pseudo_element {
                    tags += 1;
                } else {
                    classes += 1;
                }
                continue;
            }
            match character {
                '#' => ids += 1,
                '.' => classes += 1,
                _ => {}
            }
            cursor += character.len_utf8();
        }
    }
    (ids, classes, tags)
}

fn nth_of_selector(argument: &str) -> Option<&str> {
    let bytes = argument.as_bytes();
    let mut cursor = 0;
    while cursor + 2 < bytes.len() {
        if bytes[cursor].eq_ignore_ascii_case(&b'o')
            && bytes[cursor + 1].eq_ignore_ascii_case(&b'f')
            && cursor > 0
            && bytes[cursor - 1].is_ascii_whitespace()
            && bytes[cursor + 2].is_ascii_whitespace()
        {
            return Some(argument[cursor + 2..].trim());
        }
        cursor += 1;
    }
    None
}
