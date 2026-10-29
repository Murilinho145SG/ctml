use super::{
    ElementIdentity, SelectorPosition, SelectorSibling, SelectorTreeAncestor, SelectorTreeElement,
    parse_declarations, parse_stylesheet, parse_stylesheet_file, selector_matches,
    selector_matches_with_attributes, selector_matches_with_context,
    selector_matches_with_selector_tree_context, selector_matches_with_tree_context,
    selector_specificity,
};

fn identity(tag: &str, class: Option<&str>) -> ElementIdentity {
    ElementIdentity {
        tag: tag.to_owned(),
        class: class.map(str::to_owned),
        id: None,
    }
}

fn position(node_id: usize, child_index: usize, child_count: usize) -> SelectorPosition {
    SelectorPosition {
        node_id,
        child_index,
        child_count,
        type_index: child_index,
        type_count: child_count,
        ..SelectorPosition::default()
    }
}

fn sibling(
    node_id: usize,
    element: ElementIdentity,
    position: SelectorPosition,
) -> SelectorSibling {
    SelectorSibling {
        element,
        attributes: Vec::new(),
        position,
        node_id,
    }
}

fn tree_ancestor(
    element: ElementIdentity,
    position: SelectorPosition,
    siblings_before: Vec<SelectorSibling>,
) -> SelectorTreeAncestor {
    SelectorTreeAncestor {
        element,
        attributes: Vec::new(),
        position,
        siblings_before,
    }
}

fn tree_element(
    node_id: usize,
    parent_id: Option<usize>,
    element: ElementIdentity,
    position: SelectorPosition,
    ancestors: Vec<SelectorTreeAncestor>,
) -> SelectorTreeElement {
    SelectorTreeElement {
        node_id,
        parent_id,
        element,
        attributes: Vec::new(),
        position,
        siblings_before: Vec::new(),
        ancestors,
    }
}

#[path = "tests/imports.rs"]
mod imports;
#[path = "tests/parser.rs"]
mod parser;
#[path = "tests/selectors_basic.rs"]
mod selectors_basic;
#[path = "tests/selectors_combinators.rs"]
mod selectors_combinators;
#[path = "tests/selectors_interaction.rs"]
mod selectors_interaction;
#[path = "tests/selectors_relational.rs"]
mod selectors_relational;
#[path = "tests/selectors_structural.rs"]
mod selectors_structural;
