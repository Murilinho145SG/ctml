use super::{
    SelectorTreeElement, list::split_selector_list, match_selector_chain, selector_compounds,
};

pub(super) fn selector_has_matches(
    argument: &str,
    anchor_node_id: usize,
    selector_tree: &[SelectorTreeElement],
) -> bool {
    let Some(anchor) = selector_tree
        .iter()
        .find(|element| element.node_id == anchor_node_id)
    else {
        return false;
    };
    split_selector_list(argument).into_iter().any(|relative| {
        let relative = relative.trim();
        if relative.is_empty() {
            return false;
        }
        let first = relative.chars().next().unwrap();
        let relation = match first {
            '>' | '+' | '~' => Some(first),
            _ => None,
        };
        let selector = if relation.is_some() {
            relative[first.len_utf8()..].trim()
        } else {
            relative
        };
        if selector.is_empty() {
            return false;
        }
        let anchored_selector = relation.map_or_else(
            || format!(":scope {selector}"),
            |relation| format!(":scope {relation} {selector}"),
        );
        selector_tree.iter().any(|candidate| {
            let within_relative_scope = match relation {
                None | Some('>') => is_descendant(candidate, anchor_node_id),
                Some('+' | '~') => is_in_following_sibling_subtree(
                    candidate,
                    anchor,
                    selector_tree,
                    relation == Some('+'),
                ),
                _ => false,
            };
            within_relative_scope
                && matches_tree_element_with_scope(
                    &anchored_selector,
                    candidate,
                    selector_tree,
                    anchor_node_id,
                )
        })
    })
}

fn matches_tree_element_with_scope(
    selector: &str,
    current: &SelectorTreeElement,
    selector_tree: &[SelectorTreeElement],
    scope_node_id: usize,
) -> bool {
    let ancestors = current
        .ancestors
        .iter()
        .map(|ancestor| ancestor.element.clone())
        .collect::<Vec<_>>();
    let ancestor_attributes = current
        .ancestors
        .iter()
        .map(|ancestor| ancestor.attributes.clone())
        .collect::<Vec<_>>();
    let siblings_before_ancestors = current
        .ancestors
        .iter()
        .map(|ancestor| ancestor.siblings_before.clone())
        .collect::<Vec<_>>();
    let ancestor_positions = current
        .ancestors
        .iter()
        .map(|ancestor| ancestor.position)
        .collect::<Vec<_>>();
    let Some(compounds) = selector_compounds(selector) else {
        return false;
    };
    match_selector_chain(
        &compounds,
        compounds.len().checked_sub(1),
        &current.element,
        &current.attributes,
        &ancestors,
        &ancestor_attributes,
        &current.siblings_before,
        &siblings_before_ancestors,
        current.position,
        &ancestor_positions,
        selector_tree,
        Some(scope_node_id),
    )
}

fn is_descendant(candidate: &SelectorTreeElement, ancestor_node_id: usize) -> bool {
    candidate
        .ancestors
        .iter()
        .any(|ancestor| ancestor.position.node_id == ancestor_node_id)
}

fn is_in_following_sibling_subtree(
    candidate: &SelectorTreeElement,
    anchor: &SelectorTreeElement,
    selector_tree: &[SelectorTreeElement],
    adjacent: bool,
) -> bool {
    let Some(parent_id) = anchor.parent_id else {
        return false;
    };
    let mut sibling_root = candidate;
    while sibling_root.parent_id != Some(parent_id) {
        let Some(parent) = sibling_root
            .parent_id
            .and_then(|parent_id| selector_tree.iter().find(|item| item.node_id == parent_id))
        else {
            return false;
        };
        sibling_root = parent;
    }
    if adjacent {
        sibling_root.position.child_index == anchor.position.child_index + 1
    } else {
        sibling_root.position.child_index > anchor.position.child_index
    }
}
