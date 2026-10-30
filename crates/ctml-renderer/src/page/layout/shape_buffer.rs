use cosmic_text::Shaping;

use crate::{font_family::resolve_family_list, font_metrics::normal_line_height};

use super::super::{
    PageRasterizer,
    sizing::{line_layout_sizes, resolve_layout_width, resolve_width, select_axis_size},
    sizing_height::resolve_layout_height_with_ancestors,
    sizing_intrinsic::measure_intrinsic_widths,
    sizing_padding::{inline_margin, inline_padding},
};
use super::{line_alignments, line_writing_modes, paragraph_directions};

pub fn shape_buffer(page: &mut PageRasterizer, content_width: f32, content_height: f32) {
    page.buffer.set_size(
        Some(content_width.max(content_height)),
        Some(content_height),
    );
    let run_families = page
        .runs
        .iter()
        .map(|run| resolve_family_list(&run.style.family, &page.font_system))
        .collect::<Vec<_>>();
    let page_writing_mode = page.writing_mode();
    let attributes = page
        .runs
        .iter()
        .zip(&run_families)
        .map(|(run, family)| {
            let mut style = run.style.clone();
            if style.padding_start_spacer
                && style.padding[0]
                    .as_deref()
                    .is_some_and(|padding| padding.contains('%'))
            {
                let parent_width = resolve_layout_width(
                    &style.layout_sizes[..style.layout_sizes.len().saturating_sub(1)],
                    content_width,
                    page_writing_mode,
                );
                style.line_height = style.padding[0]
                    .as_deref()
                    .and_then(|value| {
                        resolve_width(
                            value,
                            parent_width,
                            style.font_size,
                            style.line_height,
                            style.font_relative_units,
                            style.viewport_size,
                        )
                    })
                    .unwrap_or_default();
            }
            let margin_start_side = match style.writing_mode {
                crate::style::WritingMode::HorizontalTb => 0,
                crate::style::WritingMode::VerticalRl => 1,
                crate::style::WritingMode::VerticalLr => 3,
            };
            if style.margin_start_spacer
                && style.margin[margin_start_side]
                    .as_deref()
                    .is_some_and(|margin| margin.contains('%'))
            {
                let parent_width = resolve_layout_width(
                    &style.layout_sizes[..style.layout_sizes.len().saturating_sub(1)],
                    content_width,
                    page_writing_mode,
                );
                style.line_height = style.layout_sizes.last().map_or(0.0, |size| {
                    super::super::sizing_padding::resolve_margin(size, parent_width)
                        [margin_start_side]
                });
            }
            if let Some(height_size) = &style.empty_block_height {
                let same_axis_orientation = (height_size.writing_mode
                    != crate::style::WritingMode::HorizontalTb)
                    == (page_writing_mode != crate::style::WritingMode::HorizontalTb);
                let (
                    logical_size,
                    logical_min,
                    logical_max,
                    logical_name,
                    logical_min_name,
                    logical_max_name,
                ) = if same_axis_orientation {
                    (
                        &height_size.block_size,
                        &height_size.min_block_size,
                        &height_size.max_block_size,
                        "block-size",
                        "min-block-size",
                        "max-block-size",
                    )
                } else {
                    (
                        &height_size.inline_size,
                        &height_size.min_inline_size,
                        &height_size.max_inline_size,
                        "inline-size",
                        "min-inline-size",
                        "max-inline-size",
                    )
                };
                let (
                    physical_size,
                    physical_min,
                    physical_max,
                    physical_name,
                    physical_min_name,
                    physical_max_name,
                ) = if page_writing_mode == crate::style::WritingMode::HorizontalTb {
                    (
                        &height_size.height,
                        &height_size.min_height,
                        &height_size.max_height,
                        "height",
                        "min-height",
                        "max-height",
                    )
                } else {
                    (
                        &height_size.width,
                        &height_size.min_width,
                        &height_size.max_width,
                        "width",
                        "min-width",
                        "max-width",
                    )
                };
                let block_size = select_axis_size(
                    logical_size.as_ref(),
                    physical_size.as_ref(),
                    &height_size.size_property_orders,
                    logical_name,
                    physical_name,
                )
                .flatten();
                let min_block_size = select_axis_size(
                    logical_min.as_ref(),
                    physical_min.as_ref(),
                    &height_size.size_property_orders,
                    logical_min_name,
                    physical_min_name,
                )
                .flatten();
                let max_block_size = select_axis_size(
                    logical_max.as_ref(),
                    physical_max.as_ref(),
                    &height_size.size_property_orders,
                    logical_max_name,
                    physical_max_name,
                )
                .flatten();
                style.line_height = resolve_layout_height_with_ancestors(
                    block_size,
                    min_block_size,
                    max_block_size,
                    content_height,
                    content_width,
                    &style.layout_sizes,
                    page_writing_mode,
                    height_size.border_box,
                    &height_size.padding,
                    &height_size.margin,
                    style.font_size,
                    style.line_height,
                    style.font_relative_units,
                    style.viewport_size,
                );
                style.line_height_normal = false;
            }
            if style.line_height_normal {
                style.line_height = normal_line_height(&mut page.font_system, &style, family)
                    .unwrap_or(style.font_size * 1.2);
            }
            style.to_attrs(family)
        })
        .collect::<Vec<_>>();
    let root_family = resolve_family_list(&page.root_style.family, &page.font_system);
    let mut root_style = page.root_style.clone();
    if root_style.line_height_normal {
        root_style.line_height =
            normal_line_height(&mut page.font_system, &root_style, &root_family)
                .unwrap_or(root_style.font_size * 1.2);
    }
    let root_attributes = root_style.to_attrs(&root_family);
    measure_intrinsic_widths(page, &attributes, &root_attributes);
    page.buffer.set_rich_text(
        page.runs
            .iter()
            .zip(&attributes)
            .map(|(run, attrs)| (run.text.as_str(), attrs.clone())),
        &root_attributes,
        Shaping::Advanced,
        Some(page.root_style.text_align),
    );
    let paragraph_directions = paragraph_directions(&page.runs, page.buffer.lines.len());
    for (line, direction) in page.buffer.lines.iter_mut().zip(paragraph_directions) {
        line.set_base_rtl(Some(direction == crate::style::Direction::Rtl));
    }
    let line_styles = line_alignments(&page.runs);
    let page_writing_mode = page.writing_mode();
    let line_writing_modes =
        line_writing_modes(&page.runs, page.buffer.lines.len(), page_writing_mode);
    for (
        line,
        (
            align,
            align_last,
            text_justify,
            _,
            keep_all,
            hyphens_none,
            preserve_spaces,
            ellipsize,
            _,
            _,
            _,
            _,
        ),
    ) in page
        .buffer
        .lines
        .iter_mut()
        .zip(line_styles.iter().copied())
    {
        line.set_align(Some(align));
        line.set_align_last(align_last);
        line.set_text_justify(text_justify);
        line.set_keep_all(keep_all);
        line.set_hyphens_none(hyphens_none);
        line.set_preserve_spaces(preserve_spaces);
        line.set_ellipsize(ellipsize);
    }
    page.buffer.shape_until_scroll(&mut page.font_system, false);
    let line_sizes = line_layout_sizes(&page.runs, page.buffer.lines.len());
    let metrics = page.buffer.metrics();
    let monospace_width = page.buffer.monospace_width();
    let tab_width = page.buffer.tab_width();
    let hinting = page.buffer.hinting();
    let page_writing_mode = page.writing_mode();
    for (
        ((line, sizes), line_writing_mode),
        (_, _, _, wrap, _, _, _, ellipsize, indent, indent_percent, hanging, each_line),
    ) in page
        .buffer
        .lines
        .iter_mut()
        .zip(line_sizes)
        .zip(line_writing_modes)
        .zip(line_styles)
    {
        let has_custom_width = !sizes.is_empty();
        let line_is_orthogonal = super::super::sizing::is_vertical(line_writing_mode)
            != super::super::sizing::is_vertical(page_writing_mode);
        let inline_available = if line_is_orthogonal {
            content_height
        } else {
            content_width
        };
        line.set_wrap(wrap);
        let line_content_width = if has_custom_width {
            let has_page_block_size = sizes.iter().any(|size| {
                size.block_size.is_some()
                    || size.min_block_size.is_some()
                    || size.max_block_size.is_some()
                    || size.width.is_some()
                    || size.min_width.is_some()
                    || size.max_width.is_some()
            });
            if line_is_orthogonal && has_page_block_size {
                resolve_layout_width(&sizes, content_width, page_writing_mode)
            } else {
                resolve_layout_width(&sizes, inline_available, line_writing_mode)
            }
        } else {
            inline_available
        };
        let (padding_start, _) = inline_padding(&sizes, inline_available, page_writing_mode);
        let (margin_start, _) = inline_margin(&sizes, inline_available, page_writing_mode);
        let line_width = line_content_width + padding_start + margin_start;
        line.set_indent(
            padding_start + margin_start + indent + line_content_width * indent_percent / 100.0,
            hanging,
            each_line,
        );
        if !has_custom_width && line.layout_opt().is_some() {
            continue;
        }
        line.reset_layout();
        line.layout(
            &mut page.font_system,
            metrics.font_size,
            Some(line_width),
            wrap,
            ellipsize,
            monospace_width,
            tab_width,
            hinting,
        );
    }
}
