use cosmic_text::Shaping;

use crate::{font_family::resolve_family_list, font_metrics::normal_line_height};

use super::{
    PageRasterizer, line_alignments,
    sizing::{line_layout_sizes, resolve_layout_height, resolve_layout_width},
};

pub(super) fn shape_buffer(page: &mut PageRasterizer, content_width: f32, content_height: f32) {
    page.buffer
        .set_size(Some(content_width), Some(content_height));
    let run_families = page
        .runs
        .iter()
        .map(|run| resolve_family_list(&run.style.family, &page.font_system))
        .collect::<Vec<_>>();
    let attributes = page
        .runs
        .iter()
        .zip(&run_families)
        .map(|(run, family)| {
            let mut style = run.style.clone();
            if let Some(height_size) = &style.empty_block_height {
                style.line_height = resolve_layout_height(
                    height_size.height.as_deref(),
                    height_size.min_height.as_deref(),
                    height_size.max_height.as_deref(),
                    content_height,
                    style.font_size,
                    style.line_height,
                    style.font_relative_units,
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
    page.buffer.set_rich_text(
        page.runs
            .iter()
            .zip(&attributes)
            .map(|(run, attrs)| (run.text.as_str(), attrs.clone())),
        &root_attributes,
        Shaping::Advanced,
        Some(page.root_style.text_align),
    );
    for (line, align) in page
        .buffer
        .lines
        .iter_mut()
        .zip(line_alignments(&page.runs))
    {
        line.set_align(Some(align));
    }
    page.buffer.shape_until_scroll(&mut page.font_system, false);
    let line_sizes = line_layout_sizes(&page.runs, page.buffer.lines.len());
    let metrics = page.buffer.metrics();
    let wrap = page.buffer.wrap();
    let ellipsize = page.buffer.ellipsize();
    let monospace_width = page.buffer.monospace_width();
    let tab_width = page.buffer.tab_width();
    let hinting = page.buffer.hinting();
    for (line, sizes) in page.buffer.lines.iter_mut().zip(line_sizes) {
        let has_custom_width = !sizes.is_empty();
        if !has_custom_width && line.layout_opt().is_some() {
            continue;
        }
        let line_width = if has_custom_width {
            resolve_layout_width(&sizes, content_width)
        } else {
            content_width
        };
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
