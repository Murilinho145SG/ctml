use super::{SelectorPosition, SelectorTreeElement, css_escape_end, nth_expression_matches};
use crate::selector::{has::matches_tree_element_with_scope, list::split_selector_list};

pub(super) fn nth_child_matches(
    argument: &str,
    node_id: usize,
    position: SelectorPosition,
    selector_tree: &[SelectorTreeElement],
    from_end: bool,
) -> bool {
    let Some((formula, selector_list)) = split_nth_of_argument(argument) else {
        let index = if from_end {
            position.child_count.saturating_sub(position.child_index) + 1
        } else {
            position.child_index
        };
        return nth_expression_matches(argument, index);
    };
    let Some(current) = selector_tree
        .iter()
        .find(|element| element.node_id == node_id)
    else {
        return false;
    };
    let Some(parent_id) = current.parent_id else {
        return false;
    };
    let mut siblings = selector_tree
        .iter()
        .filter(|element| element.parent_id == Some(parent_id))
        .collect::<Vec<_>>();
    siblings.sort_by_key(|element| element.position.child_index);
    let Some(current_index) = siblings
        .iter()
        .position(|element| element.node_id == node_id)
    else {
        return false;
    };
    let selectors = split_selector_list(selector_list);
    let matches = siblings
        .iter()
        .map(|element| {
            selectors.iter().any(|selector| {
                matches_tree_element_with_scope(selector.trim(), element, selector_tree, None)
            })
        })
        .collect::<Vec<_>>();
    if !matches[current_index] {
        return false;
    }
    let index = if from_end {
        matches[current_index + 1..]
            .iter()
            .filter(|matches_selector| **matches_selector)
            .count()
            + 1
    } else {
        matches[..current_index]
            .iter()
            .filter(|matches_selector| **matches_selector)
            .count()
            + 1
    };
    nth_expression_matches(formula, index)
}

fn split_nth_of_argument(argument: &str) -> Option<(&str, &str)> {
    let bytes = argument.as_bytes();
    let mut cursor = 0;
    let mut nested_depth = 0usize;
    let mut quote = None;
    while cursor < bytes.len() {
        let character = argument[cursor..].chars().next()?;
        if character == '\\' {
            cursor = css_escape_end(argument, cursor)?;
            continue;
        }
        if let Some(current_quote) = quote {
            if current_quote == character {
                quote = None;
            }
        } else {
            match character {
                '"' | '\'' => quote = Some(character),
                '(' | '[' => nested_depth += 1,
                ')' | ']' => nested_depth = nested_depth.saturating_sub(1),
                'o' | 'O'
                    if nested_depth == 0
                        && bytes
                            .get(cursor + 1)
                            .is_some_and(|byte| byte.eq_ignore_ascii_case(&b'f'))
                        && cursor > 0
                        && bytes[cursor - 1].is_ascii_whitespace()
                        && bytes.get(cursor + 2).is_some_and(u8::is_ascii_whitespace) =>
                {
                    return Some((argument[..cursor].trim(), argument[cursor + 2..].trim()));
                }
                _ => {}
            }
        }
        cursor += character.len_utf8();
    }
    None
}
