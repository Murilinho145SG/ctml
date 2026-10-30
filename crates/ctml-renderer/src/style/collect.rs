use crate::{
    StyleRule, UiNode,
    css_style::{self, ElementIdentity},
};
use ctml_css::{SelectorPosition, SelectorSibling, SelectorTreeElement};
use std::collections::HashMap;

use super::{
    LayoutHeight, LayoutSize, TextRun, TextStyle, UserSelect, append_text, collect_spacers,
    list_marker::format_marker, pseudo, push_line_break, selector_sibling_contexts,
};

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
    focused_path: &[usize],
    keyboard_focus: bool,
    selector_tree: &[SelectorTreeElement],
    inherited: &TextStyle,
    font_metrics: Option<&crate::font_metrics::FontRelativeUnitsContext<'_>>,
    output: &mut Vec<TextRun>,
    last_was_space: &mut bool,
    capitalize_next: &mut bool,
    quote_depth: &mut usize,
    counters: &mut HashMap<String, Vec<i32>>,
) {
    let siblings = selector_sibling_contexts(
        nodes,
        next_node_id,
        hovered_path,
        active_path,
        focused_path,
        keyboard_focus,
    );
    let mut element_index = 0;
    let mut list_item_index = 0;
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
                let mut inherited_style = inherited.clone();
                inherited_style.counter_reset.clear();
                inherited_style.counter_increment.clear();
                inherited_style.counter_set.clear();
                if let Some(font_metrics) = font_metrics {
                    inherited_style.font_relative_units = font_metrics.resolve(inherited);
                }
                let initial_style = css_style::resolve_style_with_selector_tree_context(
                    &inherited_style,
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
                let local_font_units = font_metrics
                    .map(|font_metrics| font_metrics.resolve(&initial_style))
                    .unwrap_or(initial_style.font_relative_units);
                let mut style = css_style::resolve_style_with_selector_tree_context_and_font_units(
                    &inherited_style,
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
                    local_font_units,
                );
                if style.user_select == UserSelect::All {
                    style.user_select_all_owner = Some(sibling.node_id);
                    style.user_select_none = false;
                }
                if let Some((_, language)) = attributes
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case("lang"))
                {
                    style.language = language
                        .as_deref()
                        .filter(|language| !language.trim().is_empty())
                        .map(str::to_owned);
                }
                style.bidi_isolate_direction = if style.unicode_bidi == super::UnicodeBidi::Isolate
                {
                    Some(style.direction)
                } else {
                    inherited.bidi_isolate_direction
                };
                style.opacity *= inherited.opacity;
                if style.display_none {
                    element_index += 1;
                    continue;
                }
                let mut counter_scope = Vec::new();
                for (name, value) in &style.counter_reset {
                    counters.entry(name.clone()).or_default().push(*value);
                    counter_scope.push(name.clone());
                }
                for (name, value) in &style.counter_set {
                    set_counter(counters, &mut counter_scope, name, *value);
                }
                for (name, value) in &style.counter_increment {
                    let values = counters.entry(name.clone()).or_insert_with(|| {
                        counter_scope.push(name.clone());
                        vec![0]
                    });
                    if let Some(current) = values.last_mut() {
                        *current = current.saturating_add(*value);
                    }
                }
                let list_item = style.display_list_item.unwrap_or(tag == "li");
                if list_item {
                    list_item_index += 1;
                }
                let ordered_list_item =
                    list_item && path.last().is_some_and(|parent| parent.tag == "ol");
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
                    quote_depth,
                    counters,
                );
                let mut quote_depth_probe = *quote_depth;
                let has_after_content = pseudo::resolve_generated_content(
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
                    &mut quote_depth_probe,
                    counters,
                )
                .is_some();
                let block = style.display_block.unwrap_or_else(|| is_block(tag));
                let has_block_size = block
                    && (style.width.is_some()
                        || style.min_width.is_some()
                        || style.max_width.is_some()
                        || style.inline_size.is_some()
                        || style.min_inline_size.is_some()
                        || style.max_inline_size.is_some()
                        || style.height.is_some()
                        || style.min_height.is_some()
                        || style.max_height.is_some()
                        || style.block_size.is_some()
                        || style.min_block_size.is_some()
                        || style.max_block_size.is_some()
                        || style.margin.iter().flatten().any(|margin| margin != "0")
                        || style.padding.iter().flatten().any(|padding| padding != "0")
                        || style
                            .border_style
                            .iter()
                            .any(|style| *style != super::BorderLineStyle::None)
                        || style.outline_style != super::BorderLineStyle::None
                        || style.overflow_x != super::Overflow::Visible
                        || style.overflow_y != super::Overflow::Visible);
                if has_block_size {
                    style.layout_sizes.push(LayoutSize {
                        node_id: sibling.node_id,
                        width: style.width.clone(),
                        min_width: style.min_width.clone(),
                        max_width: style.max_width.clone(),
                        inline_size: style.inline_size.clone(),
                        min_inline_size: style.min_inline_size.clone(),
                        max_inline_size: style.max_inline_size.clone(),
                        height: style.height.clone(),
                        min_height: style.min_height.clone(),
                        max_height: style.max_height.clone(),
                        block_size: style.block_size.clone(),
                        min_block_size: style.min_block_size.clone(),
                        max_block_size: style.max_block_size.clone(),
                        border_box: style.border_box,
                        border_width: style.border_width.clone(),
                        border_style: style.border_style,
                        border_color: style.border_color,
                        border_radius: style.border_radius.clone(),
                        outline_width: style.outline_width.clone(),
                        outline_style: style.outline_style,
                        outline_color: style.outline_color,
                        outline_offset: style.outline_offset.clone(),
                        padding: style.padding.clone(),
                        margin: style.margin.clone(),
                        intrinsic_min_content_width: None,
                        intrinsic_max_content_width: None,
                        overflow_x: style.overflow_x,
                        overflow_y: style.overflow_y,
                        size_property_orders: style.size_property_orders.clone(),
                        writing_mode: style.writing_mode,
                        font_size: style.font_size,
                        line_height: style.line_height,
                        font_relative_units: style.font_relative_units,
                        viewport_size: style.viewport_size,
                    });
                }
                if block {
                    push_line_break(output, &style, element_path, last_was_space);
                    *capitalize_next = true;
                }
                path.push(identity.clone());
                element_path.push(sibling.node_id);
                path_attributes.push(attributes.clone());
                path_siblings.push(siblings[..element_index].to_vec());
                path_positions.push(position);
                collect_spacers::append_block_start_spacers(
                    &style,
                    block,
                    !children.is_empty() || before_content.is_some() || has_after_content,
                    element_path,
                    output,
                    last_was_space,
                    capitalize_next,
                );
                if list_item {
                    if let Some(marker) = format_marker(
                        style.list_style_type.as_deref(),
                        ordered_list_item,
                        list_item_index,
                    ) {
                        append_text(
                            &marker,
                            &style,
                            element_path,
                            output,
                            last_was_space,
                            capitalize_next,
                        );
                    }
                }
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
                if block
                    && children.is_empty()
                    && before_content.is_none()
                    && !has_after_content
                    && !list_item
                    && (style.width.is_some()
                        || style.min_width.is_some()
                        || style.max_width.is_some()
                        || style.inline_size.is_some()
                        || style.min_inline_size.is_some()
                        || style.max_inline_size.is_some()
                        || style.height.is_some()
                        || style.min_height.is_some()
                        || style.max_height.is_some()
                        || style.block_size.is_some()
                        || style.min_block_size.is_some()
                        || style.max_block_size.is_some()
                        || style.margin.iter().flatten().any(|margin| margin != "0")
                        || style.padding.iter().flatten().any(|padding| padding != "0")
                        || style
                            .border_style
                            .iter()
                            .any(|style| *style != super::BorderLineStyle::None))
                {
                    let mut spacer_style = style.clone();
                    spacer_style.empty_block_height = Some(LayoutHeight {
                        width: style.width.clone(),
                        min_width: style.min_width.clone(),
                        max_width: style.max_width.clone(),
                        inline_size: style.inline_size.clone(),
                        min_inline_size: style.min_inline_size.clone(),
                        max_inline_size: style.max_inline_size.clone(),
                        height: style.height.clone(),
                        min_height: style.min_height.clone(),
                        max_height: style.max_height.clone(),
                        block_size: style.block_size.clone(),
                        min_block_size: style.min_block_size.clone(),
                        max_block_size: style.max_block_size.clone(),
                        border_box: style.border_box,
                        border_width: style.border_width.clone(),
                        border_style: style.border_style,
                        border_color: style.border_color,
                        padding: style.padding.clone(),
                        margin: style.margin.clone(),
                        size_property_orders: style.size_property_orders.clone(),
                        writing_mode: style.writing_mode,
                        font_size: style.font_size,
                        line_height: style.line_height,
                        font_relative_units: style.font_relative_units,
                        viewport_size: style.viewport_size,
                    });
                    let last_was_space_before = *last_was_space;
                    let capitalize_next_before = *capitalize_next;
                    append_text(
                        "\u{200b}",
                        &spacer_style,
                        element_path,
                        output,
                        last_was_space,
                        capitalize_next,
                    );
                    *last_was_space = last_was_space_before;
                    *capitalize_next = capitalize_next_before;
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
                    focused_path,
                    keyboard_focus,
                    selector_tree,
                    &style,
                    font_metrics,
                    output,
                    last_was_space,
                    capitalize_next,
                    quote_depth,
                    counters,
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
                    quote_depth,
                    counters,
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
                if has_block_size
                    && (!children.is_empty() || before_content.is_some() || has_after_content)
                {
                    let last_was_space_before = *last_was_space;
                    let capitalize_next_before = *capitalize_next;
                    let mut end_marker_style = style.clone();
                    end_marker_style.layout_sizes.clear();
                    end_marker_style.layout_end_sizes.clear();
                    append_text(
                        "\u{200b}",
                        &end_marker_style,
                        element_path,
                        output,
                        last_was_space,
                        capitalize_next,
                    );
                    *last_was_space = last_was_space_before;
                    *capitalize_next = capitalize_next_before;
                    if let Some(layout_size) = style
                        .layout_sizes
                        .iter()
                        .find(|size| size.node_id == sibling.node_id)
                        && let Some(last_run) = output.last_mut()
                    {
                        last_run.style.layout_end_sizes.push(layout_size.clone());
                    }
                }
                path.pop();
                element_path.pop();
                path_attributes.pop();
                path_siblings.pop();
                path_positions.pop();
                for name in counter_scope.into_iter().rev() {
                    if let Some(values) = counters.get_mut(&name) {
                        values.pop();
                        if values.is_empty() {
                            counters.remove(&name);
                        }
                    }
                }
                element_index += 1;
                if block {
                    push_line_break(output, &style, element_path, last_was_space);
                    *capitalize_next = true;
                }
            }
        }
    }
}

fn set_counter(
    counters: &mut HashMap<String, Vec<i32>>,
    scope: &mut Vec<String>,
    name: &str,
    value: i32,
) {
    let values = counters.entry(name.to_owned()).or_insert_with(|| {
        scope.push(name.to_owned());
        vec![value]
    });
    if let Some(current) = values.last_mut() {
        *current = value;
    }
}
