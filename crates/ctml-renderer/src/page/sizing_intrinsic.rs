use std::collections::{HashMap, HashSet};

use cosmic_text::{Attrs, Buffer, Metrics, Shaping, Wrap};

use crate::font_family::resolve_family_list;
use ctml_css::{FontRelativeUnits, ViewportSize};

use crate::style::{LayoutSize, OverflowWrap, TextRun, WhiteSpace, WordBreak};

use super::PageRasterizer;
use super::sizing::resolve_width;

pub(super) fn resolve_intrinsic_width(
    value: &str,
    available: f32,
    min_content: Option<f32>,
    max_content: Option<f32>,
    font_size: f32,
    line_height: f32,
    font_relative_units: FontRelativeUnits,
    viewport_size: ViewportSize,
) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    match value.as_str() {
        "min-content" => min_content,
        "max-content" => max_content,
        "fit-content" => Some(
            max_content
                .unwrap_or(available)
                .min(min_content.unwrap_or_default().max(available)),
        ),
        _ => value
            .strip_prefix("fit-content(")
            .and_then(|value| value.strip_suffix(')'))
            .and_then(|argument| {
                resolve_width(
                    argument,
                    available,
                    font_size,
                    line_height,
                    font_relative_units,
                    viewport_size,
                )
            })
            .map(|argument| {
                max_content
                    .unwrap_or(available)
                    .min(min_content.unwrap_or_default().max(argument))
            }),
    }
}

pub(super) fn resolve_intrinsic_layout_width(
    size: &LayoutSize,
    value: &str,
    available: f32,
) -> Option<f32> {
    if size.inline_size_contained {
        return Some(0.0);
    }
    resolve_intrinsic_width(
        value,
        available,
        size.intrinsic_min_content_width,
        size.intrinsic_max_content_width,
        size.font_size,
        size.line_height,
        size.font_relative_units,
        size.viewport_size,
    )
}

pub(super) fn resolve_constrained_size(
    specified: Option<&str>,
    min: Option<&str>,
    max: Option<&str>,
    available: f32,
    font_size: f32,
    line_height: f32,
    font_relative_units: FontRelativeUnits,
    viewport_size: ViewportSize,
    intrinsic: Option<(f32, f32)>,
) -> f32 {
    let resolve = |value: &str| {
        resolve_width(
            value,
            available,
            font_size,
            line_height,
            font_relative_units,
            viewport_size,
        )
        .or_else(|| {
            let (min_content, max_content) = intrinsic?;
            let value = value.trim().to_ascii_lowercase();
            match value.as_str() {
                "min-content" => Some(min_content),
                "max-content" => Some(max_content),
                "fit-content" => Some(max_content.min(min_content.max(available))),
                _ => value
                    .strip_prefix("fit-content(")
                    .and_then(|value| value.strip_suffix(')'))
                    .and_then(|argument| {
                        resolve_width(
                            argument,
                            available,
                            font_size,
                            line_height,
                            font_relative_units,
                            viewport_size,
                        )
                    })
                    .map(|argument| max_content.min(min_content.max(argument))),
            }
        })
    };
    let min = min.and_then(resolve);
    let max = max.and_then(resolve);
    let specified = specified.and_then(resolve).unwrap_or(available);
    match (min, max) {
        (Some(min), Some(max)) if min > max => specified.max(min),
        (Some(min), Some(max)) => specified.max(min).min(max),
        (Some(min), None) => specified.max(min),
        (None, Some(max)) => specified.min(max),
        (None, None) => specified,
    }
}

pub(super) fn measure_root_intrinsic_widths(page: &mut PageRasterizer) -> Option<(f32, f32)> {
    if !root_has_intrinsic_size(page) {
        return None;
    }

    let families = page
        .runs
        .iter()
        .map(|run| resolve_family_list(&run.style.family, &page.font_system))
        .collect::<Vec<_>>();
    let attributes = page
        .runs
        .iter()
        .zip(&families)
        .map(|(run, family)| run.style.to_attrs(family))
        .collect::<Vec<_>>();
    let root_family = resolve_family_list(&page.root_style.family, &page.font_system);
    let root_attributes = page.root_style.to_attrs(&root_family);
    let spans = page
        .runs
        .iter()
        .zip(&attributes)
        .map(|(run, attrs)| (run.text.as_str(), attrs.clone()))
        .collect::<Vec<_>>();
    let run_indices = (0..page.runs.len()).collect::<Vec<_>>();
    let wrap = min_content_wrap(&page.runs, &run_indices);
    let max_content = measure_spans(
        &mut page.font_system,
        spans.iter().map(|(text, attrs)| (*text, attrs.clone())),
        &root_attributes,
        Wrap::None,
        None,
    );
    let min_content = measure_spans(
        &mut page.font_system,
        spans.iter().map(|(text, attrs)| (*text, attrs.clone())),
        &root_attributes,
        wrap,
        Some(1.0),
    );
    Some((min_content.min(max_content), max_content))
}

