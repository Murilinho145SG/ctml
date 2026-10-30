use std::collections::HashMap;
use std::collections::HashSet;

use cssparser::{Parser, ParserInput, Token};

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
            first_letter.style.font_optical_sizing = pseudo_style.font_optical_sizing;
            first_letter
                .style
                .font_variation_settings
                .clone_from(&pseudo_style.font_variation_settings);
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
    quote_depth: &mut usize,
    counters: &HashMap<String, Vec<i32>>,
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
    let mut counter_inherited = inherited.clone();
    counter_inherited.counter_reset.clear();
    counter_inherited.counter_increment.clear();
    counter_inherited.counter_set.clear();
    let mut style = css_style::resolve_style_with_selector_tree_context(
        &counter_inherited,
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
    let mut updated_quote_depth = *quote_depth;
    let text = parse_generated_content(
        value,
        attributes,
        &style.quotes,
        &mut updated_quote_depth,
        counters,
    )?;
    *quote_depth = updated_quote_depth;
    style.opacity *= inherited.opacity;
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

fn parse_generated_content(
    value: &str,
    attributes: &[(String, Option<String>)],
    quotes: &[(String, String)],
    quote_depth: &mut usize,
    counters: &HashMap<String, Vec<i32>>,
) -> Option<String> {
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    let mut output = String::new();
    while !parser.is_exhausted() {
        let token = parser.next().ok()?;
        match token {
            Token::QuotedString(text) => output.push_str(text),
            Token::Ident(name) if name.eq_ignore_ascii_case("open-quote") => {
                if let Some((open, _)) = quote_at(quotes, *quote_depth) {
                    output.push_str(open);
                }
                *quote_depth = quote_depth.saturating_add(1);
            }
            Token::Ident(name) if name.eq_ignore_ascii_case("close-quote") => {
                *quote_depth = quote_depth.saturating_sub(1);
                if let Some((_, close)) = quote_at(quotes, *quote_depth) {
                    output.push_str(close);
                }
            }
            Token::Ident(name) if name.eq_ignore_ascii_case("no-open-quote") => {
                *quote_depth = quote_depth.saturating_add(1);
            }
            Token::Ident(name) if name.eq_ignore_ascii_case("no-close-quote") => {
                *quote_depth = quote_depth.saturating_sub(1);
            }
            Token::Function(name) if name.eq_ignore_ascii_case("attr") => {
                let attribute_name = parser
                    .parse_nested_block(|nested| -> Result<_, cssparser::ParseError<()>> {
                        let name = nested
                            .expect_ident_cloned()
                            .map_err(|_| nested.new_custom_error(()))?;
                        nested.skip_whitespace();
                        if !nested.is_exhausted() {
                            return Err(nested.new_custom_error(()));
                        }
                        Ok(name)
                    })
                    .ok()?;
                if let Some((_, Some(attribute_value))) = attributes
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case(&attribute_name))
                {
                    output.push_str(attribute_value);
                }
            }
            Token::Function(name) if name.eq_ignore_ascii_case("counter") => {
                let (counter_name, style_name) =
                    parser.parse_nested_block(parse_counter_arguments).ok()?;
                if let Some(value) = counters.get(&counter_name).and_then(|values| values.last()) {
                    output.push_str(&super::list_marker::format_counter_value(
                        style_name.as_deref().unwrap_or("decimal"),
                        *value,
                    )?);
                }
            }
            Token::Function(name) if name.eq_ignore_ascii_case("counters") => {
                let (counter_name, separator, style_name) =
                    parser.parse_nested_block(parse_counters_arguments).ok()?;
                if let Some(values) = counters.get(&counter_name) {
                    let formatted = values
                        .iter()
                        .map(|value| {
                            super::list_marker::format_counter_value(
                                style_name.as_deref().unwrap_or("decimal"),
                                *value,
                            )
                        })
                        .collect::<Option<Vec<_>>>()?;
                    output.push_str(&formatted.join(&separator));
                }
            }
            Token::WhiteSpace(_) => {}
            _ => return None,
        }
    }
    Some(output)
}

fn parse_counter_arguments<'i, 't>(
    parser: &mut Parser<'i, 't>,
) -> Result<(String, Option<String>), cssparser::ParseError<'i, ()>> {
    let name = parser.expect_ident_cloned()?;
    parser.skip_whitespace();
    let style = if parser.is_exhausted() {
        None
    } else {
        parser.expect_comma()?;
        let style = parser.expect_ident_cloned()?;
        parser.skip_whitespace();
        Some(style.to_ascii_lowercase())
    };
    if !parser.is_exhausted() {
        return Err(parser.new_custom_error(()));
    }
    Ok((name.to_string(), style))
}

fn parse_counters_arguments<'i, 't>(
    parser: &mut Parser<'i, 't>,
) -> Result<(String, String, Option<String>), cssparser::ParseError<'i, ()>> {
    let name = parser.expect_ident_cloned()?;
    parser.expect_comma()?;
    let separator = match parser.next()?.clone() {
        Token::QuotedString(value) => value.to_string(),
        _ => return Err(parser.new_custom_error(())),
    };
    parser.skip_whitespace();
    let style = if parser.is_exhausted() {
        None
    } else {
        parser.expect_comma()?;
        let style = parser.expect_ident_cloned()?;
        parser.skip_whitespace();
        Some(style.to_ascii_lowercase())
    };
    if !parser.is_exhausted() {
        return Err(parser.new_custom_error(()));
    }
    Ok((name.to_string(), separator, style))
}

fn quote_at(quotes: &[(String, String)], depth: usize) -> Option<(&str, &str)> {
    quotes
        .get(depth.min(quotes.len().saturating_sub(1)))
        .map(|(open, close)| (open.as_str(), close.as_str()))
}
