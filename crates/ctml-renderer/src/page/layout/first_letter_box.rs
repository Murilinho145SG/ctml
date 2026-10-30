use std::collections::HashMap;
use std::collections::HashSet;

use crate::page::{CursorSpan, highlights::first_formatted_line_ranges};
use crate::style::{Direction, TextRun, TextStyle, UserSelect, WritingMode};

use super::super::sizing::{resolve_layout_width, resolve_width};

pub(in crate::page) fn insert_first_letter_spacing(
    runs: &mut Vec<TextRun>,
    available_width: f32,
    page_writing_mode: WritingMode,
) {
    let mut output = Vec::with_capacity(runs.len() + 8);
    let mut handled_blocks = HashSet::new();
    let mut index = 0;
    while index < runs.len() {
        let Some(block) = runs[index].first_letter_block else {
            output.push(runs[index].clone());
            index += 1;
            continue;
        };
        let mut end = index + 1;
        while end < runs.len() && runs[end].first_letter_block == Some(block) {
            end += 1;
        }
        if !handled_blocks.insert(block) {
            output.extend(runs[index..end].iter().cloned());
            index = end;
            continue;
        }
        let Some(pseudo_style) = runs[index].first_letter_font_style.as_ref() else {
            output.extend(runs[index..end].iter().cloned());
            index = end;
            continue;
        };
        let containing_width = containing_inline_width(
            &runs[index].style,
            block,
            available_width,
            page_writing_mode,
        );
        let (start_side, end_side) =
            inline_sides(pseudo_style.writing_mode, pseudo_style.direction);
        let start_gap = resolve_box_side(pseudo_style, start_side, containing_width, true)
            + resolve_box_side(pseudo_style, start_side, containing_width, false)
            + resolve_border_side(pseudo_style, start_side);
        let end_gap = resolve_box_side(pseudo_style, end_side, containing_width, true)
            + resolve_box_side(pseudo_style, end_side, containing_width, false)
            + resolve_border_side(pseudo_style, end_side);
        if start_gap.abs() > f32::EPSILON {
            output.push(spacing_run(&runs[index], pseudo_style, start_gap));
        }
        output.extend(runs[index..end].iter().cloned());
        if end_gap.abs() > f32::EPSILON {
            output.push(spacing_run(&runs[end - 1], pseudo_style, end_gap));
        }
        index = end;
    }
    *runs = output;
}

pub(in crate::page) fn insert_first_line_spacing(
    buffer: &cosmic_text::Buffer,
    runs: &mut Vec<TextRun>,
    spans: &[CursorSpan],
    available_width: f32,
    page_writing_mode: WritingMode,
) -> bool {
    let first_lines = first_formatted_line_ranges(buffer, spans);
    let mut insertions: HashMap<usize, Vec<(usize, TextStyle, f32)>> = HashMap::new();
    for (block, (line, end)) in first_lines {
        let Some(first_span) = spans
            .iter()
            .filter(|span| span.first_line_block == Some(block) && span.line == line)
            .filter(|span| span.start < end && span.first_line_font_style.is_some())
            .min_by_key(|span| span.start)
        else {
            continue;
        };
        let Some(last_span) = spans
            .iter()
            .filter(|span| span.first_line_block == Some(block) && span.line == line)
            .filter(|span| span.start < end && span.first_line_font_style.is_some())
            .max_by_key(|span| span.end)
        else {
            continue;
        };
        let Some(pseudo_style) = first_span.first_line_font_style.as_ref() else {
            continue;
        };
        let Some(originating_run) = runs.get(first_span.run_index) else {
            continue;
        };
        let containing_width = containing_inline_width(
            &originating_run.style,
            block,
            available_width,
            page_writing_mode,
        );
        let (start_side, end_side) = inline_sides(
            originating_run.style.writing_mode,
            originating_run.style.direction,
        );
        let start_gap = resolve_box_side(pseudo_style, start_side, containing_width, true)
            + resolve_box_side(pseudo_style, start_side, containing_width, false)
            + resolve_border_side(pseudo_style, start_side);
        let end_gap = resolve_box_side(pseudo_style, end_side, containing_width, true)
            + resolve_box_side(pseudo_style, end_side, containing_width, false)
            + resolve_border_side(pseudo_style, end_side);
        if start_gap.abs() > f32::EPSILON {
            insertions.entry(first_span.run_index).or_default().push((
                first_span.run_offset,
                pseudo_style.clone(),
                start_gap,
            ));
        }
        if end_gap.abs() > f32::EPSILON {
            insertions.entry(last_span.run_index).or_default().push((
                last_span.run_offset + last_span.character.len_utf8(),
                pseudo_style.clone(),
                end_gap,
            ));
        }
    }
    if insertions.is_empty() {
        return false;
    }

    let mut output = Vec::with_capacity(runs.len() + insertions.len());
    for (run_index, run) in runs.iter().enumerate() {
        let Some(mut positions) = insertions.remove(&run_index) else {
            output.push(run.clone());
            continue;
        };
        positions.sort_by_key(|(offset, _, _)| *offset);
        let mut offset = 0;
        for (position, pseudo_style, spacing) in positions {
            let position = position.min(run.text.len());
            if position > offset {
                let mut prefix = run.clone();
                prefix.text = run.text[offset..position].to_owned();
                output.push(prefix);
            }
            output.push(first_line_spacing_run(run, &pseudo_style, spacing));
            offset = position;
        }
        if offset < run.text.len() {
            let mut suffix = run.clone();
            suffix.text = run.text[offset..].to_owned();
            output.push(suffix);
        }
    }
    *runs = output;
    true
}

