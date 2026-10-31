use super::{FlexAlignment, TextRun, TextStyle, UserSelect, append_text, push_line_break};
use crate::{StyleRule, UiNode, css_style};
use ctml_css::{SelectorPosition, SelectorSibling, SelectorTreeElement};

#[derive(Default)]
pub(super) struct FlexGapState {
    item_count: usize,
    last_item_id: Option<usize>,
    last_item_grow: f32,
    total_grow: f32,
}

pub(super) fn item_alignment(
    style: &TextStyle,
    inherited: &TextStyle,
    block: bool,
) -> Option<FlexAlignment> {
    if style.display_flex && block {
        if style.flex_direction_column {
            Some(style.align_items.unwrap_or(FlexAlignment::Stretch))
        } else {
            Some(visual_justify(
                style.justify_content,
                style.flex_direction_reverse,
            ))
        }
    } else if inherited.flex_items_row {
        Some(visual_justify(
            inherited.justify_content,
            inherited.flex_direction_reverse,
        ))
    } else if inherited.flex_items_column {
        Some(
            style
                .align_self
                .or(inherited.align_items)
                .unwrap_or(FlexAlignment::Stretch),
        )
    } else {
        inherited.flex_item_alignment
    }
}

fn visual_justify(alignment: FlexAlignment, reverse: bool) -> FlexAlignment {
    if !reverse {
        return alignment;
    }
    match alignment {
        FlexAlignment::FlexStart => FlexAlignment::FlexEnd,
        FlexAlignment::FlexEnd => FlexAlignment::FlexStart,
        FlexAlignment::Start
        | FlexAlignment::End
        | FlexAlignment::Center
        | FlexAlignment::Stretch
        | FlexAlignment::SpaceBetween
        | FlexAlignment::SpaceAround
        | FlexAlignment::SpaceEvenly => alignment,
    }
}

impl FlexGapState {
    pub(super) fn append_text(
        &mut self,
        style: &TextStyle,
        item_id: usize,
        text: &str,
        element_path: &[usize],
        output: &mut Vec<TextRun>,
        last_was_space: &mut bool,
        capitalize_next: &mut bool,
    ) {
        self.before_item(
            style,
            item_id,
            !text.chars().all(char::is_whitespace),
            0.0,
            element_path,
            output,
            last_was_space,
            capitalize_next,
        );
        append_text(
            text,
            style,
            element_path,
            output,
            last_was_space,
            capitalize_next,
        );
    }

    pub(super) fn before_element(
        &mut self,
        style: &TextStyle,
        item_id: usize,
        grow: f32,
        element_path: &[usize],
        output: &mut Vec<TextRun>,
        last_was_space: &mut bool,
        capitalize_next: &mut bool,
    ) {
        self.before_item(
            style,
            item_id,
            true,
            grow,
            element_path,
            output,
            last_was_space,
            capitalize_next,
        );
    }

    fn before_item(
        &mut self,
        style: &TextStyle,
        item_id: usize,
        is_item: bool,
        grow: f32,
        element_path: &[usize],
        output: &mut Vec<TextRun>,
        last_was_space: &mut bool,
        capitalize_next: &mut bool,
    ) {
        if !is_item || !(style.flex_items_row || style.flex_items_column) {
            return;
        }
        if self.last_item_id == Some(item_id) {
            return;
        }
        if self.item_count == 0
            && style.flex_items_row
            && matches!(
                style.justify_content,
                FlexAlignment::SpaceAround | FlexAlignment::SpaceEvenly
            )
        {
            append_gap(
                style,
                true,
                0.0,
                element_path,
                output,
                last_was_space,
                capitalize_next,
            );
        }
        if self.item_count > 0 {
            append_gap(
                style,
                false,
                self.last_item_grow,
                element_path,
                output,
                last_was_space,
                capitalize_next,
            );
        }
        self.item_count += 1;
        self.last_item_id = Some(item_id);
        self.last_item_grow = grow;
        self.total_grow += grow;
    }

