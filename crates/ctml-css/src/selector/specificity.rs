use super::{
    compound::{attribute_selector_end, function_selector_end},
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
            if character == '[' {
                classes += 1;
                let Some(end) = attribute_selector_end(&compound, cursor) else {
                    break;
                };
                cursor = end + 1;
                continue;
            }
            if character == ':' {
                cursor += 1;
                let name_start = cursor;
                while cursor < compound.len()
                    && !matches!(
                        compound.as_bytes()[cursor],
                        b'(' | b'.' | b'#' | b'[' | b':'
                    )
                {
                    cursor += 1;
                }
                let name = compound[name_start..cursor].to_ascii_lowercase();
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
                        } else if name != "where" {
                            classes += 1;
                        }
                        cursor = end + 1;
                        continue;
                    }
                }
                classes += 1;
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
