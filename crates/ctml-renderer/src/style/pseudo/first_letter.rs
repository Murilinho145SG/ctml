use std::collections::HashSet;
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};
use unicode_segmentation::UnicodeSegmentation;

use crate::style::TextTransform;

pub(in crate::style) fn style_first_letters(runs: &mut Vec<super::super::TextRun>) {
    let mut output = Vec::with_capacity(runs.len());
    let mut index = 0;
    let mut styled_blocks = HashSet::new();
    while index < runs.len() {
        let Some(block) = runs[index].first_letter_block else {
            output.push(runs[index].clone());
            index += 1;
            continue;
        };
        let mut end_index = index + 1;
        while end_index < runs.len() && runs[end_index].first_letter_block == Some(block) {
            end_index += 1;
        }
        let group = &runs[index..end_index];
        if styled_blocks.contains(&block) {
            output.extend(group.iter().cloned().map(ordinary_run));
            index = end_index;
            continue;
        }
        let text = group
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>();
        let Some(source_run) = group
            .iter()
            .find(|run| run.source_first_letter_character.is_some())
        else {
            output.extend(group.iter().cloned().map(ordinary_run));
            index = end_index;
            continue;
        };
        let source_character = source_run.source_first_letter_character.unwrap();
        let base_core = apply_text_transform(source_character, source_run.style.text_transform);
        let Some((core_start, core_end)) = core_range(&text, base_core.chars().count()) else {
            output.extend(group.iter().cloned().map(ordinary_run));
            index = end_index;
            continue;
        };
        let start = preceding_punctuation_start(&text, core_start);
        let end = following_punctuation_end(&text, core_end);
        styled_blocks.insert(block);
        let mut group_offset = 0;
        for run in group {
            let run_start = group_offset;
            let run_end = run_start + run.text.len();
            group_offset = run_end;
            let selected_start = start.max(run_start);
            let selected_end = end.min(run_end);
            if selected_start >= selected_end {
                output.push(ordinary_run(run.clone()));
                continue;
            }
            if selected_start > run_start {
                let mut prefix = run.clone();
                prefix.text = run.text[..selected_start - run_start].to_owned();
                prefix.source_first_letter_character = None;
                prefix.first_letter_font_style = None;
                clear_first_letter_paints(&mut prefix);
                output.push(prefix);
            }
            let mut selected = run.clone();
            let local_start = selected_start - run_start;
            let local_end = selected_end - run_start;
            selected.text = if core_start >= selected_start && core_start < selected_end {
                let transform = run
                    .first_letter_font_style
                    .as_ref()
                    .map_or(run.style.text_transform, |style| style.text_transform);
                format!(
                    "{}{}{}",
                    &run.text[local_start..core_start - run_start],
                    apply_text_transform(source_character, transform),
                    &run.text[core_end - run_start..local_end]
                )
            } else {
                run.text[local_start..local_end].to_owned()
            };
            if let Some(pseudo_style) = &run.first_letter_font_style {
                apply_first_letter_style(&mut selected.style, pseudo_style);
            }
            selected.source_first_letter_character = None;
            output.push(selected);
            if selected_end < run_end {
                let mut suffix = run.clone();
                suffix.text = run.text[local_end..].to_owned();
                suffix.source_first_letter_character = None;
                suffix.first_letter_font_style = None;
                clear_first_letter_paints(&mut suffix);
                output.push(suffix);
            }
        }
        index = end_index;
    }
    *runs = output;
}

fn clear_first_letter_paints(run: &mut super::super::TextRun) {
    run.first_letter_block = None;
    run.first_letter_foreground = None;
    run.first_letter_background = None;
    run.first_letter_decoration = None;
    run.first_letter_opacity = None;
}

fn ordinary_run(mut run: super::super::TextRun) -> super::super::TextRun {
    run.source_first_letter_character = None;
    run.first_letter_font_style = None;
    clear_first_letter_paints(&mut run);
    run
}