    pub(super) fn finish(
        &self,
        style: &TextStyle,
        element_path: &[usize],
        output: &mut Vec<TextRun>,
        last_was_space: &mut bool,
        capitalize_next: &mut bool,
    ) {
        if self.item_count == 0 || !style.flex_items_row {
            return;
        }
        if self.total_grow > 0.0 {
            if self.last_item_grow > 0.0 {
                append_gap(
                    style,
                    false,
                    self.last_item_grow,
                    element_path,
                    output,
                    last_was_space,
                    capitalize_next,
                );
            }
        } else if style.justify_content == FlexAlignment::SpaceEvenly {
            append_gap(
                style,
                false,
                0.0,
                element_path,
                output,
                last_was_space,
                capitalize_next,
            );
        }
    }
}

pub(super) fn append_gap(
    style: &TextStyle,
    leading: bool,
    grow_weight: f32,
    element_path: &[usize],
    output: &mut Vec<TextRun>,
    last_was_space: &mut bool,
    capitalize_next: &mut bool,
) {
    let gap = if style.flex_items_row {
        style.column_gap.as_deref()
    } else if style.flex_items_column {
        style.row_gap.as_deref()
    } else {
        None
    };
    let distribute = style.flex_items_row
        && style.display_block != Some(false)
        && matches!(
            style.justify_content,
            FlexAlignment::SpaceBetween | FlexAlignment::SpaceAround | FlexAlignment::SpaceEvenly
        );
    let distribute_grow =
        style.flex_items_row && grow_weight > 0.0 && flex_main_size(style).is_some();
    let wrap_break = style.flex_items_row && style.flex_wrap;
    if gap.is_none() && !distribute && !distribute_grow && !wrap_break {
        return;
    }
    let percentage_basis = flex_main_size(style);
    let resolve_length = |component: &str| {
        let percentage = || {
            let percentage = ctml_css::parse_css_percentage(component)?;
            (percentage >= 0.0).then_some(percentage_basis.unwrap_or_default() * percentage / 100.0)
        };
        percentage().or_else(|| {
            ctml_css::parse_nonnegative_font_relative_length_with_viewport(
                component,
                style.font_size,
                style.line_height,
                style.font_relative_units,
                style.viewport_size,
            )
        })
    };
    let gap_px = gap
        .and_then(|gap| {
            ctml_css::parse_math_expression(gap, resolve_length).or_else(|| resolve_length(gap))
        })
        .filter(|gap| *gap > 0.0)
        .unwrap_or_default();

    let mut spacer_style = style.clone();
    spacer_style.font_size = 1.0;
    spacer_style.font_scale = 1.0;
    spacer_style.line_height = 1.0;
    spacer_style.line_height_multiplier = None;
    spacer_style.line_height_normal = false;
    spacer_style.inline_box = true;
    spacer_style.pointer_events_none = true;
    spacer_style.cursor_hidden = true;
    spacer_style.user_select = UserSelect::None;
    spacer_style.user_select_none = true;
    spacer_style.user_select_all_owner = None;

    if style.flex_items_row {
        spacer_style.letter_spacing = gap_px;
        spacer_style.letter_spacing_px = true;
        spacer_style.flex_distributed_gap = distribute || distribute_grow;
        spacer_style.flex_distributed_gap_leading =
            leading && style.justify_content == FlexAlignment::SpaceAround;
        spacer_style.flex_distributed_gap_applied = false;
        spacer_style.flex_grow_weight = grow_weight;
        spacer_style.flex_wrap_break = wrap_break;
        spacer_style.flex_wrap_gap_px = if wrap_break {
            resolve_wrap_row_gap(style)
        } else {
            0.0
        };
        spacer_style.flex_wrap_gap_applied = false;
    } else {
        spacer_style.line_height = gap_px;
        spacer_style.flow_spacer = true;
    }

    let previous_space = *last_was_space;
    let previous_capitalize = *capitalize_next;
    append_text(
        "\u{200b}",
        &spacer_style,
        element_path,
        output,
        last_was_space,
        capitalize_next,
    );
    if style.flex_items_column {
        push_line_break(output, &spacer_style, element_path, last_was_space);
    }
    *last_was_space = previous_space;
    *capitalize_next = previous_capitalize;
}

