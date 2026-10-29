#[derive(Clone)]
pub struct ElementIdentity {
    pub tag: String,
    pub class: Option<String>,
    pub id: Option<String>,
}

pub fn selector_matches(
    selector: &str,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
) -> bool {
    selector_matches_with_attributes(selector, current, ancestors, &[])
}

pub fn selector_matches_with_attributes(
    selector: &str,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
) -> bool {
    selector_matches_with_context(selector, current, ancestors, attributes, &[])
}

pub fn selector_matches_with_context(
    selector: &str,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
) -> bool {
    let Some(compounds) = selector_compounds(selector) else {
        return false;
    };
    let Some((last, _)) = compounds.last() else {
        return false;
    };
    if !compound_matches(last, current, attributes, ancestors, ancestor_attributes) {
        return false;
    }
    let mut ancestor_index = ancestors.len();
    for index in (0..compounds.len() - 1).rev() {
        let (compound, _) = &compounds[index];
        match compounds[index + 1].1 {
            SelectorRelation::Child => {
                let Some(parent_index) = ancestor_index.checked_sub(1) else {
                    return false;
                };
                if !compound_matches(
                    compound,
                    &ancestors[parent_index],
                    ancestor_attributes
                        .get(parent_index)
                        .map_or(&[], Vec::as_slice),
                    &ancestors[..parent_index],
                    &ancestor_attributes[..parent_index.min(ancestor_attributes.len())],
                ) {
                    return false;
                }
                ancestor_index = parent_index;
            }
            SelectorRelation::Descendant => {
                let Some(found) = ancestors[..ancestor_index].iter().enumerate().rposition(
                    |(index, ancestor)| {
                        compound_matches(
                            compound,
                            ancestor,
                            ancestor_attributes.get(index).map_or(&[], Vec::as_slice),
                            &ancestors[..index],
                            &ancestor_attributes[..index.min(ancestor_attributes.len())],
                        )
                    },
                ) else {
                    return false;
                };
                ancestor_index = found;
            }
        }
    }
    true
}

#[derive(Clone, Copy)]
enum SelectorRelation {
    Descendant,
    Child,
}

