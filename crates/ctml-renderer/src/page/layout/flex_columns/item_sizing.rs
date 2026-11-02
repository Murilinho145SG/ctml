use crate::{
    page::{
        PageRasterizer,
        layout::flex_columns::VisualLine,
        sizing::resolve_layout_width,
        sizing_padding::{resolve_borders, resolve_margin, resolve_padding},
    },
    style::{FlexBasis, WritingMode},
};

pub(super) fn resolve_item_width(
    page: &PageRasterizer,
    item: usize,
    item_lines: &[usize],
    lines: &[VisualLine],
    basis: Option<&FlexBasis>,
    container_width: f32,
    writing_mode: WritingMode,
) -> f32 {
    let basis_width = match basis {
        Some(FlexBasis::Percentage(percentage)) => Some(container_width * percentage),
        Some(FlexBasis::Length(length)) => Some(*length),
        Some(FlexBasis::GridFraction {
            fraction,
            fixed_space,
            percentage_space,
        }) => Some(
            ((container_width - fixed_space - container_width * percentage_space).max(0.0)
                * fraction)
                .max(0.0),
        ),
        _ => None,
    };
    let measured = item_lines
        .iter()
        .map(|line| (lines[*line].max_x - lines[*line].min_x).max(0.0))
        .fold(0.0_f32, f32::max);
    let Some((run, size_index)) = page.runs.iter().find_map(|run| {
        run.style
            .layout_sizes
            .iter()
            .position(|size| size.node_id == item)
            .map(|index| (run, index))
    }) else {
        return basis_width.unwrap_or(measured);
    };
    let size = &run.style.layout_sizes[size_index];
    let containing_width = resolve_layout_width(
        &run.style.layout_sizes[..size_index],
        container_width,
        writing_mode,
    );
    let padding = resolve_padding(size, containing_width);
    let border = resolve_borders(size, containing_width);
    let decoration = if writing_mode == WritingMode::HorizontalTb {
        padding[1] + padding[3] + border[1] + border[3]
    } else {
        padding[0] + padding[2] + border[0] + border[2]
    };
    basis_width.map_or(measured + decoration, |width| {
        width + if size.border_box { 0.0 } else { decoration }
    })
}

pub(super) fn resolve_item_spacing(
    page: &PageRasterizer,
    item: usize,
    container_width: f32,
    writing_mode: WritingMode,
) -> (f32, f32, f32) {
    let Some(run) = page.runs.iter().find(|run| {
        run.style
            .layout_sizes
            .iter()
            .any(|size| size.node_id == item)
    }) else {
        return (0.0, 0.0, 1.0);
    };
    let Some(index) = run
        .style
        .layout_sizes
        .iter()
        .position(|size| size.node_id == item)
    else {
        return (0.0, 0.0, run.style.flex_shrink);
    };
    let containing_width = resolve_layout_width(
        &run.style.layout_sizes[..index],
        container_width,
        writing_mode,
    );
    let margins = resolve_margin(&run.style.layout_sizes[index], containing_width);
    let (start, end) = match writing_mode {
        WritingMode::HorizontalTb => (margins[3], margins[1]),
        WritingMode::VerticalRl => (margins[0], margins[2]),
        WritingMode::VerticalLr => (margins[2], margins[0]),
    };
    (start, end, run.style.flex_shrink)
}
