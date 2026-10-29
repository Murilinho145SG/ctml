mod list;
mod specificity;

pub use specificity::selector_specificity;

#[derive(Clone)]
pub struct ElementIdentity {
    pub tag: String,
    pub class: Option<String>,
    pub id: Option<String>,
}

#[derive(Clone)]
pub struct SelectorSibling {
    pub element: ElementIdentity,
    pub attributes: Vec<(String, Option<String>)>,
    pub position: SelectorPosition,
}

#[derive(Clone, Copy, Default)]
pub struct SelectorPosition {
    pub child_index: usize,
    pub child_count: usize,
    pub type_index: usize,
    pub type_count: usize,
    pub is_empty: bool,
    pub is_root: bool,
}

pub fn selector_matches(
    selector: &str,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
) -> bool {
    selector_matches_with_attributes(selector, current, ancestors, &[])
}

pub fn selector_matches_with_attributes(
    selector: &str,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
) -> bool {
    selector_matches_with_context(selector, current, ancestors, attributes, &[])
}

pub fn selector_matches_with_context(
    selector: &str,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
) -> bool {
    selector_matches_with_tree_context(
        selector,
        current,
        ancestors,
        attributes,
        ancestor_attributes,
        &[],
        &[],
        SelectorPosition::default(),
        &[],
    )
}

pub fn selector_matches_with_tree_context(
    selector: &str,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    siblings_before_current: &[SelectorSibling],
    siblings_before_ancestors: &[Vec<SelectorSibling>],
    current_position: SelectorPosition,
    ancestor_positions: &[SelectorPosition],
) -> bool {
    let Some(compounds) = selector_compounds(selector) else {
        return false;
    };
    match_selector_chain(
        &compounds,
        compounds.len().checked_sub(1),
        current,
        attributes,
        ancestors,
        ancestor_attributes,
        siblings_before_current,
        siblings_before_ancestors,
        current_position,
        ancestor_positions,
    )
}

#[allow(clippy::too_many_arguments)]
fn match_selector_chain(
    compounds: &[(String, SelectorRelation)],
    index: Option<usize>,
    current: &ElementIdentity,
    attributes: &[(String, Option<String>)],
    ancestors: &[ElementIdentity],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    siblings_before_current: &[SelectorSibling],
    siblings_before_ancestors: &[Vec<SelectorSibling>],
    current_position: SelectorPosition,
    ancestor_positions: &[SelectorPosition],
) -> bool {
    let Some(index) = index else {
        return false;
    };
    if !compound_matches(
        &compounds[index].0,
        current,
        attributes,
        ancestors,
        ancestor_attributes,
        siblings_before_current,
        siblings_before_ancestors,
        current_position,
        ancestor_positions,
    ) {
        return false;
    }
    if index == 0 {
        return true;
    }
    let recurse = |candidate: &SelectorSibling, prior: &[SelectorSibling]| {
        match_selector_chain(
            compounds,
            Some(index - 1),
            &candidate.element,
            &candidate.attributes,
            ancestors,
            ancestor_attributes,
            prior,
            &siblings_before_ancestors[..ancestors.len().min(siblings_before_ancestors.len())],
            candidate.position,
            ancestor_positions,
        )
    };
    match compounds[index].1 {
        SelectorRelation::Child => ancestors.len().checked_sub(1).is_some_and(|parent_index| {
            match_selector_chain(
                compounds,
                Some(index - 1),
                &ancestors[parent_index],
                ancestor_attributes
                    .get(parent_index)
                    .map_or(&[], Vec::as_slice),
                &ancestors[..parent_index],
                &ancestor_attributes[..parent_index.min(ancestor_attributes.len())],
                siblings_before_ancestors
                    .get(parent_index)
                    .map_or(&[], Vec::as_slice),
                &siblings_before_ancestors[..parent_index.min(siblings_before_ancestors.len())],
                ancestor_positions
                    .get(parent_index)
                    .copied()
                    .unwrap_or_default(),
                &ancestor_positions[..parent_index.min(ancestor_positions.len())],
            )
        }),
        SelectorRelation::Descendant => (0..ancestors.len()).rev().any(|ancestor_index| {
            match_selector_chain(
                compounds,
                Some(index - 1),
                &ancestors[ancestor_index],
                ancestor_attributes
                    .get(ancestor_index)
                    .map_or(&[], Vec::as_slice),
                &ancestors[..ancestor_index],
                &ancestor_attributes[..ancestor_index.min(ancestor_attributes.len())],
                siblings_before_ancestors
                    .get(ancestor_index)
                    .map_or(&[], Vec::as_slice),
                &siblings_before_ancestors[..ancestor_index.min(siblings_before_ancestors.len())],
                ancestor_positions
                    .get(ancestor_index)
                    .copied()
                    .unwrap_or_default(),
                &ancestor_positions[..ancestor_index.min(ancestor_positions.len())],
            )
        }),
        SelectorRelation::AdjacentSibling => {
            siblings_before_current.last().is_some_and(|candidate| {
                recurse(
                    candidate,
                    &siblings_before_current[..siblings_before_current.len() - 1],
                )
            })
        }
        SelectorRelation::GeneralSibling => {
            siblings_before_current
                .iter()
                .enumerate()
                .rev()
                .any(|(candidate_index, candidate)| {
                    recurse(candidate, &siblings_before_current[..candidate_index])
                })
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum SelectorRelation {
    Descendant,
    Child,
    AdjacentSibling,
    GeneralSibling,
}

pub(super) fn selector_compounds(selector: &str) -> Option<Vec<(String, SelectorRelation)>> {
    let mut compounds = Vec::new();
    let mut start = None;
    let mut relation = SelectorRelation::Descendant;
    let mut bracket_depth = 0;
    let mut paren_depth = 0;
    let mut quote = None;
    for (index, character) in selector.char_indices() {
        if paren_depth > 0 {
            match (quote, character) {
                (Some(current), character) if current == character => quote = None,
                (None, '"' | '\'') => quote = Some(character),
                (None, '(') => paren_depth += 1,
                (None, ')') => paren_depth -= 1,
                _ => {}
            }
            continue;
        }
        if bracket_depth > 0 {
            match (quote, character) {
                (Some(current), character) if current == character => quote = None,
                (None, '"' | '\'') => quote = Some(character),
                (None, '[') => bracket_depth += 1,
                (None, ']') => bracket_depth -= 1,
                _ => {}
            }
            continue;
        }
        if character == '[' {
            bracket_depth = 1;
            if start.is_none() {
                start = Some(index);
            }
            continue;
        }
        if character == '(' {
            paren_depth = 1;
            continue;
        }
        if character.is_ascii_whitespace() || matches!(character, '>' | '+' | '~') {
            if let Some(compound_start) = start.take() {
                compounds.push((selector[compound_start..index].to_owned(), relation));
                relation = SelectorRelation::Descendant;
            } else if matches!(character, '>' | '+' | '~')
                && (compounds.is_empty() || !matches!(relation, SelectorRelation::Descendant))
            {
                return None;
            }
            relation = match character {
                '>' => SelectorRelation::Child,
                '+' => SelectorRelation::AdjacentSibling,
                '~' => SelectorRelation::GeneralSibling,
                _ => relation,
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(compound_start) = start {
        compounds.push((selector[compound_start..].to_owned(), relation));
        Some(compounds)
    } else if matches!(relation, SelectorRelation::Descendant) {
        Some(compounds)
    } else {
        None
    }
}

mod compound;

use compound::compound_matches;
