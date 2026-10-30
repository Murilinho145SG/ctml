use crate::{
    StyleRule, UiNode,
    css_style::{self, ElementIdentity},
};
use ctml_css::{
    SelectorPosition, SelectorSibling, SelectorTreeAncestor, SelectorTreeElement, ViewportSize,
};
use std::collections::HashMap;

mod collect;
mod font_units;
mod list_marker;
mod pseudo;
mod text;
mod text_style;
pub(super) use text::{append_text, push_character, push_line_break};
pub(super) use text_style::{
    FontVariantCaps, FontVariantPosition, Hyphens, LayoutHeight, LayoutSize, LineBreak,
    OverflowWrap, TextDecorationSkipInk, TextDecorationStyle, TextEmphasisPosition,
    TextEmphasisStyle, TextOrientation, TextOverflow, TextRendering, TextShadow, TextStyle,
    TextTransform, WhiteSpace, WordBreak,
};

pub(super) fn compile_runs_with_font_system(
    nodes: &[UiNode],
    stylesheet: &[StyleRule],
    hovered_path: &[usize],
    active_path: &[usize],
    focused_path: &[usize],
    keyboard_focus: bool,
    font_system: Option<&cosmic_text::FontSystem>,
    viewport_size: ViewportSize,
) -> (Vec<TextRun>, TextStyle) {
    let font_metrics = font_system.map(crate::font_metrics::FontRelativeUnitsContext::new);
    let root_position = SelectorPosition {
        node_id: 0,
        child_count: nodes
            .iter()
            .filter(|node| matches!(node, UiNode::Element { .. }))
            .count(),
        is_empty: nodes.is_empty(),
        is_root: true,
        is_hovered: !hovered_path.is_empty(),
        is_active: !active_path.is_empty(),
        is_focused: false,
        is_focus_within: !focused_path.is_empty(),
        ..SelectorPosition::default()
    };
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let root_tree_element = SelectorTreeElement {
        node_id: 0,
        parent_id: None,
        element: body.clone(),
        attributes: Vec::new(),
        position: root_position,
        siblings_before: Vec::new(),
        ancestors: Vec::new(),
    };
    let mut selector_tree = vec![root_tree_element];
    let root_ancestor = SelectorTreeAncestor {
        element: body.clone(),
        attributes: Vec::new(),
        position: root_position,
        siblings_before: Vec::new(),
    };
    let mut tree_next_node_id = 1;
    build_selector_tree(
        nodes,
        0,
        &[root_ancestor],
        &mut tree_next_node_id,
        hovered_path,
        active_path,
        focused_path,
        keyboard_focus,
        &mut selector_tree,
    );
    let root_style = font_units::resolve_root_style(
        &body,
        root_position,
        &selector_tree,
        stylesheet,
        font_metrics.as_ref(),
        viewport_size,
    );
    let mut runs = Vec::new();
    let mut path = vec![body];
    let mut path_attributes = vec![Vec::new()];
    let mut path_siblings = vec![Vec::new()];
    let mut path_positions = vec![root_position];
    let mut element_path = Vec::new();
    let mut next_node_id = 1;
    let mut last_was_space = true;
    let mut capitalize_next = true;
    let mut quote_depth = 0;
    let mut counters = HashMap::new();
    if !root_style.display_none {
        collect::collect_runs(
            nodes,
            stylesheet,
            &mut path,
            &mut path_attributes,
            &mut path_siblings,
            &mut path_positions,
            &mut element_path,
            &mut next_node_id,
            hovered_path,
            active_path,
            focused_path,
            keyboard_focus,
            &selector_tree,
            &root_style,
            font_metrics.as_ref(),
            &mut runs,
            &mut last_was_space,
            &mut capitalize_next,
            &mut quote_depth,
            &mut counters,
        );
    }
    trim_runs(&mut runs);
    let selection_background = css_style::selection_background_color(stylesheet);
    for run in &mut runs {
        let Some(node_id) = run.element_path.last() else {
            continue;
        };
        let Some(element) = selector_tree
            .iter()
            .find(|element| element.node_id == *node_id)
        else {
            continue;
        };
        let selected_style = css_style::resolve_selection_style(
            &run.style,
            &element.element,
            element.position,
            &selector_tree,
            stylesheet,
            selection_background,
        );
        run.selection_background = selected_style.background_color;
        run.selection_foreground =
            (selected_style.color != run.style.color).then_some(selected_style.color);
        let block = run
            .element_path
            .iter()
            .rev()
            .filter_map(|node_id| {
                selector_tree
                    .iter()
                    .find(|element| element.node_id == *node_id)
            })
            .find(|element| collect::is_block(&element.element.tag));
        if let Some(block) = block {
            run.first_line_block = Some(block.node_id);
            let first_line = css_style::first_line_style(
                &run.style,
                &block.element,
                block.position,
                &selector_tree,
                stylesheet,
            );
            run.first_line_foreground = first_line.foreground;
            run.first_line_background = first_line.background;
            run.first_line_decoration = first_line.decoration;
            run.first_line_opacity = first_line.opacity;
            let mut first_letter_inherited = first_line.resolved.clone();
            first_letter_inherited.opacity = run.style.opacity;
            first_letter_inherited.vertical_align_offset = run.style.vertical_align_offset;
            run.first_line_font_style = first_line
                .has_rendered_text_style
                .then_some(first_line.resolved);
            run.first_letter_block = Some(block.node_id);
            let first_letter = css_style::first_letter_style(
                &first_letter_inherited,
                &block.element,
                block.position,
                &selector_tree,
                stylesheet,
            );
            run.first_letter_foreground = first_letter.foreground;
            run.first_letter_background = first_letter.background;
            run.first_letter_decoration = first_letter.decoration;
            run.first_letter_opacity = first_letter.opacity;
            run.first_letter_font_style = first_letter
                .has_rendered_text_style
                .then_some(first_letter.resolved);
        }
    }
    pseudo::style_first_letters(&mut runs);
    (runs, root_style)
}