pub(super) fn measure_intrinsic_widths(
    page: &mut PageRasterizer,
    attributes: &[Attrs<'_>],
    default_attributes: &Attrs<'_>,
) {
    let mut node_runs = HashMap::<usize, Vec<usize>>::new();
    for (run_index, run) in page.runs.iter().enumerate() {
        for size in &run.style.layout_sizes {
            if size_has_intrinsic_size(size) {
                node_runs.entry(size.node_id).or_default().push(run_index);
            }
        }
    }

    let mut widths = HashMap::with_capacity(node_runs.len());
    for (node_id, run_indices) in node_runs {
        let mut seen_runs = HashSet::new();
        let run_indices = run_indices
            .into_iter()
            .filter(|run_index| seen_runs.insert(*run_index))
            .collect::<Vec<_>>();
        let spans = run_indices
            .iter()
            .map(|&run_index| {
                (
                    page.runs[run_index].text.as_str(),
                    attributes[run_index].clone(),
                )
            })
            .collect::<Vec<_>>();
        let wrap = min_content_wrap(&page.runs, &run_indices);
        let max_content = measure_spans(
            &mut page.font_system,
            spans.iter().map(|(text, attrs)| (*text, attrs.clone())),
            default_attributes,
            Wrap::None,
            None,
        );
        let min_content = measure_spans(
            &mut page.font_system,
            spans.iter().map(|(text, attrs)| (*text, attrs.clone())),
            default_attributes,
            wrap,
            Some(1.0),
        );
        widths.insert(node_id, (min_content.min(max_content), max_content));
    }

    for run in &mut page.runs {
        for size in run
            .style
            .layout_sizes
            .iter_mut()
            .chain(&mut run.style.layout_end_sizes)
        {
            if let Some((min_content, max_content)) = widths.get(&size.node_id) {
                size.intrinsic_min_content_width = Some(*min_content);
                size.intrinsic_max_content_width = Some(*max_content);
            }
        }
    }
}

fn measure_spans<'a, I>(
    font_system: &mut cosmic_text::FontSystem,
    spans: I,
    default_attributes: &Attrs<'_>,
    wrap: Wrap,
    width: Option<f32>,
) -> f32
where
    I: IntoIterator<Item = (&'a str, Attrs<'a>)>,
{
    let mut buffer = Buffer::new_empty(Metrics::new(16.0, 20.0));
    buffer.set_wrap(wrap);
    buffer.set_size(width, None);
    buffer.set_rich_text(spans, default_attributes, Shaping::Advanced, None);
    buffer.shape_until_scroll(font_system, false);
    buffer
        .layout_runs()
        .map(|line| line.line_w)
        .fold(0.0, f32::max)
}

fn min_content_wrap(runs: &[TextRun], run_indices: &[usize]) -> Wrap {
    if run_indices.iter().any(|&index| {
        let style = &runs[index].style;
        style.white_space == WhiteSpace::NoWrap
            || style.text_wrap_mode == crate::style::TextWrapMode::NoWrap
    }) {
        return Wrap::None;
    }
    if run_indices.iter().any(|&index| {
        let style = &runs[index].style;
        style.overflow_wrap == OverflowWrap::Anywhere || style.word_break == WordBreak::BreakAll
    }) {
        Wrap::Glyph
    } else {
        Wrap::Word
    }
}

fn size_has_intrinsic_size(size: &LayoutSize) -> bool {
    [
        size.width.as_deref(),
        size.min_width.as_deref(),
        size.max_width.as_deref(),
        size.inline_size.as_deref(),
        size.min_inline_size.as_deref(),
        size.max_inline_size.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|value| {
        let value = value.trim().to_ascii_lowercase();
        matches!(
            value.as_str(),
            "min-content" | "max-content" | "fit-content"
        ) || value.starts_with("fit-content(")
    })
}

fn root_has_intrinsic_size(page: &PageRasterizer) -> bool {
    [
        page.root_style.width.as_deref(),
        page.root_style.min_width.as_deref(),
        page.root_style.max_width.as_deref(),
        page.root_style.inline_size.as_deref(),
        page.root_style.min_inline_size.as_deref(),
        page.root_style.max_inline_size.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(is_intrinsic_size)
}

fn is_intrinsic_size(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    matches!(
        value.as_str(),
        "min-content" | "max-content" | "fit-content"
    ) || value.starts_with("fit-content(")
}