fn selector_compounds(selector: &str) -> Option<Vec<(String, SelectorRelation)>> {
    let mut compounds = Vec::new();
    let mut start = None;
    let mut relation = SelectorRelation::Descendant;
    let mut bracket_depth = 0;
    let mut paren_depth = 0;
    let mut quote = None;
    for (index, character) in selector.char_indices() {
        if paren_depth > 0 {
            match (quote, character) {
                (Some(current), character) if current == character => quote = None,
                (None, '"' | '\'') => quote = Some(character),
                (None, '(') => paren_depth += 1,
                (None, ')') => paren_depth -= 1,
                _ => {}
            }
            continue;
        }
        if bracket_depth > 0 {
            match (quote, character) {
                (Some(current), character) if current == character => quote = None,
                (None, '"' | '\'') => quote = Some(character),
                (None, '[') => bracket_depth += 1,
                (None, ']') => bracket_depth -= 1,
                _ => {}
            }
            continue;
        }
        if character == '[' {
            bracket_depth = 1;
            if start.is_none() {
                start = Some(index);
            }
            continue;
        }
        if character == '(' {
            paren_depth = 1;
            continue;
        }
        if character.is_ascii_whitespace() || character == '>' {
            if let Some(compound_start) = start.take() {
                compounds.push((selector[compound_start..index].to_owned(), relation));
                relation = SelectorRelation::Descendant;
            } else if character == '>'
                && (compounds.is_empty() || matches!(relation, SelectorRelation::Child))
            {
                return None;
            }
            if character == '>' {
                relation = SelectorRelation::Child;
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(compound_start) = start {
        compounds.push((selector[compound_start..].to_owned(), relation));
        Some(compounds)
    } else {
        None
    }
}

fn compound_matches(
    compound: &str,
    element: &ElementIdentity,
    attributes: &[(String, Option<String>)],
    ancestors: &[ElementIdentity],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
) -> bool {
    if compound == "*" {
        return true;
    }
    let mut cursor = 0;
    let bytes = compound.as_bytes();
    if bytes.first() == Some(&b'*') {
        cursor = 1;
    } else if bytes
        .first()
        .is_some_and(|byte| *byte != b'.' && *byte != b'#' && *byte != b'[' && *byte != b':')
    {
        let start = cursor;
        while cursor < bytes.len()
            && bytes[cursor] != b'.'
            && bytes[cursor] != b'#'
            && bytes[cursor] != b'['
            && bytes[cursor] != b':'
        {
            cursor += 1;
        }
        if !compound[start..cursor].eq_ignore_ascii_case(&element.tag) {
            return false;
        }
    }
    while cursor < bytes.len() {
        let marker = bytes[cursor];
        if marker == b'[' {
            let Some(end) = attribute_selector_end(compound, cursor) else {
                return false;
            };
            if !attribute_matches(&compound[cursor + 1..end], attributes) {
                return false;
            }
            cursor = end + 1;
            continue;
        }
        if marker == b':' {
            cursor += 1;
            let name_start = cursor;
            while cursor < bytes.len()
                && bytes[cursor] != b'('
                && bytes[cursor] != b'.'
                && bytes[cursor] != b'#'
                && bytes[cursor] != b'['
                && bytes[cursor] != b':'
                && bytes[cursor] != b':'
            {
                cursor += 1;
            }
            if name_start == cursor || bytes.get(cursor) != Some(&b'(') {
                return false;
            }
            let Some(end) = function_selector_end(compound, cursor) else {
                return false;
            };
            let argument = &compound[cursor + 1..end];
            let matches = split_selector_list(argument)
                .into_iter()
                .map(str::trim)
                .filter(|selector| !selector.is_empty())
                .any(|selector| {
                    selector_matches_with_context(
                        selector,
                        element,
                        ancestors,
                        attributes,
                        ancestor_attributes,
                    )
                });
            let pseudo_matches = match compound[name_start..cursor].to_ascii_lowercase().as_str() {
                "is" | "where" => matches,
                "not" => !matches,
                _ => false,
            };
            if !pseudo_matches {
                return false;
            }
            cursor = end + 1;
            continue;
        }
        if marker != b'.' && marker != b'#' {
            return false;
        }
        cursor += 1;
        let start = cursor;
        while cursor < bytes.len()
            && bytes[cursor] != b'.'
            && bytes[cursor] != b'#'
            && bytes[cursor] != b'['
        {
            cursor += 1;
        }
        if start == cursor {
            return false;
        }
        let name = &compound[start..cursor];
        if marker == b'#' && element.id.as_deref() != Some(name) {
            return false;
        }
        if marker == b'.'
            && !element
                .class
                .as_deref()
                .is_some_and(|classes| classes.split_ascii_whitespace().any(|class| class == name))
        {
            return false;
        }
    }
    true
}

fn attribute_selector_end(selector: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    for (offset, character) in selector[start + 1..].char_indices() {
        match (quote, character) {
            (Some(current), character) if current == character => quote = None,
            (None, '"' | '\'') => quote = Some(character),
            (None, ']') => return Some(start + 1 + offset),
            _ => {}
        }
    }
    None
}

fn function_selector_end(selector: &str, start: usize) -> Option<usize> {
    let mut depth = 0;
    let mut quote = None;
    for (offset, character) in selector[start..].char_indices() {
        match (quote, character) {
            (Some(current), character) if current == character => quote = None,
            (None, '"' | '\'') => quote = Some(character),
            (None, '(') => depth += 1,
            (None, ')') => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_selector_list(source: &str) -> Vec<&str> {
    let mut selectors = Vec::new();
    let mut start = 0;
    let mut bracket_depth = 0;
    let mut paren_depth = 0;
    let mut quote = None;
    for (index, character) in source.char_indices() {
        if quote.is_some() {
            if quote == Some(character) {
                quote = None;
            }
            continue;
        }
        if character == '"' || character == '\'' {
            quote = Some(character);
        } else if character == '[' {
            bracket_depth += 1;
        } else if character == ']' {
            bracket_depth -= 1;
        } else if bracket_depth == 0 && character == '(' {
            paren_depth += 1;
        } else if bracket_depth == 0 && character == ')' {
            paren_depth -= 1;
        } else if bracket_depth == 0 && paren_depth == 0 && character == ',' {
            selectors.push(&source[start..index]);
            start = index + 1;
        }
    }
    selectors.push(&source[start..]);
    selectors
}

fn attribute_matches(selector: &str, attributes: &[(String, Option<String>)]) -> bool {
    let selector = selector.trim();
    if selector.is_empty() {
        return false;
    }
    let operator_start = selector.find(['=', '~', '|', '^', '$', '*']);
    let (name, operator, value_and_flag) = if let Some(index) = operator_start {
        let Some(operator) = (match selector.as_bytes().get(index..index + 2) {
            Some(b"~=") => Some("~="),
            Some(b"|=") => Some("|="),
            Some(b"^=") => Some("^="),
            Some(b"$=") => Some("$="),
            Some(b"*=") => Some("*="),
            _ if selector.as_bytes().get(index) == Some(&b'=') => Some("="),
            _ => None,
        }) else {
            return false;
        };
        (
            selector[..index].trim(),
            Some(operator),
            selector[index + operator.len()..].trim(),
        )
    } else {
        (selector, None, "")
    };
    if name.is_empty() {
        return false;
    }
    let attribute = attributes
        .iter()
        .find(|(attribute, _)| attribute.eq_ignore_ascii_case(name));
    let Some((_, attribute_value)) = attribute else {
        return false;
    };
    let Some(operator) = operator else {
        return true;
    };
    let Some(attribute_value) = attribute_value.as_deref() else {
        return false;
    };

    let (value, case_insensitive) = match value_and_flag.rsplit_once(char::is_whitespace) {
        Some((value, flag)) if flag.trim().eq_ignore_ascii_case("i") => (value.trim(), true),
        Some((value, flag)) if flag.trim().eq_ignore_ascii_case("s") => (value.trim(), false),
        _ => (value_and_flag, false),
    };
    let value = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value);
    let (attribute_value, value) = if case_insensitive {
        (
            attribute_value.to_ascii_lowercase(),
            value.to_ascii_lowercase(),
        )
    } else {
        (attribute_value.to_owned(), value.to_owned())
    };
    match operator {
        "=" => attribute_value == value,
        "~=" => attribute_value
            .split_ascii_whitespace()
            .any(|item| item == value),
        "|=" => attribute_value == value || attribute_value.starts_with(&format!("{value}-")),
        "^=" => !value.is_empty() && attribute_value.starts_with(&value),
        "$=" => !value.is_empty() && attribute_value.ends_with(&value),
        "*=" => !value.is_empty() && attribute_value.contains(&value),
        _ => false,
    }
}

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
                                .map(selector_specificity)
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
