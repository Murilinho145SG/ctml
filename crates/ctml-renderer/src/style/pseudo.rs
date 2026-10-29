use std::collections::HashSet;

use crate::style::TextTransform;
use crate::{StyleDeclaration, StyleRule, css_style, style::TextStyle};
use ctml_css::{
    ElementIdentity, SelectorPosition, SelectorSibling, SelectorTreeElement,
    selector_matches_with_selector_tree_context, selector_specificity,
};

pub(super) struct GeneratedContent {
    pub(super) text: String,
    pub(super) style: TextStyle,
}

pub(super) fn style_first_letters(runs: &mut Vec<super::TextRun>) {
    let mut styled_blocks = HashSet::new();
    let mut output = Vec::with_capacity(runs.len());
    for run in runs.drain(..) {
        let Some(block) = run.first_letter_block else {
            output.push(run);
            continue;
        };
        if styled_blocks.contains(&block) {
            output.push(run);
            continue;
        }
        let Some((start, character)) = run
            .text
            .char_indices()
            .find(|(_, character)| !character.is_whitespace())
        else {
            output.push(run);
            continue;
        };
        let source_character = run.source_first_non_whitespace.unwrap_or(character);
        let original_transform = apply_text_transform(source_character, run.style.text_transform);
        let selected_character_count = original_transform.chars().count().max(1);
        let end = run.text[start..]
            .char_indices()
            .nth(selected_character_count)
            .map_or(run.text.len(), |(offset, _)| start + offset);
        if start > 0 {
            let mut prefix = run.clone();
            prefix.text = run.text[..start].to_owned();
            prefix.source_first_non_whitespace = None;
            prefix.first_letter_font_style = None;
            output.push(prefix);
        }
        let mut first_letter = run.clone();
        first_letter.text = apply_text_transform(
            source_character,
            run.first_letter_font_style
                .as_ref()
                .map_or(run.style.text_transform, |style| style.text_transform),
        );
        first_letter.source_first_non_whitespace = Some(source_character);
        if let Some(pseudo_style) = &run.first_letter_font_style {
            first_letter.style.opacity = pseudo_style.opacity;
            first_letter.style.family.clone_from(&pseudo_style.family);
            first_letter.style.font_size = pseudo_style.font_size;
            first_letter.style.font_scale = pseudo_style.font_scale;
            first_letter.style.weight = pseudo_style.weight;
            first_letter.style.font_style = pseudo_style.font_style;
            first_letter.style.stretch = pseudo_style.stretch;
            first_letter.style.kerning = pseudo_style.kerning;
            first_letter.style.common_ligatures = pseudo_style.common_ligatures;
            first_letter.style.discretionary_ligatures = pseudo_style.discretionary_ligatures;
            first_letter.style.historical_ligatures = pseudo_style.historical_ligatures;
            first_letter.style.contextual_alternates = pseudo_style.contextual_alternates;
            first_letter.style.font_variant_caps = pseudo_style.font_variant_caps;
            first_letter.style.font_variant_position = pseudo_style.font_variant_position;
            first_letter
                .style
                .font_variant_numeric
                .clone_from(&pseudo_style.font_variant_numeric);
            first_letter
                .style
                .font_variant_east_asian
                .clone_from(&pseudo_style.font_variant_east_asian);
            first_letter
                .style
                .font_feature_settings
                .clone_from(&pseudo_style.font_feature_settings);
            first_letter.style.line_height = pseudo_style.line_height;
            first_letter.style.line_height_multiplier = pseudo_style.line_height_multiplier;
            first_letter.style.line_height_normal = pseudo_style.line_height_normal;
            first_letter.style.vertical_align_offset = pseudo_style.vertical_align_offset;
            first_letter.style.letter_spacing = pseudo_style.letter_spacing;
            first_letter.style.letter_spacing_px = pseudo_style.letter_spacing_px;
            first_letter
                .style
                .text_shadows
                .clone_from(&pseudo_style.text_shadows);
            first_letter
                .style
                .text_emphasis_style
                .clone_from(&pseudo_style.text_emphasis_style);
            first_letter.style.text_emphasis_color = pseudo_style.text_emphasis_color;
            first_letter.style.text_emphasis_position = pseudo_style.text_emphasis_position;
        }
        first_letter.first_letter_font_style = None;
        output.push(first_letter);
        if end < run.text.len() {
            let mut suffix = run.clone();
            suffix.text = run.text[end..].to_owned();
            suffix.first_letter_font_style = None;
            output.push(suffix);
        }
        styled_blocks.insert(block);
    }
    *runs = output;
}