fn apply_first_letter_style(
    style: &mut crate::style::TextStyle,
    pseudo_style: &crate::style::TextStyle,
) {
    style.family.clone_from(&pseudo_style.family);
    style.font_size = pseudo_style.font_size;
    style.font_scale = pseudo_style.font_scale;
    style.weight = pseudo_style.weight;
    style.font_style = pseudo_style.font_style;
    style.stretch = pseudo_style.stretch;
    style.kerning = pseudo_style.kerning;
    style.common_ligatures = pseudo_style.common_ligatures;
    style.discretionary_ligatures = pseudo_style.discretionary_ligatures;
    style.historical_ligatures = pseudo_style.historical_ligatures;
    style.contextual_alternates = pseudo_style.contextual_alternates;
    style.font_variant_caps = pseudo_style.font_variant_caps;
    style.font_variant_position = pseudo_style.font_variant_position;
    style.text_rendering = pseudo_style.text_rendering;
    style.font_optical_sizing = pseudo_style.font_optical_sizing;
    style
        .font_variation_settings
        .clone_from(&pseudo_style.font_variation_settings);
    style
        .font_variant_numeric
        .clone_from(&pseudo_style.font_variant_numeric);
    style
        .font_variant_east_asian
        .clone_from(&pseudo_style.font_variant_east_asian);
    style
        .font_feature_settings
        .clone_from(&pseudo_style.font_feature_settings);
    style.line_height = pseudo_style.line_height;
    style.line_height_multiplier = pseudo_style.line_height_multiplier;
    style.line_height_normal = pseudo_style.line_height_normal;
    style.vertical_align_offset = pseudo_style.vertical_align_offset;
    style.letter_spacing = pseudo_style.letter_spacing;
    style.letter_spacing_px = pseudo_style.letter_spacing_px;
    style.word_spacing_px = pseudo_style.word_spacing_px;
    style.word_spacing_percent = pseudo_style.word_spacing_percent;
    style.word_spacing_space = pseudo_style.word_spacing_space;
    style.text_shadows.clone_from(&pseudo_style.text_shadows);
    style
        .text_emphasis_style
        .clone_from(&pseudo_style.text_emphasis_style);
    style.text_emphasis_color = pseudo_style.text_emphasis_color;
    style.text_emphasis_position = pseudo_style.text_emphasis_position;
}

fn core_range(text: &str, transformed_character_count: usize) -> Option<(usize, usize)> {
    let (start, grapheme) = text.grapheme_indices(true).find(|(_, grapheme)| {
        grapheme
            .chars()
            .next()
            .is_some_and(is_letter_number_or_symbol)
    })?;
    let end = if transformed_character_count == 1 {
        start + grapheme.len()
    } else {
        start
            + text[start..]
                .char_indices()
                .nth(transformed_character_count)
                .map_or(text[start..].len(), |(offset, _)| offset)
    };
    Some((start, end))
}

fn preceding_punctuation_start(text: &str, core_start: usize) -> usize {
    let mut start = core_start;
    for (index, character) in text[..core_start].char_indices().rev() {
        if is_punctuation(character) {
            start = index;
        } else if !is_typographic_space(character) {
            break;
        }
    }
    start
}

fn following_punctuation_end(text: &str, core_end: usize) -> usize {
    let mut end = core_end;
    for (offset, character) in text[core_end..].char_indices() {
        let index = core_end + offset;
        if is_trailing_punctuation(character) {
            end = index + character.len_utf8();
        } else if !is_typographic_space(character) {
            break;
        }
    }
    end
}

fn is_letter_number_or_symbol(character: char) -> bool {
    matches!(
        character.general_category(),
        GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter
            | GeneralCategory::DecimalNumber
            | GeneralCategory::LetterNumber
            | GeneralCategory::OtherNumber
            | GeneralCategory::MathSymbol
            | GeneralCategory::CurrencySymbol
            | GeneralCategory::ModifierSymbol
            | GeneralCategory::OtherSymbol
    )
}

fn is_punctuation(character: char) -> bool {
    matches!(
        character.general_category(),
        GeneralCategory::ConnectorPunctuation
            | GeneralCategory::DashPunctuation
            | GeneralCategory::OpenPunctuation
            | GeneralCategory::ClosePunctuation
            | GeneralCategory::InitialPunctuation
            | GeneralCategory::FinalPunctuation
            | GeneralCategory::OtherPunctuation
    )
}

fn is_trailing_punctuation(character: char) -> bool {
    is_punctuation(character)
        && !matches!(
            character.general_category(),
            GeneralCategory::OpenPunctuation | GeneralCategory::DashPunctuation
        )
}

fn is_typographic_space(character: char) -> bool {
    character != '\u{3000}' && character.general_category() == GeneralCategory::SpaceSeparator
}

fn apply_text_transform(character: char, transform: TextTransform) -> String {
    match transform {
        TextTransform::Uppercase | TextTransform::Capitalize => character.to_uppercase().collect(),
        TextTransform::Lowercase => character.to_lowercase().collect(),
        TextTransform::None => character.to_string(),
    }
}
