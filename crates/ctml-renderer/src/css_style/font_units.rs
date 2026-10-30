use super::apply_matching_styles_with_inline_context;
use crate::{StyleRule, style::TextStyle};
use ctml_css::{ElementIdentity, SelectorPosition, SelectorSibling, SelectorTreeElement};

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_style_with_selector_tree_context(
    inherited: &TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    siblings_before_current: &[SelectorSibling],
    siblings_before_ancestors: &[Vec<SelectorSibling>],
    current_position: SelectorPosition,
    ancestor_positions: &[SelectorPosition],
    selector_tree: &[SelectorTreeElement],
    inline_style: Option<&str>,
    stylesheet: &[StyleRule],
) -> TextStyle {
    resolve_style_with_selector_tree_context_and_font_units(
        inherited,
        current,
        ancestors,
        attributes,
        ancestor_attributes,
        siblings_before_current,
        siblings_before_ancestors,
        current_position,
        ancestor_positions,
        selector_tree,
        inline_style,
        stylesheet,
        inherited.font_relative_units,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_style_with_selector_tree_context_and_font_units(
    inherited: &TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    siblings_before_current: &[SelectorSibling],
    siblings_before_ancestors: &[Vec<SelectorSibling>],
    current_position: SelectorPosition,
    ancestor_positions: &[SelectorPosition],
    selector_tree: &[SelectorTreeElement],
    inline_style: Option<&str>,
    stylesheet: &[StyleRule],
    local_font_units: ctml_css::FontRelativeUnits,
) -> TextStyle {
    let mut style = inherited.clone();
    style.font_relative_units = local_font_units;
    style.width = None;
    style.min_width = None;
    style.max_width = None;
    style.inline_size = None;
    style.min_inline_size = None;
    style.max_inline_size = None;
    style.height = None;
    style.min_height = None;
    style.max_height = None;
    style.block_size = None;
    style.min_block_size = None;
    style.max_block_size = None;
    style.size_property_orders.clear();
    style.current_declaration_order = 0;
    style.border_box = false;
    style.padding = std::array::from_fn(|_| None);
    style.logical_padding = std::array::from_fn(|_| None);
    style.physical_padding_orders = [0; 4];
    style.logical_padding_orders = [0; 4];
    style.border_width = std::array::from_fn(|_| None);
    style.logical_border_width = std::array::from_fn(|_| None);
    style.physical_border_width_orders = [0; 4];
    style.logical_border_width_orders = [0; 4];
    style.border_style = [crate::style::BorderLineStyle::None; 4];
    style.logical_border_style = std::array::from_fn(|_| None);
    style.physical_border_style_orders = [0; 4];
    style.logical_border_style_orders = [0; 4];
    style.border_color = std::array::from_fn(|_| None);
    style.logical_border_color = std::array::from_fn(|_| None);
    style.physical_border_color_orders = [0; 4];
    style.logical_border_color_orders = [0; 4];
    style.border_radius = std::array::from_fn(|_| [None, None]);
    style.border_radius_orders = [0; 4];
    style.margin = std::array::from_fn(|_| None);
    style.logical_margin = std::array::from_fn(|_| None);
    style.physical_margin_orders = [0; 4];
    style.logical_margin_orders = [0; 4];
    style.empty_block_height = None;
    style.layout_end_sizes.clear();
    style.user_select = crate::style::UserSelect::Auto;
    style.flow_spacer = false;
    apply_matching_styles_with_inline_context(
        &mut style,
        current,
        ancestors,
        attributes,
        ancestor_attributes,
        siblings_before_current,
        siblings_before_ancestors,
        current_position,
        ancestor_positions,
        selector_tree,
        inline_style,
        Some(inherited),
        stylesheet,
    );
    super::resolve_logical_padding(&mut style);
    super::resolve_logical_margin(&mut style);
    super::resolve_logical_borders(&mut style);
    style
}
