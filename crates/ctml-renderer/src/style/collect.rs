use crate::{
    StyleRule, UiNode,
    css_style::{self, ElementIdentity},
};
use ctml_css::{SelectorPosition, SelectorSibling, SelectorTreeElement};

use super::{
    TextRun, TextStyle, append_text, is_block, pseudo, push_line_break, selector_sibling_contexts,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_runs(
    nodes: &[UiNode],
    stylesheet: &[StyleRule],
    path: &mut Vec<ElementIdentity>,
    path_attributes: &mut Vec<Vec<(String, Option<String>)>>,
    path_siblings: &mut Vec<Vec<SelectorSibling>>,
    path_positions: &mut Vec<SelectorPosition>,
    element_path: &mut Vec<usize>,
    next_node_id: &mut usize,
    hovered_path: &[usize],
    active_path: &[usize],
    selector_tree: &[SelectorTreeElement],
    inherited: &TextStyle,
    output: &mut Vec<TextRun>,
    last_was_space: &mut bool,
    capitalize_next: &mut bool,
) {
    let siblings = selector_sibling_contexts(nodes, next_node_id, hovered_path, active_path);
    let mut element_index = 0;
    for node in nodes {
        match node {
            UiNode::Text(text) => append_text(
                text,
                inherited,
                element_path,
                output,
                last_was_space,
                capitalize_next,
            ),
            UiNode::Element {
                tag,
                class,
                id,
                attributes,
                children,
            } => {
                let sibling = &siblings[element_index];
                let position = sibling.position;
                let identity = ElementIdentity {
                    tag: tag.clone(),
                    class: class.clone(),
                    id: id.clone(),
                };
                let inline_style = attributes
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case("style"))
                    .and_then(|(_, value)| value.as_deref());
                let style = css_style::resolve_style_with_selector_tree_context(
                    inherited,
                    &identity,
                    path,
                    attributes,
                    path_attributes,
                    &siblings[..element_index],
                    path_siblings,
                    position,
                    path_positions,
                    selector_tree,
                    inline_style,
                    stylesheet,
                );
                let before_content = pseudo::resolve_generated_content(
                    "before",
                    &style,
                    &identity,
                    path,
                    attributes,
                    path_attributes,
                    &siblings[..element_index],
                    path_siblings,
                    position,
                    path_positions,
                    selector_tree,
                    stylesheet,
                );
                let after_content = pseudo::resolve_generated_content(
                    "after",
                    &style,
                    &identity,
                    path,
                    attributes,
                    path_attributes,
                    &siblings[..element_index],
                    path_siblings,
                    position,
                    path_positions,
                    selector_tree,
                    stylesheet,
                );
                let block = is_block(tag);
                if block {
                    push_line_break(output, &style, element_path, last_was_space);
                    *capitalize_next = true;
                }
                path.push(identity);
                element_path.push(sibling.node_id);
                path_attributes.push(attributes.clone());
                path_siblings.push(siblings[..element_index].to_vec());
                path_positions.push(position);
                if let Some(content) = &before_content {
                    append_text(
                        &content.text,
                        &content.style,
                        element_path,
                        output,
                        last_was_space,
                        capitalize_next,
                    );
                }
                collect_runs(
                    children,
                    stylesheet,
                    path,
                    path_attributes,
                    path_siblings,
                    path_positions,
                    element_path,
                    next_node_id,
                    hovered_path,
                    active_path,
                    selector_tree,
                    &style,
                    output,
                    last_was_space,
                    capitalize_next,
                );
                if let Some(content) = &after_content {
                    append_text(
                        &content.text,
                        &content.style,
                        element_path,
                        output,
                        last_was_space,
                        capitalize_next,
                    );
                }
                path.pop();
                element_path.pop();
                path_attributes.pop();
                path_siblings.pop();
                path_positions.pop();
                element_index += 1;
                if block {
                    push_line_break(output, &style, element_path, last_was_space);
                    *capitalize_next = true;
                }
            }
        }
    }
}
