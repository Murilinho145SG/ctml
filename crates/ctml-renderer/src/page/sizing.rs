use ctml_css::parse_length;

use crate::style::{LayoutSize, TextRun};

pub(super) fn resolve_width(value: &str, available_width: f32, font_size: f32) -> Option<f32> {
    let value = value.trim();
    if value == "0" {
        return Some(0.0);
    }
    if let Some(percentage) = value.strip_suffix('%') {
        return percentage
            .trim()
            .parse::<f32>()
            .ok()
            .map(|percentage| available_width * percentage / 100.0)
            .filter(|width| width.is_finite() && *width >= 0.0);
    }
    parse_length(value, font_size)
}

pub(super) fn resolve_layout_width(
    sizes: &[LayoutSize],
    available_width: f32,
    font_size: f32,
) -> f32 {
    let mut content_width = available_width;
    for size in sizes {
        let min_width = size
            .min_width
            .as_deref()
            .and_then(|value| resolve_width(value, content_width, font_size));
        let max_width = size
            .max_width
            .as_deref()
            .and_then(|value| resolve_width(value, content_width, font_size));
        let specified_width = size
            .width
            .as_deref()
            .and_then(|value| resolve_width(value, content_width, font_size))
            .unwrap_or(content_width);
        content_width = match (min_width, max_width) {
            (Some(min), Some(max)) if min > max => specified_width.max(min),
            (Some(min), Some(max)) => specified_width.max(min).min(max),
            (Some(min), None) => specified_width.max(min),
            (None, Some(max)) => specified_width.min(max),
            (None, None) => specified_width,
        };
    }
    content_width.max(1.0)
}

pub(super) fn line_layout_sizes(runs: &[TextRun], line_count: usize) -> Vec<Vec<LayoutSize>> {
    let mut sizes = vec![None; line_count];
    let mut line = 0;
    let mut line_has_text = false;
    for run in runs {
        for character in run.text.chars() {
            if character == '\n' {
                line += 1;
                line_has_text = false;
            } else if !line_has_text {
                if let Some(line_sizes) = sizes.get_mut(line) {
                    *line_sizes = Some(run.style.layout_sizes.clone());
                }
                line_has_text = true;
            }
        }
    }
    sizes.into_iter().map(Option::unwrap_or_default).collect()
}
