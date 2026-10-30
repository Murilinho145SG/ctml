use super::{BorderLineStyle, LayoutSize, Overflow, TextStyle};

pub(super) fn register_layout_size(style: &mut TextStyle, node_id: usize, block: bool) -> bool {
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
            || style.aspect_ratio.is_some()
            || !style.box_shadows.is_empty()
            || !style.background_layers.is_empty()
            || !style.background_images.is_empty()
            || style.margin.iter().flatten().any(|margin| margin != "0")
            || style.padding.iter().flatten().any(|padding| padding != "0")
            || style
                .border_style
                .iter()
                .any(|style| *style != BorderLineStyle::None)
            || style.outline_style != BorderLineStyle::None
            || style.overflow_x != Overflow::Visible
            || style.overflow_y != Overflow::Visible
            || style.paint_contained
            || style.size_contained
            || style.inline_size_contained);
    if !has_block_size {
        return false;
    }
    style.layout_sizes.push(LayoutSize {
        node_id,
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
        aspect_ratio: style.aspect_ratio,
        box_shadows: style.box_shadows.clone(),
        background_layers: style.background_layers.clone(),
        background_images: style.background_images.clone(),
        background_repeats: style.background_repeats.clone(),
        opacity: style.opacity,
        visibility_hidden: style.visibility_hidden,
        content_skipped: style.content_skipped,
        padding: style.padding.clone(),
        margin: style.margin.clone(),
        intrinsic_min_content_width: None,
        intrinsic_max_content_width: None,
        overflow_x: style.overflow_x,
        overflow_y: style.overflow_y,
        paint_contained: style.paint_contained,
        size_contained: style.size_contained,
        inline_size_contained: style.inline_size_contained,
        contain_intrinsic_width: style.contain_intrinsic_width.clone(),
        contain_intrinsic_height: style.contain_intrinsic_height.clone(),
        size_property_orders: style.size_property_orders.clone(),
        writing_mode: style.writing_mode,
        font_size: style.font_size,
        line_height: style.line_height,
        font_relative_units: style.font_relative_units,
        viewport_size: style.viewport_size,
    });
    true
}
