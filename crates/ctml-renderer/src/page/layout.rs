use cosmic_text::Shaping;
use std::collections::HashMap;

use crate::{font_family::resolve_family_list, font_metrics::normal_line_height};
use crate::{
    page::highlights::first_formatted_line_ranges,
    style::{TextRun, TextStyle, TextTransform},
};

use super::{
    CursorSpan, PageRasterizer,
    sizing::{line_layout_sizes, resolve_layout_height, resolve_layout_width},
};

pub(super) fn line_alignments(
    runs: &[TextRun],
) -> Vec<(
    cosmic_text::Align,
    Option<cosmic_text::Align>,
    cosmic_text::TextJustify,
    cosmic_text::Wrap,
    bool,
    bool,
    bool,
    cosmic_text::Ellipsize,
    f32,
    f32,
    bool,
    bool,
)> {
    let line_style = |style: &TextStyle| {
        (
            style.text_align,
            style.text_align_last,
            style.text_justify,
            style.wrap_mode(),
            style.word_break == crate::style::WordBreak::KeepAll,
            style.hyphens == crate::style::Hyphens::None,
            matches!(
                style.white_space,
                crate::style::WhiteSpace::PreWrap | crate::style::WhiteSpace::BreakSpaces
            ),
            style.ellipsize_mode(),
            style.text_indent,
            style.text_indent_percent,
            style.text_indent_hanging,
            style.text_indent_each_line,
        )
    };
    let mut alignments = vec![(
        cosmic_text::Align::Left,
        None,
        cosmic_text::TextJustify::Auto,
        cosmic_text::Wrap::Word,
        false,
        false,
        false,
        cosmic_text::Ellipsize::None,
        0.0,
        0.0,
        false,
        false,
    )];
    let mut line_has_text = false;
    for run in runs {
        for character in run.text.chars() {
            if character == '\n' {
                alignments.push(line_style(&run.style));
                line_has_text = false;
            } else if !line_has_text {
                *alignments.last_mut().expect("at least one text line") = line_style(&run.style);
                line_has_text = true;
            }
        }
    }
    alignments
}

pub(super) fn apply_first_line_word_spacing(
    buffer: &cosmic_text::Buffer,
    runs: &mut [TextRun],
    spans: &[CursorSpan],
) -> bool {
    let first_lines = first_formatted_line_ranges(buffer, spans);
    let mut changed = false;
    for span in spans {
        if !span.character.is_whitespace() {
            continue;
        }
        let Some(block) = span.first_line_block else {
            continue;
        };
        let Some((line, end)) = first_lines.get(&block).copied() else {
            continue;
        };
        if span.line != line || span.start >= end {
            continue;
        }
        let Some(pseudo_style) = &span.first_line_font_style else {
            continue;
        };
        let Some(run) = runs.get_mut(span.run_index) else {
            continue;
        };
        let font_size = run.style.font_size.max(1.0);
        let pseudo_spacing = pseudo_style.word_spacing_px
            + pseudo_style.word_spacing_percent.unwrap_or_default() * font_size / 100.0;
        let current_spacing = run.style.word_spacing_px
            + run.style.word_spacing_percent.unwrap_or_default() * font_size / 100.0;
        let delta = pseudo_spacing - current_spacing;
        if delta == 0.0 {
            continue;
        }
        if run.style.letter_spacing_px {
            run.style.letter_spacing /= font_size;
            run.style.letter_spacing_px = false;
        }
        run.style.letter_spacing += delta / font_size;
        run.style.word_spacing_px = pseudo_style.word_spacing_px;
        run.style.word_spacing_percent = pseudo_style.word_spacing_percent;
        changed = true;
    }
    changed
}

