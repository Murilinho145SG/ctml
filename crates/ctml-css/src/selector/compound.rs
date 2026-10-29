use super::list::split_selector_list;
use super::{
    ElementIdentity, SelectorPosition, SelectorSibling, SelectorTreeElement, selector_has_matches,
    selector_matches_with_selector_tree_context_and_scope,
};
mod identifiers;
mod nth;
mod state;
use identifiers::identifier_end;
pub(super) use identifiers::{css_escape_end, decode_css_identifier};
use nth::nth_child_matches;
use state::{direction_pseudo_matches, language_pseudo_matches, link_pseudo_matches};

pub(super) fn compound_matches(
    compound: &str,
    element: &ElementIdentity,
    attributes: &[(String, Option<String>)],
    ancestors: &[ElementIdentity],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    siblings_before_current: &[SelectorSibling],
    siblings_before_ancestors: &[Vec<SelectorSibling>],
    position: SelectorPosition,
    ancestor_positions: &[SelectorPosition],
    selector_tree: &[SelectorTreeElement],
    scope_node_id: Option<usize>,
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
        let Some(end) = identifier_end(compound, cursor) else {
            return false;
        };
        let Some(name) = decode_css_identifier(&compound[cursor..end]) else {
            return false;
        };
        cursor = end;
        if !name.eq_ignore_ascii_case(&element.tag) {
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
            while cursor < bytes.len() && bytes[cursor] != b'(' {
                if bytes[cursor] == b'\\' {
                    let Some(end) = css_escape_end(compound, cursor) else {
                        return false;
                    };
                    cursor = end;
                } else if matches!(bytes[cursor], b'.' | b'#' | b'[' | b':') {
                    break;
                } else {
                    let Some(character) = compound[cursor..].chars().next() else {
                        return false;
                    };
                    cursor += character.len_utf8();
                }
            }
            if name_start == cursor {
                return false;
            }
            let Some(name) = decode_css_identifier(&compound[name_start..cursor]) else {
                return false;
            };
            let name = name.to_ascii_lowercase();
            if bytes.get(cursor) != Some(&b'(') {
                let matches = if name == "scope" {
                    scope_node_id == Some(position.node_id)
                } else if matches!(name.as_str(), "link" | "visited" | "any-link") {
                    link_pseudo_matches(&name, element, attributes)
                } else {
                    structural_pseudo_matches(&name, position)
                };
                if !matches {
                    return false;
                }
                continue;
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
                    selector_matches_with_selector_tree_context_and_scope(
                        selector,
                        element,
                        ancestors,
                        attributes,
                        ancestor_attributes,
                        siblings_before_current,
                        siblings_before_ancestors,
                        position,
                        ancestor_positions,
                        selector_tree,
                        scope_node_id,
                    )
                });
            let argument = argument.trim();
            let pseudo_matches = match name.as_str() {
                "is" | "where" => matches,
                "not" => !matches,
                "has" => selector_has_matches(argument, position.node_id, selector_tree),
                "lang" => language_pseudo_matches(argument, attributes, ancestor_attributes),
                "dir" => direction_pseudo_matches(argument, attributes, ancestor_attributes),
                "nth-child" => {
                    nth_child_matches(argument, position.node_id, position, selector_tree, false)
                }
                "nth-last-child" => {
                    position.child_index > 0
                        && nth_child_matches(
                            argument,
                            position.node_id,
                            position,
                            selector_tree,
                            true,
                        )
                }
                "nth-of-type" => nth_expression_matches(argument, position.type_index),
                "nth-last-of-type" => {
                    position.type_index > 0
                        && nth_expression_matches(
                            argument,
                            position.type_count.saturating_sub(position.type_index) + 1,
                        )
                }
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
        let Some(end) = identifier_end(compound, cursor) else {
            return false;
        };
        cursor = end;
        if start == cursor {
            return false;
        }
        let Some(name) = decode_css_identifier(&compound[start..cursor]) else {
            return false;
        };
        if marker == b'#' && element.id.as_deref() != Some(name.as_str()) {
            return false;
        }
        if marker == b'.'
            && !element.class.as_deref().is_some_and(|classes| {
                classes
                    .split_ascii_whitespace()
                    .any(|class| class == name.as_str())
            })
        {
            return false;
        }
    }
    true
}

