use crate::{
    StyleRule,
    css_style::{self, ElementIdentity},
    style::TextStyle,
};
use ctml_css::{SelectorPosition, SelectorSibling, SelectorTreeElement};

pub(super) fn is_block(tag: &str) -> bool {
    matches!(
        tag,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "div"
            | "footer"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "li"
            | "main"
            | "nav"
            | "ol"
            | "p"
            | "section"
            | "ul"
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_block(
    style: &TextStyle,
    inherited: &TextStyle,
    current_tag: &str,
    siblings: &[SelectorSibling],
    element_index: usize,
    ancestors: &[ElementIdentity],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    siblings_before_ancestors: &[Vec<SelectorSibling>],
    ancestor_positions: &[SelectorPosition],
    selector_tree: &[SelectorTreeElement],
    stylesheet: &[StyleRule],
) -> (bool, bool) {
    let flex_row_item = inherited.flex_items_row;
    let flex_column_item = inherited.flex_items_column;
    let run_in_inline = !flex_row_item
        && !flex_column_item
        && style.display_run_in
        && siblings.get(element_index + 1).is_some_and(|next| {
            let inline_style = next
                .attributes
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("style"))
                .and_then(|(_, value)| value.as_deref());
            let next_style = css_style::resolve_style_with_selector_tree_context(
                inherited,
                &next.element,
                ancestors,
                &next.attributes,
                ancestor_attributes,
                &siblings[..=element_index],
                siblings_before_ancestors,
                next.position,
                ancestor_positions,
                selector_tree,
                inline_style,
                stylesheet,
            );
            !next_style.display_none
                && next_style
                    .display_block
                    .unwrap_or_else(|| is_block(&next.element.tag))
        });
    let block = if flex_row_item {
        false
    } else if flex_column_item {
        true
    } else if style.display_run_in {
        !run_in_inline
    } else {
        style.display_block.unwrap_or_else(|| is_block(current_tag))
    };
    (block, run_in_inline)
}
