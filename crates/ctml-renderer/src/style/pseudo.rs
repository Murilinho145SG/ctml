use crate::{StyleDeclaration, StyleRule, css_style, style::TextStyle};
use ctml_css::{
    ElementIdentity, SelectorPosition, SelectorSibling, SelectorTreeElement,
    selector_matches_with_selector_tree_context, selector_specificity,
};

pub(super) struct GeneratedContent {
    pub(super) text: String,
    pub(super) style: TextStyle,
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