pub(super) fn apply_first_line_line_height(
    buffer: &cosmic_text::Buffer,
    runs: &mut Vec<TextRun>,
    spans: &[CursorSpan],
) -> bool {
    let first_lines = first_formatted_line_ranges(buffer, spans);
    let mut prefix_ends = HashMap::new();
    for span in spans {
        let Some(block) = span.first_line_block else {
            continue;
        };
        let Some((line, end)) = first_lines.get(&block).copied() else {
            continue;
        };
        if span.line != line || span.start >= end {
            continue;
        }
        let Some(style) = &span.first_line_font_style else {
            continue;
        };
        let Some(run) = runs.get(span.run_index) else {
            continue;
        };
        if run.style.line_height == style.line_height
            && run.style.line_height_multiplier == style.line_height_multiplier
            && run.style.line_height_normal == style.line_height_normal
        {
            continue;
        }
        prefix_ends
            .entry(span.run_index)
            .and_modify(|(run_end, _): &mut (usize, TextStyle)| {
                *run_end = (*run_end).max(span.run_offset + span.character.len_utf8());
            })
            .or_insert((span.run_offset + span.character.len_utf8(), style.clone()));
    }

    if prefix_ends.is_empty() {
        return false;
    }
    let mut output = Vec::with_capacity(runs.len() + prefix_ends.len());
    for (run_index, run) in runs.iter().enumerate() {
        let Some((end, pseudo_style)) = prefix_ends.remove(&run_index) else {
            output.push(run.clone());
            continue;
        };
        let end = end.min(run.text.len());
        let mut first_line = run.clone();
        first_line.text = run.text[..end].to_owned();
        first_line.style.line_height = pseudo_style.line_height;
        first_line.style.line_height_multiplier = pseudo_style.line_height_multiplier;
        first_line.style.line_height_normal = pseudo_style.line_height_normal;
        output.push(first_line);
        if end < run.text.len() {
            let mut remainder = run.clone();
            remainder.text = run.text[end..].to_owned();
            output.push(remainder);
        }
    }
    *runs = output;
    true
}

pub(super) fn apply_first_line_text_transform(
    buffer: &cosmic_text::Buffer,
    runs: &mut [TextRun],
    spans: &[CursorSpan],
) -> bool {
    let first_lines = first_formatted_line_ranges(buffer, spans);
    let mut capitalize_next = HashMap::new();
    let mut replacements = vec![Vec::new(); runs.len()];
    for span in spans {
        let Some(block) = span.first_line_block else {
            continue;
        };
        let Some((line, end)) = first_lines.get(&block).copied() else {
            continue;
        };
        if span.line != line || span.start >= end {
            continue;
        }
        let Some(style) = &span.first_line_font_style else {
            continue;
        };
        if span
            .first_letter_font_style
            .as_ref()
            .is_some_and(|first_letter| first_letter.text_transform != style.text_transform)
        {
            continue;
        }
        let capitalize = capitalize_next.entry(block).or_insert(true);
        let transformed = match style.text_transform {
            TextTransform::Uppercase => span.character.to_uppercase().collect::<String>(),
            TextTransform::Lowercase => span.character.to_lowercase().collect::<String>(),
            TextTransform::Capitalize if *capitalize => {
                span.character.to_uppercase().collect::<String>()
            }
            TextTransform::Capitalize | TextTransform::None => span.character.to_string(),
        };
        *capitalize = !span.character.is_alphanumeric();
        if transformed != span.character.to_string() {
            replacements[span.run_index].push((span.run_offset, span.character, transformed));
        }
    }

    let mut changed = false;
    for (run, replacements) in runs.iter_mut().zip(replacements) {
        for (offset, character, transformed) in replacements.into_iter().rev() {
            run.text
                .replace_range(offset..offset + character.len_utf8(), &transformed);
            changed = true;
        }
    }
    changed
}

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
    for (
        (line, sizes),
        (_, _, _, wrap, _, _, _, ellipsize, indent, indent_percent, hanging, each_line),
    ) in page
        .buffer
        .lines
        .iter_mut()
        .zip(line_sizes)
        .zip(line_styles)
    {
        let has_custom_width = !sizes.is_empty();
        line.set_wrap(wrap);
        let line_width = if has_custom_width {
            resolve_layout_width(&sizes, content_width)
        } else {
            content_width
        };
        line.set_indent(
            indent + line_width * indent_percent / 100.0,
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

fn paragraph_directions(runs: &[TextRun], line_count: usize) -> Vec<crate::style::Direction> {
    let mut directions = Vec::with_capacity(line_count);
    let mut current = None;
    for run in runs {
        for character in run.text.chars() {
            current.get_or_insert(run.style.direction);
            if character == '\n' {
                directions.push(current.take().unwrap_or_default());
            }
        }
    }
    if current.is_some() || directions.is_empty() {
        directions.push(current.unwrap_or_default());
    }
    directions.resize(
        line_count,
        *directions.last().unwrap_or(&crate::style::Direction::Ltr),
    );
    directions.truncate(line_count);
    directions
}