fn containing_inline_width(
    style: &TextStyle,
    block: usize,
    available_width: f32,
    page_writing_mode: WritingMode,
) -> f32 {
    let parent_count = style
        .layout_sizes
        .iter()
        .rposition(|size| size.node_id == block)
        .unwrap_or(style.layout_sizes.len());
    resolve_layout_width(
        &style.layout_sizes[..parent_count],
        available_width,
        page_writing_mode,
    )
}

pub(in crate::page) fn resolve_first_letter_padding(
    originating_style: &TextStyle,
    pseudo_style: &TextStyle,
    block: usize,
    available_width: f32,
    page_writing_mode: WritingMode,
) -> [f32; 4] {
    let containing_width =
        containing_inline_width(originating_style, block, available_width, page_writing_mode);
    std::array::from_fn(|side| resolve_box_side(pseudo_style, side, containing_width, true))
}

fn inline_sides(writing_mode: WritingMode, direction: Direction) -> (usize, usize) {
    match (writing_mode, direction) {
        (WritingMode::HorizontalTb, Direction::Ltr) => (3, 1),
        (WritingMode::HorizontalTb, Direction::Rtl) => (1, 3),
        (_, Direction::Ltr) => (0, 2),
        (_, Direction::Rtl) => (2, 0),
    }
}

fn resolve_box_side(style: &TextStyle, side: usize, containing_width: f32, padding: bool) -> f32 {
    let value = if padding {
        style.padding[side].as_deref()
    } else {
        style.margin[side].as_deref()
    };
    let Some(value) = value else {
        return 0.0;
    };
    if padding {
        return resolve_width(
            value,
            containing_width,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
        .unwrap_or_default();
    }
    value
        .strip_suffix('%')
        .and_then(|percent| percent.trim().parse::<f32>().ok())
        .map(|percent| containing_width * percent / 100.0)
        .filter(|margin| margin.is_finite())
        .or_else(|| {
            ctml_css::parse_signed_font_relative_length_with_viewport(
                value,
                style.font_size,
                style.line_height,
                style.font_relative_units,
                style.viewport_size,
            )
        })
        .unwrap_or_default()
}

pub(in crate::page) fn resolve_border_side(style: &TextStyle, side: usize) -> f32 {
    if style.border_style[side] == crate::style::BorderLineStyle::None
        && style.border_image_source.is_none()
    {
        return 0.0;
    }
    match style.border_width[side].as_deref().unwrap_or("medium") {
        "thin" => 1.0,
        "medium" => 3.0,
        "thick" => 5.0,
        value => resolve_width(
            value,
            0.0,
            style.font_size,
            style.line_height,
            style.font_relative_units,
            style.viewport_size,
        )
        .unwrap_or_default(),
    }
}

fn spacing_run(source: &TextRun, pseudo_style: &TextStyle, spacing: f32) -> TextRun {
    let mut spacer = source.clone();
    spacer.text = "\u{200b}".to_owned();
    spacer.source_first_letter_character = None;
    spacer.style = source.style.clone();
    spacer.style.font_size = 1.0;
    spacer.style.font_scale = 1.0;
    spacer.style.line_height = 1.0;
    spacer.style.line_height_multiplier = None;
    spacer.style.line_height_normal = false;
    spacer.style.letter_spacing = spacing;
    spacer.style.letter_spacing_px = true;
    spacer.style.inline_box = true;
    spacer.style.pointer_events_none = true;
    spacer.style.cursor_hidden = true;
    spacer.style.user_select = UserSelect::None;
    spacer.style.color = pseudo_style.color;
    spacer.style.opacity = source.style.opacity * source.first_letter_opacity.unwrap_or(1.0);
    spacer.first_letter_foreground = None;
    spacer.first_letter_background = None;
    spacer.first_letter_decoration = None;
    spacer.first_letter_opacity = None;
    spacer.first_letter_font_style = Some(pseudo_style.clone());
    spacer.selection_background = None;
    spacer.selection_foreground = None;
    spacer
}

fn first_line_spacing_run(source: &TextRun, pseudo_style: &TextStyle, spacing: f32) -> TextRun {
    let mut spacer = spacing_run(source, pseudo_style, spacing);
    spacer.first_letter_block = None;
    spacer.first_letter_foreground = None;
    spacer.first_letter_background = None;
    spacer.first_letter_decoration = None;
    spacer.first_letter_opacity = None;
    spacer.first_letter_font_style = None;
    spacer.first_line_font_style = Some(pseudo_style.clone());
    spacer
}
