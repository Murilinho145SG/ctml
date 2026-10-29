use std::collections::HashMap;

mod font_variant;
mod style_helpers;
mod variables;

use style_helpers::{
    SUPPORTED_ALL_PROPERTIES, canonical_property_name, cascade_custom_properties,
    declaration_value, is_all_keyword, is_custom_property, layer_cascade_priority, reverted_layers,
};
use variables::{contains_var_function, substitute_vars};

use crate::{StyleDeclaration, StyleRule, style::TextStyle};
pub(super) use ctml_css::ElementIdentity;
#[cfg(test)]
pub(super) use ctml_css::selector_matches;
use ctml_css::{
    SelectorPosition, SelectorSibling, SelectorTreeElement,
    selector_matches_with_selector_tree_context, selector_specificity,
};

type MatchedDeclaration<'a> = (
    bool,
    usize,
    Option<usize>,
    (usize, usize, usize),
    usize,
    usize,
    &'a StyleDeclaration,
);

#[cfg(test)]
pub(super) fn resolve_style(
    inherited: &TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    stylesheet: &[StyleRule],
) -> TextStyle {
    resolve_style_with_attributes(inherited, current, ancestors, &[], stylesheet)
}

#[cfg(test)]
pub(super) fn resolve_style_with_attributes(
    inherited: &TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
    stylesheet: &[StyleRule],
) -> TextStyle {
    resolve_style_with_context(inherited, current, ancestors, attributes, &[], stylesheet)
}

#[cfg(test)]
pub(super) fn resolve_style_with_context(
    inherited: &TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    stylesheet: &[StyleRule],
) -> TextStyle {
    resolve_style_with_inline_context(
        inherited,
        current,
        ancestors,
        attributes,
        ancestor_attributes,
        None,
        stylesheet,
    )
}

