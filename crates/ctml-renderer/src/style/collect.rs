use super::{
    LayoutHeight, TextRun, TextStyle, UserSelect, append_text, collect_counters, collect_display,
    collect_flex, collect_helpers, collect_layout, collect_spacers, list_marker::format_marker,
    pseudo, push_line_break, selector_sibling_contexts,
};
use crate::{StyleRule, UiNode, css_style::ElementIdentity};
use ctml_css::{SelectorPosition, SelectorSibling, SelectorTreeElement};
use std::collections::{HashMap, HashSet};

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
    opacity_groups: &mut HashMap<usize, f32>,
    mix_blend_modes: &mut HashMap<usize, super::BackgroundBlendMode>,
    isolation_groups: &mut HashSet<usize>,
    transition_styles: &mut HashMap<usize, super::ElementTransitionStyle>,
    page_writing_mode: super::WritingMode,
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
    let ordered_nodes = collect_flex::order_nodes(
        nodes,
        &siblings,
        inherited,
        stylesheet,
        path,
        path_attributes,
        path_siblings,
        path_positions,
        selector_tree,
    );
    let (mut element_index, mut list_item_index) = (0, 0);
    let mut flex_gap_state = collect_flex::FlexGapState::default();
    let mut run_in_before_block = false;
    for (node_index, source_element_index, flex_item_id) in ordered_nodes {
        let node = &nodes[node_index];
        if let Some(source_element_index) = source_element_index {
            element_index = source_element_index;
        }
        match node {
            UiNode::Text(text) => flex_gap_state.append_text(
                inherited,
                flex_item_id,
                text,
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
                let identity = collect_helpers::element_identity(tag, class, id);
                let inline_style = collect_helpers::inline_style(attributes);
                let mut style = collect_helpers::resolve_element_style(
                    inherited,
                    &identity,
                    attributes,
                    path,
                    path_attributes,
                    &siblings[..element_index],
                    path_siblings,
                    position,
                    path_positions,
                    selector_tree,
                    inline_style,
                    stylesheet,
                    font_metrics,
                );
                style.content_skipped_by_ancestor = inherited.content_skipped;
                style.content_skipped =
                    inherited.content_skipped || style.content_visibility_hidden;
                if style.content_visibility_hidden {
                    style.paint_contained = true;
                    style.layout_contained = true;
                    style.size_contained = true;
                    style.inline_size_contained = true;
                    style.style_contained = true;
                } else if style.content_visibility_auto {
                    style.paint_contained = true;
                    style.layout_contained = true;
                    style.style_contained = true;
                }
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
                opacity_groups.insert(sibling.node_id, style.opacity);
                mix_blend_modes.insert(sibling.node_id, style.mix_blend_mode);
                if style.isolation {
                    isolation_groups.insert(sibling.node_id);
                }
                transition_styles.insert(
                    sibling.node_id,
                    super::ElementTransitionStyle {
                        opacity: style.transition_opacity.then_some(super::TransitionTiming {
                            duration_ms: style.transition_duration_ms,
                            delay_ms: style.transition_delay_ms,
                            easing: style.transition_easing,
                        }),
                        color: style.transition_color.then_some(super::TransitionTiming {
                            duration_ms: style.transition_color_duration_ms,
                            delay_ms: style.transition_color_delay_ms,
                            easing: style.transition_color_easing,
                        }),
                        color_value: style.color,
                    },
                );
                style.opacity *= inherited.opacity;
                if style.display_none {
                    continue;
                }
                flex_gap_state.before_element(
                    inherited,
                    flex_item_id,
                    style.flex_grow,
                    style.flex_shrink,
                    collect_flex::effective_flex_basis(&style),
                    element_path,
                    output,
                    last_was_space,
                    capitalize_next,
                );
                let scoped_counters = style.style_contained.then(|| counters.clone());
                let scoped_quote_depth = style.style_contained.then_some(*quote_depth);
                let mut counter_scope = Vec::new();
                for (name, value) in &style.counter_reset {
                    counters.entry(name.clone()).or_default().push(*value);
                    counter_scope.push(name.clone());
                }
                for (name, value) in &style.counter_set {
                    collect_counters::set_counter(counters, &mut counter_scope, name, *value);
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
                if list_item && !style.content_skipped {
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
                let (block, run_in_inline) = collect_display::resolve_block(
                    &style,
                    inherited,
                    tag,
                    &siblings,
                    element_index,
                    path,
                    path_attributes,
                    path_siblings,
                    path_positions,
                    selector_tree,
                    stylesheet,
                );
                style.flex_items_row = style.display_flex && !style.flex_direction_column;
                style.flex_items_column = style.display_flex && style.flex_direction_column;
                style.flex_item_no_wrap = inherited.flex_item_no_wrap
                    || (inherited.flex_items_row && !inherited.flex_wrap);
                style.flex_item_alignment = collect_flex::item_alignment(&style, inherited, block);
                style.inline_box = !block;
                let empty_box = children.is_empty()
                    && before_content.is_none()
                    && !has_after_content
                    && !list_item;
                let has_block_size =
                    collect_layout::register_layout_size(&mut style, sibling.node_id, block);
                if block && !run_in_before_block {
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
                    empty_box,
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
                if let Some(content) = &before_content
                    && !style.content_skipped
                {
                    append_text(
                        &content.text,
                        &content.style,
                        element_path,
                        output,
                        last_was_space,
                        capitalize_next,
                    );
                }
                let contained_block_fallback =
                    if page_writing_mode == super::WritingMode::HorizontalTb {
                        style.contain_intrinsic_height.as_deref()
                    } else {
                        style.contain_intrinsic_width.as_deref()
                    };
                let inline_containment_is_block_axis = style.inline_size_contained
                    && (style.writing_mode != super::WritingMode::HorizontalTb)
                        != (page_writing_mode != super::WritingMode::HorizontalTb);
                if block
                    && empty_box
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
                        || style.aspect_ratio.is_some()
                        || !style.background_layers.is_empty()
                        || !style.background_images.is_empty()
                        || style.border_image_source.is_some()
                        || ((style.size_contained
                            || style.content_visibility_auto
                            || inline_containment_is_block_axis)
                            && contained_block_fallback.is_some())
                        || !style.box_shadows.is_empty()
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
                        aspect_ratio: style.aspect_ratio,
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
                        size_contained: style.size_contained,
                        inline_size_contained: style.inline_size_contained,
                        contain_intrinsic_width: style.contain_intrinsic_width.clone(),
                        contain_intrinsic_height: style.contain_intrinsic_height.clone(),
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
                if tag == "br" {
                    push_line_break(output, &style, element_path, last_was_space);
                    *capitalize_next = true;
                } else {
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
                        opacity_groups,
                        mix_blend_modes,
                        isolation_groups,
                        transition_styles,
                        page_writing_mode,
                        font_metrics,
                        output,
                        last_was_space,
                        capitalize_next,
                        quote_depth,
                        counters,
                    );
                }
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
                if let Some(content) = &after_content
                    && !style.content_skipped
                {
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
                if let Some(scoped_counters) = scoped_counters {
                    *counters = scoped_counters;
                }
                if let Some(scoped_quote_depth) = scoped_quote_depth {
                    *quote_depth = scoped_quote_depth;
                }
                if block {
                    push_line_break(output, &style, element_path, last_was_space);
                    *capitalize_next = true;
                }
                run_in_before_block = run_in_inline;
            }
        }
    }
    flex_gap_state.finish(
        inherited,
        element_path,
        output,
        last_was_space,
        capitalize_next,
    );
}