fn structural_pseudo_matches(name: &str, position: SelectorPosition) -> bool {
    match name {
        "root" => position.is_root,
        "empty" => position.is_empty,
        "hover" => position.is_hovered,
        "active" => position.is_active,
        "first-child" | "first-of-type" => {
            if name == "first-child" {
                position.child_index == 1
            } else {
                position.type_index == 1
            }
        }
        "last-child" | "last-of-type" => {
            if name == "last-child" {
                position.child_index > 0 && position.child_index == position.child_count
            } else {
                position.type_index > 0 && position.type_index == position.type_count
            }
        }
        "only-child" => position.child_count == 1,
        "only-of-type" => position.type_count == 1,
        _ => false,
    }
}

fn nth_expression_matches(expression: &str, index: usize) -> bool {
    if index == 0 {
        return false;
    }
    let expression = expression
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    if expression == "odd" {
        return index % 2 == 1;
    }
    if expression == "even" {
        return index % 2 == 0;
    }
    let Some(n_index) = expression.find('n') else {
        return expression.parse::<usize>().ok() == Some(index);
    };
    if expression[n_index + 1..].contains('n') {
        return false;
    }
    let coefficient = match &expression[..n_index] {
        "" | "+" => 1,
        "-" => -1,
        value => match value.parse::<i64>() {
            Ok(value) => value,
            Err(_) => return false,
        },
    };
    let remainder = &expression[n_index + 1..];
    let offset = if remainder.is_empty() {
        0
    } else {
        match remainder.parse::<i64>() {
            Ok(value) => value,
            Err(_) => return false,
        }
    };
    let delta = index as i64 - offset;
    if coefficient == 0 {
        return delta == 0;
    }
    delta % coefficient == 0 && delta / coefficient >= 0
}

pub(super) fn attribute_selector_end(selector: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    let mut escaped_until = 0;
    for (offset, character) in selector[start + 1..].char_indices() {
        let index = start + 1 + offset;
        if index < escaped_until {
            continue;
        }
        if character == '\\' {
            escaped_until = css_escape_end(selector, index)?;
            continue;
        }
        match (quote, character) {
            (Some(current), character) if current == character => quote = None,
            (None, '"' | '\'') => quote = Some(character),
            (None, ']') => return Some(start + 1 + offset),
            _ => {}
        }
    }
    None
}

pub(super) fn function_selector_end(selector: &str, start: usize) -> Option<usize> {
    let mut depth = 0;
    let mut quote = None;
    let mut escaped_until = 0;
    for (offset, character) in selector[start..].char_indices() {
        let index = start + offset;
        if index < escaped_until {
            continue;
        }
        if character == '\\' {
            escaped_until = css_escape_end(selector, index)?;
            continue;
        }
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

fn attribute_matches(selector: &str, attributes: &[(String, Option<String>)]) -> bool {
    let selector = selector.trim();
    if selector.is_empty() {
        return false;
    }
    let operator_start = find_attribute_operator(selector);
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
    let Some(name) = decode_css_identifier(name) else {
        return false;
    };
    if name.is_empty() {
        return false;
    }
    let attribute = attributes
        .iter()
        .find(|(attribute, _)| attribute.eq_ignore_ascii_case(&name));
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
    let Some(value) = decode_css_identifier(value) else {
        return false;
    };
    let (attribute_value, value) = if case_insensitive {
        (
            attribute_value.to_ascii_lowercase(),
            value.to_ascii_lowercase(),
        )
    } else {
        (attribute_value.to_owned(), value)
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

fn find_attribute_operator(selector: &str) -> Option<usize> {
    let mut quote = None;
    let mut cursor = 0;
    while cursor < selector.len() {
        let character = selector[cursor..].chars().next()?;
        if character == '\\' {
            cursor = css_escape_end(selector, cursor)?;
            continue;
        }
        if let Some(current) = quote {
            if current == character {
                quote = None;
            }
        } else if matches!(character, '"' | '\'') {
            quote = Some(character);
        } else if matches!(character, '=' | '~' | '|' | '^' | '$' | '*') {
            return Some(cursor);
        }
        cursor += character.len_utf8();
    }
    None
}
