use cosmic_text::Metrics;

use crate::{
    font_family::resolve_family_list,
    style::{compile_runs_with_opacity_map, synthesize_font_variant_caps},
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
    let (
        compiled_runs,
        root_style,
        opacity_groups,
        mix_blend_modes,
        isolation_groups,
        transition_styles,
    ) = compile_runs_with_opacity_map(
        &page.nodes,
        &page.stylesheet,
        &page.hovered_path,
        &page.active_path,
        &page.focused_path,
        page.keyboard_focus,
        Some(&page.font_system),
        &page.color_profiles,
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
    page.opacity_groups = opacity_groups;
    page.element_colors = transition_styles
        .iter()
        .map(|(node_id, style)| (*node_id, style.color_value))
        .collect();
    page.mix_blend_modes = mix_blend_modes;
    page.isolation_groups = isolation_groups;
    page.decoration_opacity_scales.clear();
    page.root_opacity = root_style.opacity;
    page.root_style = root_style;
    page.root_style.opacity = 1.0;
    page.cursor_spans = cursor_spans(&page.runs);
    page.buffer.set_metrics(Metrics::new(
        page.root_style.font_size.max(1.0),
        page.root_style.line_height.max(1.0),
    ));
    viewport
}
