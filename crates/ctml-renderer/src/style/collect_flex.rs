pub(super) use super::collect_flex_order::order_nodes;

mod gap;
use super::{
    FlexAlignment, FlexBasis, TextRun, TextStyle, UserSelect, append_text, push_line_break,
};
pub(super) use gap::{FlexGapState, item_alignment};

pub(super) fn effective_flex_basis(style: &TextStyle) -> Option<FlexBasis> {
    let basis = style.flex_basis.clone().unwrap_or(FlexBasis::Auto);
    match basis {
        FlexBasis::Auto => {
            let logical_order = style.size_property_orders.get("inline-size");
            let physical_order = style.size_property_orders.get("width");
            let value = match (logical_order, physical_order) {
                (Some(logical), Some(physical)) if logical > physical => {
                    style.inline_size.as_deref()
                }
                (Some(_), Some(_)) => style.width.as_deref(),
                (Some(_), None) => style.inline_size.as_deref(),
                (None, Some(_)) => style.width.as_deref(),
                (None, None) => style.inline_size.as_deref().or(style.width.as_deref()),
            };
            value.and_then(|value| parse_item_basis(value, style))
        }
        FlexBasis::Content => Some(FlexBasis::Content),
        basis => Some(basis),
    }
}

pub(super) fn effective_flex_main_constraints(
    item_style: &TextStyle,
    container_writing_mode: super::WritingMode,
) -> (Option<String>, Option<String>) {
    let container_is_vertical = matches!(
        container_writing_mode,
        super::WritingMode::VerticalRl | super::WritingMode::VerticalLr
    );
    let item_is_vertical = matches!(
        item_style.writing_mode,
        super::WritingMode::VerticalRl | super::WritingMode::VerticalLr
    );
    let (logical_min, logical_max, logical_min_name, logical_max_name) =
        if container_is_vertical == item_is_vertical {
            (
                item_style.min_inline_size.as_ref(),
                item_style.max_inline_size.as_ref(),
                "min-inline-size",
                "max-inline-size",
            )
        } else {
            (
                item_style.min_block_size.as_ref(),
                item_style.max_block_size.as_ref(),
                "min-block-size",
                "max-block-size",
            )
        };
    let (physical_min, physical_max, physical_min_name, physical_max_name) =
        if container_is_vertical {
            (
                item_style.min_height.as_ref(),
                item_style.max_height.as_ref(),
                "min-height",
                "max-height",
            )
        } else {
            (
                item_style.min_width.as_ref(),
                item_style.max_width.as_ref(),
                "min-width",
                "max-width",
            )
        };
    let select = |logical: Option<&String>,
                  physical: Option<&String>,
                  logical_name: &str,
                  physical_name: &str| {
        let value = match (
            item_style.size_property_orders.get(logical_name),
            item_style.size_property_orders.get(physical_name),
        ) {
            (Some(logical_order), Some(physical_order)) if logical_order > physical_order => {
                logical
            }
            (Some(_), Some(_)) => physical,
            (Some(_), None) => logical,
            (None, Some(_)) => physical,
            (None, None) => logical.or(physical),
        };
        value.cloned()
    };
    (
        select(
            logical_min,
            physical_min,
            logical_min_name,
            physical_min_name,
        ),
        select(
            logical_max,
            physical_max,
            logical_max_name,
            physical_max_name,
        ),
    )
}

fn parse_item_basis(value: &str, style: &TextStyle) -> Option<FlexBasis> {
    if value.eq_ignore_ascii_case("min-content") {
        return Some(FlexBasis::MinContent);
    }
    if value.eq_ignore_ascii_case("max-content") {
        return Some(FlexBasis::MaxContent);
    }
    if let Some(percentage) = ctml_css::parse_css_percentage(value) {
        return (percentage >= 0.0).then_some(FlexBasis::Percentage(percentage / 100.0));
    }
    ctml_css::parse_nonnegative_font_relative_length_with_viewport(
        value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    )
    .filter(|length| length.is_finite())
    .map(FlexBasis::Length)
}

pub(super) fn append_gap(
    style: &TextStyle,
    leading: bool,
    grow_weight: f32,
    shrink_weight: f32,
    basis: Option<FlexBasis>,
    min_main_size: Option<String>,
    max_main_size: Option<String>,
    justify_distribution: bool,
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
    let distribute = justify_distribution
        && style.flex_items_row
        && style.display_block != Some(false)
        && matches!(
            style.justify_content,
            FlexAlignment::SpaceBetween | FlexAlignment::SpaceAround | FlexAlignment::SpaceEvenly
        );
    let distribute_grow =
        style.flex_items_row && grow_weight > 0.0 && flex_main_size(style).is_some();
    let distribute_shrink =
        style.flex_items_row && shrink_weight > 0.0 && flex_main_size(style).is_some();
    let distribute_basis = style.flex_items_row
        && (basis.is_some() || min_main_size.is_some() || max_main_size.is_some());
    let wrap_break = style.flex_items_row && style.flex_wrap;
    if gap.is_none()
        && !distribute
        && !distribute_grow
        && !distribute_shrink
        && !distribute_basis
        && !wrap_break
    {
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
        spacer_style.flex_distributed_gap =
            distribute || distribute_grow || distribute_shrink || distribute_basis;
        spacer_style.flex_distributed_gap_leading =
            leading && style.justify_content == FlexAlignment::SpaceAround;
        spacer_style.flex_distributed_gap_applied = false;
        spacer_style.flex_grow_weight = grow_weight;
        spacer_style.flex_shrink_weight = shrink_weight;
        spacer_style.flex_basis = basis;
        spacer_style.flex_min_main_size = min_main_size;
        spacer_style.flex_max_main_size = max_main_size;
        spacer_style.flex_justify_distributed = distribute;
        spacer_style.flex_wrap_break = wrap_break;
        spacer_style.flex_wrap_gap_px = if wrap_break {
            resolve_wrap_row_gap(style)
        } else {
            0.0
        };
        spacer_style.flex_wrap_gap_applied = false;
        spacer_style.flex_wrap_cross_size_px = if wrap_break {
            explicit_row_cross_size(style)
        } else {
            None
        };
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

pub(super) fn flex_main_size(style: &TextStyle) -> Option<f32> {
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
