use super::TextStyle;
use crate::{StyleRule, UiNode, css_style};
use ctml_css::{SelectorPosition, SelectorSibling, SelectorTreeElement};

#[allow(clippy::too_many_arguments)]
pub(super) fn order_nodes(
    nodes: &[UiNode],
    siblings: &[SelectorSibling],
    inherited: &TextStyle,
    stylesheet: &[StyleRule],
    ancestors: &[css_style::ElementIdentity],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    siblings_before_ancestors: &[Vec<SelectorSibling>],
    ancestor_positions: &[SelectorPosition],
    selector_tree: &[SelectorTreeElement],
) -> Vec<(usize, Option<usize>, usize)> {
    let is_flex_container =
        inherited.flex_items_row || inherited.flex_items_column || inherited.display_grid;
    let mut element_index = 0;
    let mut item_id = 0;
    let mut previous_was_text = false;
    let mut ordered_nodes = nodes
        .iter()
        .enumerate()
        .map(|(node_index, node)| {
            let (source_element_index, order) = if matches!(node, UiNode::Element { .. }) {
                item_id += 1;
                previous_was_text = false;
                let sibling = &siblings[element_index];
                let inline_style = sibling
                    .attributes
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case("style"))
                    .and_then(|(_, value)| value.as_deref());
                let order = if is_flex_container {
                    css_style::resolve_style_with_selector_tree_context(
                        inherited,
                        &sibling.element,
                        ancestors,
                        &sibling.attributes,
                        ancestor_attributes,
                        &siblings[..element_index],
                        siblings_before_ancestors,
                        sibling.position,
                        ancestor_positions,
                        selector_tree,
                        inline_style,
                        stylesheet,
                    )
                    .flex_order
                } else {
                    0
                };
                let source_element_index = Some(element_index);
                element_index += 1;
                (source_element_index, order)
            } else {
                if !previous_was_text {
                    item_id += 1;
                }
                previous_was_text = true;
                (None, 0)
            };
            (node_index, source_element_index, item_id, order)
        })
        .collect::<Vec<_>>();

    if is_flex_container {
        ordered_nodes.sort_by_key(|(_, _, _, order)| *order);
        if !inherited.display_grid && inherited.flex_direction_reverse {
            reverse_item_groups(&mut ordered_nodes);
        }
    }

    ordered_nodes
        .into_iter()
        .map(|(node_index, source_element_index, item_id, _)| {
            (node_index, source_element_index, item_id)
        })
        .collect()
}

fn reverse_item_groups(nodes: &mut Vec<(usize, Option<usize>, usize, i32)>) {
    let mut groups = Vec::<Vec<(usize, Option<usize>, usize, i32)>>::new();
    for node in nodes.drain(..) {
        if groups.last().is_some_and(|group| group[0].2 == node.2) {
            groups.last_mut().expect("group was just found").push(node);
        } else {
            groups.push(vec![node]);
        }
    }
    nodes.extend(groups.into_iter().rev().flatten());
}