fn apply_text_transform(character: char, transform: TextTransform) -> String {
    match transform {
        TextTransform::Uppercase | TextTransform::Capitalize => character.to_uppercase().collect(),
        TextTransform::Lowercase => character.to_lowercase().collect(),
        TextTransform::None => character.to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_generated_content(
    pseudo_element: &str,
    inherited: &TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    siblings_before_current: &[SelectorSibling],
    siblings_before_ancestors: &[Vec<SelectorSibling>],
    current_position: SelectorPosition,
    ancestor_positions: &[SelectorPosition],
    selector_tree: &[SelectorTreeElement],
    stylesheet: &[StyleRule],
) -> Option<GeneratedContent> {
    let mut pseudo_rules = Vec::new();
    let mut winning_content = None;
    for (rule_index, rule) in stylesheet.iter().enumerate() {
        let mut base_selectors = Vec::new();
        let mut matching_specificity = None;
        for selector in &rule.selectors {
            let Some(base_selector) = strip_pseudo_element(selector, pseudo_element) else {
                continue;
            };
            if !selector_matches_with_selector_tree_context(
                &base_selector,
                current,
                ancestors,
                attributes,
                ancestor_attributes,
                siblings_before_current,
                siblings_before_ancestors,
                current_position,
                ancestor_positions,
                selector_tree,
            ) {
                continue;
            }
            let specificity = add_pseudo_element_specificity(selector_specificity(&base_selector));
            matching_specificity = Some(
                matching_specificity.map_or(specificity, |current: (usize, usize, usize)| {
                    current.max(specificity)
                }),
            );
            base_selectors.push(base_selector);
        }
        let Some(specificity) = matching_specificity else {
            continue;
        };
        pseudo_rules.push(StyleRule {
            selectors: base_selectors,
            declarations: rule.declarations.clone(),
            layer: rule.layer,
        });
        for (declaration_index, declaration) in rule.declarations.iter().enumerate() {
            if !declaration.property.eq_ignore_ascii_case("content") {
                continue;
            }
            let (_, important) = declaration_value(&declaration.value);
            let priority = (important, specificity, rule_index, declaration_index);
            if winning_content.as_ref().is_none_or(
                |(current_priority, _): &(
                    (bool, (usize, usize, usize), usize, usize),
                    &StyleDeclaration,
                )| { priority >= *current_priority },
            ) {
                winning_content = Some((priority, declaration));
            }
        }
    }
    let (_, declaration) = winning_content?;
    let (value, _) = declaration_value(&declaration.value);
    let text = parse_string_content(value)?;
    let style = css_style::resolve_style_with_selector_tree_context(
        inherited,
        current,
        ancestors,
        attributes,
        ancestor_attributes,
        siblings_before_current,
        siblings_before_ancestors,
        current_position,
        ancestor_positions,
        selector_tree,
        None,
        &pseudo_rules,
    );
    Some(GeneratedContent { text, style })
}

fn strip_pseudo_element(selector: &str, pseudo_element: &str) -> Option<String> {
    let selector = selector.trim_end();
    let expected_suffix = format!("::{pseudo_element}");
    let legacy_suffix = format!(":{pseudo_element}");
    let suffix = [expected_suffix.as_str(), legacy_suffix.as_str()]
        .into_iter()
        .find(|suffix| selector.to_ascii_lowercase().ends_with(suffix))?;
    let suffix_start = selector.len().checked_sub(suffix.len())?;
    let escaped_colon = selector[..suffix_start]
        .bytes()
        .rev()
        .take_while(|byte| *byte == b'\\')
        .count()
        % 2
        == 1;
    if escaped_colon {
        return None;
    }
    let base = selector.get(..suffix_start)?;
    if !selector.get(base.len()..)?.eq_ignore_ascii_case(suffix) {
        return None;
    }
    let trimmed_base = base.trim_end();
    if trimmed_base.is_empty() {
        return None;
    }
    Some(if trimmed_base.len() != base.len() {
        format!("{trimmed_base} *")
    } else {
        trimmed_base.to_owned()
    })
}

fn add_pseudo_element_specificity(
    (ids, classes, tags): (usize, usize, usize),
) -> (usize, usize, usize) {
    (ids, classes, tags + 1)
}

fn declaration_value(value: &str) -> (&str, bool) {
    let value = value.trim_end();
    let Some(without_priority) = value.get(..value.len().saturating_sub("!important".len())) else {
        return (value, false);
    };
    if value[without_priority.len()..]
        .trim()
        .eq_ignore_ascii_case("!important")
    {
        (without_priority.trim_end(), true)
    } else {
        (value, false)
    }
}

fn parse_string_content(value: &str) -> Option<String> {
    let value = value.trim();
    let quote = value.chars().next()?;
    if !matches!(quote, '\'' | '"') || value.chars().last()? != quote {
        return None;
    }
    if value.len() < quote.len_utf8() * 2 {
        return None;
    }
    let inner = &value[quote.len_utf8()..value.len() - quote.len_utf8()];
    let mut output = String::with_capacity(inner.len());
    let mut characters = inner.chars();
    while let Some(character) = characters.next() {
        if character == '\\' {
            output.push(characters.next()?);
        } else {
            output.push(character);
        }
    }
    Some(output)
}
