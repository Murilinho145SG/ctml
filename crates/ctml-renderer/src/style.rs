use crate::{
    StyleRule, UiNode,
    css_style::{self, ElementIdentity},
};
use ctml_css::{SelectorPosition, SelectorSibling};

mod text_style;
pub(super) use text_style::{
    FontVariantCaps, FontVariantPosition, TextDecorationSkipInk, TextDecorationStyle, TextStyle,
    TextTransform,
};

pub(super) fn compile_runs(
    nodes: &[UiNode],
    stylesheet: &[StyleRule],
    hovered_path: &[usize],
    active_path: &[usize],
) -> (Vec<TextRun>, TextStyle) {
    let root_position = SelectorPosition {
        child_count: nodes
            .iter()
            .filter(|node| matches!(node, UiNode::Element { .. }))
            .count(),
        is_empty: nodes.is_empty(),
        is_root: true,
        is_hovered: !hovered_path.is_empty(),
        is_active: !active_path.is_empty(),
        ..SelectorPosition::default()
    };
    let mut root_style = TextStyle::default();
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    css_style::apply_matching_styles_with_position(
        &mut root_style,
        &body,
        root_position,
        stylesheet,
    );
    let mut runs = Vec::new();
    let mut path = vec![body];
    let mut path_attributes = vec![Vec::new()];
    let mut path_siblings = vec![Vec::new()];
    let mut path_positions = vec![root_position];
    let mut element_path = Vec::new();
    let mut next_node_id = 0;
    let mut last_was_space = true;
    let mut capitalize_next = true;
    collect_runs(
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
        &root_style,
        &mut runs,
        &mut last_was_space,
        &mut capitalize_next,
    );
    trim_runs(&mut runs);
    (runs, root_style)
}

pub(super) struct TextRun {
    pub(super) text: String,
    pub(super) style: TextStyle,
    pub(super) element_path: Vec<usize>,
}

fn collect_runs(
    nodes: &[UiNode],
    stylesheet: &[StyleRule],
    path: &mut Vec<ElementIdentity>,
    path_attributes: &mut Vec<Vec<(String, Option<String>)>>,
    path_siblings: &mut Vec<Vec<SelectorSibling>>,
    path_positions: &mut Vec<SelectorPosition>,
    element_path: &mut Vec<usize>,
    next_node_id: &mut usize,
    hovered_path: &[usize],
    active_path: &[usize],
    inherited: &TextStyle,
    output: &mut Vec<TextRun>,
    last_was_space: &mut bool,
    capitalize_next: &mut bool,
) {
    let siblings = selector_sibling_contexts(nodes, next_node_id, hovered_path, active_path);
    let mut element_index = 0;
    for node in nodes {
        match node {
            UiNode::Text(text) => append_text(
                text,
                inherited,
                element_path,
                output,
                last_was_space,
                capitalize_next,
            ),
            UiNode::Element {
                tag,
                class,
                id,
                attributes,
                children,
            } => {
                let sibling = &siblings[element_index];
                let position = sibling.position;
                let identity = ElementIdentity {
                    tag: tag.clone(),
                    class: class.clone(),
                    id: id.clone(),
                };
                let inline_style = attributes
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case("style"))
                    .and_then(|(_, value)| value.as_deref());
                let style = css_style::resolve_style_with_tree_inline_context(
                    inherited,
                    &identity,
                    path,
                    attributes,
                    path_attributes,
                    &siblings[..element_index],
                    path_siblings,
                    position,
                    path_positions,
                    inline_style,
                    stylesheet,
                );
                let block = is_block(tag);
                if block {
                    push_line_break(output, &style, element_path, last_was_space);
                    *capitalize_next = true;
                }
                path.push(identity);
                element_path.push(sibling.node_id);
                path_attributes.push(attributes.clone());
                path_siblings.push(siblings[..element_index].to_vec());
                path_positions.push(position);
                collect_runs(
                    children,
                    stylesheet,
                    path,
                    path_attributes,
                    path_siblings,
                    path_positions,
                    element_path,
                    next_node_id,
                    hovered_path,
                    active_path,
                    &style,
                    output,
                    last_was_space,
                    capitalize_next,
                );
                path.pop();
                element_path.pop();
                path_attributes.pop();
                path_siblings.pop();
                path_positions.pop();
                element_index += 1;
                if block {
                    push_line_break(output, &style, element_path, last_was_space);
                    *capitalize_next = true;
                }
            }
        }
    }
}

fn selector_sibling_contexts(
    nodes: &[UiNode],
    next_node_id: &mut usize,
    hovered_path: &[usize],
    active_path: &[usize],
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
                },
                node_id,
            }
        })
        .collect()
}

fn append_text(
    text: &str,
    style: &TextStyle,
    element_path: &[usize],
    output: &mut Vec<TextRun>,
    last_was_space: &mut bool,
    capitalize_next: &mut bool,
) {
    for character in text.chars() {
        if character.is_whitespace() {
            if !*last_was_space {
                push_character(' ', style, element_path, output);
            }
            *last_was_space = true;
            *capitalize_next = true;
        } else {
            match style.text_transform {
                TextTransform::Uppercase => {
                    for transformed in character.to_uppercase() {
                        push_variant_character(transformed, style, element_path, output);
                    }
                }
                TextTransform::Lowercase => {
                    for transformed in character.to_lowercase() {
                        push_variant_character(transformed, style, element_path, output);
                    }
                }
                TextTransform::Capitalize if *capitalize_next => {
                    for transformed in character.to_uppercase() {
                        push_variant_character(transformed, style, element_path, output);
                    }
                }
                _ => push_variant_character(character, style, element_path, output),
            }
            *last_was_space = false;
            *capitalize_next = !character.is_alphanumeric();
        }
    }
}

fn push_variant_character(
    character: char,
    style: &TextStyle,
    element_path: &[usize],
    output: &mut Vec<TextRun>,
) {
    push_character(character, style, element_path, output);
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
    output
}

fn push_line_break(
    output: &mut Vec<TextRun>,
    style: &TextStyle,
    element_path: &[usize],
    last_was_space: &mut bool,
) {
    if let Some(last) = output.last_mut() {
        while last.text.ends_with(' ') {
            last.text.pop();
        }
        if !last.text.ends_with('\n') {
            push_character('\n', style, element_path, output);
        }
    }
    *last_was_space = true;
}

fn push_character(
    character: char,
    style: &TextStyle,
    element_path: &[usize],
    output: &mut Vec<TextRun>,
) {
    if let Some(last) = output.last_mut()
        && last.style == *style
        && last.element_path == element_path
    {
        last.text.push(character);
    } else {
        output.push(TextRun {
            text: character.to_string(),
            style: style.clone(),
            element_path: element_path.to_vec(),
        });
    }
}

fn trim_runs(runs: &mut Vec<TextRun>) {
    while runs
        .first()
        .is_some_and(|run| run.text.chars().all(char::is_whitespace))
    {
        runs.remove(0);
    }
    while runs
        .last()
        .is_some_and(|run| run.text.chars().all(char::is_whitespace))
    {
        runs.pop();
    }
    if let Some(last) = runs.last_mut() {
        let trimmed = last.text.trim_end().to_owned();
        last.text = trimmed;
    }
    runs.retain(|run| !run.text.is_empty());
}

fn is_block(tag: &str) -> bool {
    matches!(
        tag,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "div"
            | "footer"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "li"
            | "main"
            | "nav"
            | "ol"
            | "p"
            | "section"
            | "ul"
    )
}

#[cfg(test)]
#[path = "style_tests.rs"]
mod tests;
