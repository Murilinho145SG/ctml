use crate::style::{LayoutSize, TextRun};

pub(super) fn in_flow_layout_sizes(run: &TextRun) -> &[LayoutSize] {
    let count = run
        .style
        .layout_sizes
        .len()
        .saturating_sub(usize::from(run.style.margin_start_spacer));
    &run.style.layout_sizes[..count]
}
