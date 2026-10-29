use super::CursorIcon;
use crate::style::TextRun;

pub(super) struct CursorSpan {
    pub(super) line: usize,
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) icon: CursorIcon,
    pub(super) hidden: bool,
    pub(super) opacity: f32,
    pub(super) element_path: Vec<usize>,
    pub(super) selection_background: Option<[u8; 4]>,
    pub(super) selection_foreground: Option<[u8; 4]>,
    pub(super) first_line_block: Option<usize>,
    pub(super) first_line_foreground: Option<[u8; 4]>,
    pub(super) first_line_background: Option<[u8; 4]>,
    pub(super) first_line_decoration: Option<crate::css_style::PseudoDecoration>,
    pub(super) first_line_opacity: Option<f32>,
    pub(super) first_line_font_style: Option<crate::style::TextStyle>,
    pub(super) first_letter_block: Option<usize>,
    pub(super) first_letter_foreground: Option<[u8; 4]>,
    pub(super) first_letter_background: Option<[u8; 4]>,
    pub(super) first_letter_decoration: Option<crate::css_style::PseudoDecoration>,
    pub(super) first_letter_opacity: Option<f32>,
    pub(super) first_letter_font_style: Option<crate::style::TextStyle>,
    pub(super) character: char,
    pub(super) run_index: usize,
    pub(super) run_offset: usize,
}

pub(super) fn cursor_spans(runs: &[TextRun]) -> Vec<CursorSpan> {
    let mut spans = Vec::new();
    let mut line = 0;
    let mut index = 0;
    for (run_index, run) in runs.iter().enumerate() {
        if run.style.flow_spacer {
            line += run
                .text
                .chars()
                .filter(|character| *character == '\n')
                .count();
            index = 0;
            continue;
        }
        let mut run_offset = 0;
        for character in run.text.chars() {
            if character == '\n' {
                line += 1;
                index = 0;
                run_offset += character.len_utf8();
                continue;
            }
            let end = index + character.len_utf8();
            spans.push(CursorSpan {
                line,
                start: index,
                end,
                icon: run.style.cursor,
                hidden: run.style.cursor_hidden,
                opacity: run.style.opacity,
                element_path: run.element_path.clone(),
                selection_background: run.selection_background,
                selection_foreground: run.selection_foreground,
                first_line_block: run.first_line_block,
                first_line_foreground: run.first_line_foreground,
                first_line_background: run.first_line_background,
                first_line_decoration: run.first_line_decoration,
                first_line_opacity: run.first_line_opacity,
                first_line_font_style: run.first_line_font_style.clone(),
                first_letter_block: run.first_letter_block,
                first_letter_foreground: run.first_letter_foreground,
                first_letter_background: run.first_letter_background,
                first_letter_decoration: run.first_letter_decoration,
                first_letter_opacity: run.first_letter_opacity,
                first_letter_font_style: run.first_letter_font_style.clone(),
                character,
                run_index,
                run_offset,
            });
            index = end;
            run_offset += character.len_utf8();
        }
    }
    spans
}