#[derive(Clone)]
pub(super) struct TextRun {
    pub(super) text: String,
    pub(super) source_first_letter_character: Option<char>,
    pub(super) style: TextStyle,
    pub(super) element_path: Vec<usize>,
    pub(super) selection_background: Option<[u8; 4]>,
    pub(super) selection_foreground: Option<[u8; 4]>,
    pub(super) first_line_block: Option<usize>,
    pub(super) first_line_foreground: Option<[u8; 4]>,
    pub(super) first_line_background: Option<[u8; 4]>,
    pub(super) first_line_decoration: Option<crate::css_style::PseudoDecoration>,
    pub(super) first_line_opacity: Option<f32>,
    pub(super) first_line_font_style: Option<TextStyle>,
    pub(super) first_letter_block: Option<usize>,
    pub(super) first_letter_foreground: Option<[u8; 4]>,
    pub(super) first_letter_background: Option<[u8; 4]>,
    pub(super) first_letter_decoration: Option<crate::css_style::PseudoDecoration>,
    pub(super) first_letter_opacity: Option<f32>,
    pub(super) first_letter_font_style: Option<TextStyle>,
}

fn selector_sibling_contexts(
    nodes: &[UiNode],
    next_node_id: &mut usize,
    hovered_path: &[usize],
    active_path: &[usize],
    focused_path: &[usize],
    keyboard_focus: bool,
) -> Vec<SelectorSibling> {
    let elements = nodes
        .iter()
        .filter_map(|node| match node {
            UiNode::Element {
                tag,
                class,
                id,
                attributes,
                children,
            } => Some((tag, class, id, attributes, children)),
            UiNode::Text(_) => None,
        })
        .collect::<Vec<_>>();
    elements
        .iter()
        .enumerate()
        .map(|(index, (tag, class, id, attributes, children))| {
            let type_index = elements[..=index]
                .iter()
                .filter(|(previous_tag, _, _, _, _)| previous_tag.eq_ignore_ascii_case(tag))
                .count();
            let type_count = elements
                .iter()
                .filter(|(other_tag, _, _, _, _)| other_tag.eq_ignore_ascii_case(tag))
                .count();
            let node_id = *next_node_id;
            *next_node_id += 1;
            SelectorSibling {
                element: ElementIdentity {
                    tag: (*tag).clone(),
                    class: (*class).clone(),
                    id: (*id).clone(),
                },
                attributes: attributes.to_vec(),
                position: SelectorPosition {
                    node_id,
                    child_index: index + 1,
                    child_count: elements.len(),
                    type_index,
                    type_count,
                    is_empty: children
                        .iter()
                        .all(|child| matches!(child, UiNode::Text(text) if text.is_empty())),
                    is_root: false,
                    is_hovered: hovered_path.contains(&node_id),
                    is_active: active_path.contains(&node_id),
                    is_focused: focused_path.last() == Some(&node_id),
                    is_focus_visible: keyboard_focus && focused_path.last() == Some(&node_id),
                    is_focus_within: focused_path.contains(&node_id),
                },
                node_id,
            }
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn build_selector_tree(
    nodes: &[UiNode],
    parent_id: usize,
    ancestors: &[SelectorTreeAncestor],
    next_node_id: &mut usize,
    hovered_path: &[usize],
    active_path: &[usize],
    focused_path: &[usize],
    keyboard_focus: bool,
    output: &mut Vec<SelectorTreeElement>,
) {
    let siblings = selector_sibling_contexts(
        nodes,
        next_node_id,
        hovered_path,
        active_path,
        focused_path,
        keyboard_focus,
    );
    let children = nodes
        .iter()
        .filter_map(|node| match node {
            UiNode::Element { children, .. } => Some(children),
            UiNode::Text(_) => None,
        })
        .collect::<Vec<_>>();
    for (index, sibling) in siblings.iter().enumerate() {
        output.push(SelectorTreeElement {
            node_id: sibling.node_id,
            parent_id: Some(parent_id),
            element: sibling.element.clone(),
            attributes: sibling.attributes.clone(),
            position: sibling.position,
            siblings_before: siblings[..index].to_vec(),
            ancestors: ancestors.to_vec(),
        });
    }
    for (index, child_nodes) in children.into_iter().enumerate() {
        let sibling = &siblings[index];
        let mut child_ancestors = ancestors.to_vec();
        child_ancestors.push(SelectorTreeAncestor {
            element: sibling.element.clone(),
            attributes: sibling.attributes.clone(),
            position: sibling.position,
            siblings_before: siblings[..index].to_vec(),
        });
        build_selector_tree(
            child_nodes,
            sibling.node_id,
            &child_ancestors,
            next_node_id,
            hovered_path,
            active_path,
            focused_path,
            keyboard_focus,
            output,
        );
    }
}

pub(super) fn synthesize_font_variant_caps(run: TextRun) -> Vec<TextRun> {
    let mut output = Vec::new();
    for character in run.text.chars() {
        let synthesize = match run.style.font_variant_caps {
            FontVariantCaps::SmallCaps | FontVariantCaps::PetiteCaps => character.is_lowercase(),
            FontVariantCaps::AllSmallCaps | FontVariantCaps::AllPetiteCaps => {
                character.is_alphabetic()
            }
            _ => false,
        };
        let mut style = run.style.clone();
        style.font_variant_caps = FontVariantCaps::Normal;
        if synthesize {
            let original_size = style.font_size * style.font_scale;
            let synthesized_size = (original_size * 0.7).round();
            style.font_scale = if style.font_size > 0.0 {
                synthesized_size / style.font_size
            } else {
                0.0
            };
            for uppercase in character.to_uppercase() {
                push_character(uppercase, &style, &run.element_path, &mut output);
            }
        } else {
            push_character(character, &style, &run.element_path, &mut output);
        }
    }
    for synthesized in &mut output {
        synthesized.selection_background = run.selection_background;
        synthesized.selection_foreground = run.selection_foreground;
        synthesized.first_line_block = run.first_line_block;
        synthesized.first_line_foreground = run.first_line_foreground;
        synthesized.first_line_background = run.first_line_background;
        synthesized.first_line_decoration = run.first_line_decoration;
        synthesized.first_line_font_style = run.first_line_font_style.clone();
        synthesized.first_letter_block = run.first_letter_block;
        synthesized.first_letter_foreground = run.first_letter_foreground;
        synthesized.first_letter_background = run.first_letter_background;
        synthesized.first_letter_decoration = run.first_letter_decoration;
        synthesized.first_letter_font_style = run.first_letter_font_style.clone();
    }
    output
}

fn trim_runs(runs: &mut Vec<TextRun>) {
    while runs.first().is_some_and(|run| {
        !keeps_edge_whitespace(run.style.white_space) && run.text.chars().all(char::is_whitespace)
    }) {
        runs.remove(0);
    }
    while runs.last().is_some_and(|run| {
        !keeps_edge_whitespace(run.style.white_space) && run.text.chars().all(char::is_whitespace)
    }) {
        runs.pop();
    }
    if let Some(last) = runs.last_mut()
        && !keeps_edge_whitespace(last.style.white_space)
    {
        let trimmed = last.text.trim_end_matches(' ').to_owned();
        last.text = trimmed;
    }
    runs.retain(|run| !run.text.is_empty());
}

fn keeps_edge_whitespace(white_space: WhiteSpace) -> bool {
    matches!(
        white_space,
        WhiteSpace::Pre | WhiteSpace::PreWrap | WhiteSpace::PreLine | WhiteSpace::BreakSpaces
    )
}

#[cfg(test)]
#[path = "style_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "style_list_tests.rs"]
mod list_tests;