#[cfg(test)]
pub(super) fn resolve_style_with_inline_context(
    inherited: &TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    inline_style: Option<&str>,
    stylesheet: &[StyleRule],
) -> TextStyle {
    resolve_style_with_tree_inline_context(
        inherited,
        current,
        ancestors,
        attributes,
        ancestor_attributes,
        &[],
        &[],
        SelectorPosition::default(),
        &[],
        inline_style,
        stylesheet,
    )
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub(super) fn resolve_style_with_tree_inline_context(
    inherited: &TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    siblings_before_current: &[SelectorSibling],
    siblings_before_ancestors: &[Vec<SelectorSibling>],
    current_position: SelectorPosition,
    ancestor_positions: &[SelectorPosition],
    inline_style: Option<&str>,
    stylesheet: &[StyleRule],
) -> TextStyle {
    resolve_style_with_selector_tree_context(
        inherited,
        current,
        ancestors,
        attributes,
        ancestor_attributes,
        siblings_before_current,
        siblings_before_ancestors,
        current_position,
        ancestor_positions,
        &[],
        inline_style,
        stylesheet,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_style_with_selector_tree_context(
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
    inline_style: Option<&str>,
    stylesheet: &[StyleRule],
) -> TextStyle {
    let mut style = inherited.clone();
    apply_matching_styles_with_inline_context(
        &mut style,
        current,
        ancestors,
        attributes,
        ancestor_attributes,
        siblings_before_current,
        siblings_before_ancestors,
        current_position,
        ancestor_positions,
        selector_tree,
        inline_style,
        stylesheet,
    );
    style
}

#[cfg(test)]
pub(super) fn apply_matching_styles(
    style: &mut TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    stylesheet: &[StyleRule],
) {
    apply_matching_styles_with_context(style, current, ancestors, &[], &[], stylesheet);
}

pub(super) fn apply_matching_styles_with_selector_tree_position(
    style: &mut TextStyle,
    current: &ElementIdentity,
    position: SelectorPosition,
    selector_tree: &[SelectorTreeElement],
    stylesheet: &[StyleRule],
) {
    apply_matching_styles_inner(
        style,
        current,
        &[],
        &[],
        &[],
        &[],
        &[],
        position,
        &[],
        selector_tree,
        None,
        stylesheet,
    );
}

#[cfg(test)]
pub(super) fn apply_matching_styles_with_context(
    style: &mut TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    stylesheet: &[StyleRule],
) {
    apply_matching_styles_with_inline_context(
        style,
        current,
        ancestors,
        attributes,
        ancestor_attributes,
        &[],
        &[],
        SelectorPosition::default(),
        &[],
        &[],
        None,
        stylesheet,
    );
}

fn apply_matching_styles_with_inline_context(
    style: &mut TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    siblings_before_current: &[SelectorSibling],
    siblings_before_ancestors: &[Vec<SelectorSibling>],
    current_position: SelectorPosition,
    ancestor_positions: &[SelectorPosition],
    selector_tree: &[SelectorTreeElement],
    inline_style: Option<&str>,
    stylesheet: &[StyleRule],
) {
    apply_matching_styles_inner(
        style,
        current,
        ancestors,
        attributes,
        ancestor_attributes,
        siblings_before_current,
        siblings_before_ancestors,
        current_position,
        ancestor_positions,
        selector_tree,
        inline_style,
        stylesheet,
    );
}

#[cfg(test)]
pub(super) fn apply_matching_styles_with_attributes(
    style: &mut TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
    stylesheet: &[StyleRule],
) {
    apply_matching_styles_with_context(style, current, ancestors, attributes, &[], stylesheet);
}

fn apply_matching_styles_inner(
    style: &mut TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    siblings_before_current: &[SelectorSibling],
    siblings_before_ancestors: &[Vec<SelectorSibling>],
    current_position: SelectorPosition,
    ancestor_positions: &[SelectorPosition],
    selector_tree: &[SelectorTreeElement],
    inline_style: Option<&str>,
    stylesheet: &[StyleRule],
) {
    let inherited_style = style.clone();
    let mut matches: Vec<MatchedDeclaration<'_>> = Vec::new();
    let inline_declarations = inline_style
        .and_then(|inline_style| ctml_css::parse_declarations(inline_style).ok())
        .unwrap_or_default();
    for (rule_index, rule) in stylesheet.iter().enumerate() {
        let specificity = rule
            .selectors
            .iter()
            .filter(|selector| {
                selector_matches_with_selector_tree_context(
                    selector,
                    current,
                    ancestors,
                    attributes,
                    ancestor_attributes,
                    siblings_before_current,
                    siblings_before_ancestors,
                    current_position,
                    ancestor_positions,
                    selector_tree,
                )
            })
            .map(|selector| selector_specificity(selector))
            .max();
        if let Some(specificity) = specificity {
            for (declaration_index, declaration) in rule.declarations.iter().enumerate() {
                let (_, important) = declaration_value(&declaration.value);
                matches.push((
                    important,
                    layer_cascade_priority(rule.layer, important),
                    rule.layer,
                    specificity,
                    rule_index,
                    declaration_index,
                    declaration,
                ));
            }
        }
    }
    for (declaration_index, declaration) in inline_declarations.iter().enumerate() {
        let (_, important) = declaration_value(&declaration.value);
        matches.push((
            important,
            layer_cascade_priority(None, important),
            None,
            (usize::MAX, 0, 0),
            stylesheet.len(),
            declaration_index,
            declaration,
        ));
    }
    matches.sort_by_key(
        |(important, layer_priority, _, specificity, rule, declaration, _)| {
            (
                *important,
                *layer_priority,
                *specificity,
                *rule,
                *declaration,
            )
        },
    );
    let custom_properties = cascade_custom_properties(&inherited_style.custom_properties, &matches);

    let mut all_expansions = Vec::new();
    let expanded_matches = if matches
        .iter()
        .any(|entry| entry.6.property.eq_ignore_ascii_case("all"))
    {
        let mut expansion_ranges = vec![None; matches.len()];
        for (index, entry) in matches.iter().enumerate() {
            if !entry.6.property.eq_ignore_ascii_case("all") {
                continue;
            }
            let (value, _) = declaration_value(&entry.6.value);
            let Some(value) = substitute_vars(value, &custom_properties) else {
                continue;
            };
            if !is_all_keyword(&value) {
                continue;
            }
            let start = all_expansions.len();
            all_expansions.extend(SUPPORTED_ALL_PROPERTIES.iter().map(|property| {
                StyleDeclaration {
                    property: (*property).to_owned(),
                    value: value.clone(),
                }
            }));
            expansion_ranges[index] = Some(start..all_expansions.len());
        }
        let mut expanded = Vec::with_capacity(matches.len() + all_expansions.len());
        for (index, entry) in matches.iter().enumerate() {
            if let Some(range) = &expansion_ranges[index] {
                for declaration in &all_expansions[range.clone()] {
                    expanded.push((
                        entry.0,
                        entry.1,
                        entry.2,
                        entry.3,
                        entry.4,
                        entry.5,
                        declaration,
                    ));
                }
            } else if !entry.6.property.eq_ignore_ascii_case("all") {
                expanded.push(*entry);
            }
        }
        Some(expanded)
    } else {
        None
    };
    let matches = expanded_matches.as_deref().unwrap_or(&matches);

    let mut rollback_layers = HashMap::new();
    for entry in matches {
        let property = canonical_property_name(&entry.6.property);
        if !rollback_layers.contains_key(&property) {
            rollback_layers.insert(property.clone(), reverted_layers(matches, &property));
        }
    }
    style.custom_properties = custom_properties;

    let inherited_color = (style.color, style.color_opacity);
    for (important, _, layer, _, _, _, declaration) in matches
        .iter()
        .filter(|(_, _, _, _, _, _, declaration)| declaration.property == "color")
    {
        if rollback_layers
            .get("color")
            .is_some_and(|layers| layers.contains(&(*important, *layer)))
        {
            continue;
        }
        let (value, _) = declaration_value(&declaration.value);
        let Some(value) = substitute_vars(value, &style.custom_properties)
            .filter(|value| !value.trim().is_empty())
        else {
            apply_color_declaration(style, "unset", inherited_color);
            continue;
        };
        if !apply_color_declaration(style, &value, inherited_color)
            && contains_var_function(declaration_value(&declaration.value).0)
        {
            apply_color_declaration(style, "unset", inherited_color);
        }
    }
    for (important, _, layer, _, _, _, declaration) in
        matches.iter().filter(|(_, _, _, _, _, _, declaration)| {
            declaration.property != "color" && !is_custom_property(&declaration.property)
        })
    {
        if rollback_layers
            .get(&canonical_property_name(&declaration.property))
            .is_some_and(|layers| layers.contains(&(*important, *layer)))
        {
            continue;
        }
        let (value, _) = declaration_value(&declaration.value);
        let value = substitute_vars(value, &style.custom_properties)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "unset".to_owned());
        apply_declaration_with_inherited(style, &declaration.property, &value, &inherited_style);
    }
}

mod properties;
#[cfg(test)]
pub(super) use ctml_css::parse_length;
use properties::{apply_color_declaration, apply_declaration_with_inherited};

#[cfg(test)]
#[path = "css_style_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "css_style_font_variant_tests.rs"]
mod font_variant_tests;

#[cfg(test)]
#[path = "css_style_text_shadow_tests.rs"]
mod text_shadow_tests;

#[cfg(test)]
#[path = "css_style_text_emphasis_tests.rs"]
mod text_emphasis_tests;

#[cfg(test)]
#[path = "css_style_matrix_tests.rs"]
mod matrix_tests;