fn resolve_wrap_row_gap(style: &TextStyle) -> f32 {
    let Some(gap) = style.row_gap.as_deref() else {
        return 0.0;
    };
    let percentage_basis = explicit_row_cross_size(style).unwrap_or_default();
    let resolve = |component: &str| {
        if let Some(percentage) = ctml_css::parse_css_percentage(component) {
            return (percentage >= 0.0).then_some(percentage_basis * percentage / 100.0);
        }
        ctml_css::parse_nonnegative_font_relative_length_with_viewport(
            component,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
    };
    ctml_css::parse_math_expression(gap, resolve)
        .or_else(|| resolve(gap))
        .filter(|gap| gap.is_finite() && *gap > 0.0)
        .unwrap_or_default()
}

fn explicit_row_cross_size(style: &TextStyle) -> Option<f32> {
    let (physical_size, physical_property) = match style.writing_mode {
        super::WritingMode::HorizontalTb => (&style.height, "height"),
        super::WritingMode::VerticalRl | super::WritingMode::VerticalLr => (&style.width, "width"),
    };
    let physical_order = style.size_property_orders.get(physical_property);
    let block_size_order = style.size_property_orders.get("block-size");
    let value = match (block_size_order, physical_order) {
        (Some(block_order), Some(physical_order)) if block_order > physical_order => {
            style.block_size.as_deref()
        }
        (Some(_), Some(_)) => physical_size.as_deref(),
        (Some(_), None) => style.block_size.as_deref(),
        (None, Some(_)) => physical_size.as_deref(),
        (None, None) => style.block_size.as_deref().or(physical_size.as_deref()),
    }?;
    let resolve = |component: &str| {
        if ctml_css::parse_css_percentage(component).is_some() {
            return None;
        }
        ctml_css::parse_nonnegative_font_relative_length_with_viewport(
            component,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
    };
    ctml_css::parse_math_expression(value, resolve).or_else(|| resolve(value))
}

fn flex_main_size(style: &TextStyle) -> Option<f32> {
    let (declared_size, viewport_size) = if style.flex_items_row {
        (
            style.width.as_deref().or(style.inline_size.as_deref()),
            style.viewport_size.width,
        )
    } else {
        (
            style.height.as_deref().or(style.block_size.as_deref()),
            style.viewport_size.height,
        )
    };
    let viewport_derived_main_size = style.flex_items_row && style.display_block != Some(false);
    let Some(declared_size) = declared_size else {
        return viewport_derived_main_size.then_some(viewport_size);
    };
    if declared_size.eq_ignore_ascii_case("auto") {
        return viewport_derived_main_size.then_some(viewport_size);
    }
    let resolve = |value: &str| {
        ctml_css::parse_nonnegative_font_relative_length_with_viewport(
            value,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
        .or_else(|| {
            let percentage = ctml_css::parse_css_percentage(value)?;
            (percentage >= 0.0).then_some(viewport_size * percentage / 100.0)
        })
    };
    resolve(declared_size).or_else(|| ctml_css::parse_math_expression(declared_size, resolve))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn order_nodes(
    nodes: &[UiNode],
    siblings: &[SelectorSibling],
    inherited: &super::TextStyle,
    stylesheet: &[StyleRule],
    ancestors: &[css_style::ElementIdentity],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    siblings_before_ancestors: &[Vec<SelectorSibling>],
    ancestor_positions: &[SelectorPosition],
    selector_tree: &[SelectorTreeElement],
) -> Vec<(usize, Option<usize>, usize)> {
    let is_flex_container = inherited.flex_items_row || inherited.flex_items_column;
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
        if inherited.flex_direction_reverse {
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
