use cosmic_text::Metrics;

use crate::{
    font_family::resolve_family_list,
    style::{compile_runs_with_font_system, synthesize_font_variant_caps},
};

use super::{PageRasterizer, cursor_spans, font_features};

pub(super) fn resize_styles(
    page: &mut PageRasterizer,
    width: u32,
    height: u32,
) -> ctml_css::ViewportSize {
    let viewport = ctml_css::ViewportSize {
        width: width as f32,
        height: height as f32,
    };
    let (compiled_runs, root_style) = compile_runs_with_font_system(
        &page.nodes,
        &page.stylesheet,
        &page.hovered_path,
        &page.active_path,
        &page.focused_path,
        page.keyboard_focus,
        Some(&page.font_system),
        viewport,
    );
    page.runs = compiled_runs
        .into_iter()
        .flat_map(|run| {
            let family = resolve_family_list(&run.style.family, &page.font_system);
            if font_features::supports_caps_features(&page.font_system, &family, &run.style) {
                vec![run]
            } else {
                synthesize_font_variant_caps(run)
            }
        })
        .collect();
    page.root_style = root_style;
    page.cursor_spans = cursor_spans(&page.runs);
    page.buffer.set_metrics(Metrics::new(
        page.root_style.font_size.max(1.0),
        page.root_style.line_height.max(1.0),
    ));
    viewport
}
