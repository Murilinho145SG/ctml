use std::collections::HashMap;

use crate::UiNode;

#[derive(Default)]
pub(super) struct FocusManager {
    focusable_ids: Vec<usize>,
    tab_order: Vec<Vec<usize>>,
    button_actions: HashMap<usize, String>,
    text_inputs: HashMap<usize, TextInput>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TextInput {
    pub(super) path: Vec<usize>,
    pub(super) id: String,
    pub(super) value: String,
}

impl FocusManager {
    pub(super) fn new(nodes: &[UiNode]) -> Self {
        let mut focusable = Vec::new();
        let mut button_actions = HashMap::new();
        let mut text_inputs = HashMap::new();
        let mut next_node_id = 1;
        collect_focusable(
            nodes,
            &[],
            &[],
            &mut next_node_id,
            &mut focusable,
            &mut button_actions,
            &mut text_inputs,
        );
        let focusable_ids = focusable.iter().map(|item| item.node_id).collect();
        focusable.sort_by_key(|item| {
            if item.tab_index > 0 {
                (0, item.tab_index, item.order)
            } else {
                (1, 0, item.order)
            }
        });
        let tab_order = focusable
            .into_iter()
            .filter(|item| item.tab_index >= 0)
            .map(|item| item.path)
            .collect();
        Self {
            focusable_ids,
            tab_order,
            button_actions,
            text_inputs,
        }
    }

    pub(super) fn contains(&self, node_id: &usize) -> bool {
        self.focusable_ids.contains(node_id)
    }

    pub(super) fn action_for_path(&self, path: &[usize]) -> Option<&str> {
        path.iter()
            .rev()
            .find_map(|node_id| self.button_actions.get(node_id).map(String::as_str))
    }

    pub(super) fn action_for_focus(&self, focused_path: &[usize]) -> Option<&str> {
        self.action_for_path(focused_path)
    }

    pub(super) fn text_input_for_focus(&self, focused_path: &[usize]) -> Option<&TextInput> {
        focused_path
            .last()
            .and_then(|node_id| self.text_inputs.get(node_id))
    }

    pub(super) fn next(&self, focused_path: &[usize], backwards: bool) -> Option<Vec<usize>> {
        if self.tab_order.is_empty() {
            return None;
        }
        let current = focused_path.last().and_then(|node_id| {
            self.tab_order
                .iter()
                .position(|path| path.last() == Some(node_id))
        });
        let next = match (current, backwards) {
            (Some(index), true) => (index + self.tab_order.len() - 1) % self.tab_order.len(),
            (Some(index), false) => (index + 1) % self.tab_order.len(),
            (None, true) => self.tab_order.len() - 1,
            (None, false) => 0,
        };
        Some(self.tab_order[next].clone())
    }
}

struct FocusableNode {
    node_id: usize,
    path: Vec<usize>,
    tab_index: i32,
    order: usize,
}

fn collect_focusable(
    nodes: &[UiNode],
    parent_path: &[usize],
    parent_indices: &[usize],
    next_node_id: &mut usize,
    output: &mut Vec<FocusableNode>,
    button_actions: &mut HashMap<usize, String>,
    text_inputs: &mut HashMap<usize, TextInput>,
) {
    let mut siblings = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        let UiNode::Element {
            tag,
            id,
            attributes,
            children,
            ..
        } = node
        else {
            continue;
        };
        let node_id = *next_node_id;
        *next_node_id += 1;
        let mut path = parent_path.to_vec();
        path.push(node_id);
        let mut index_path = parent_indices.to_vec();
        index_path.push(index);
        siblings.push((node_id, path, index_path, tag, id, attributes, children));
    }

    for (node_id, path, index_path, tag, id, attributes, children) in &siblings {
        if let Some(tab_index) = tab_index(tag, attributes) {
            output.push(FocusableNode {
                node_id: *node_id,
                path: path.clone(),
                tab_index,
                order: output.len(),
            });
        }
        if tag.eq_ignore_ascii_case("button")
            && !has_attribute(attributes, "disabled")
            && let Some(action) = attributes
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("action"))
                .and_then(|(_, value)| value.as_deref())
                .map(str::trim)
                .filter(|action| !action.is_empty())
        {
            button_actions.insert(*node_id, action.to_owned());
        }
        if is_text_input(tag, attributes) && !has_attribute(attributes, "disabled") {
            let id = id
                .as_deref()
                .or_else(|| attribute(attributes, "id"))
                .or_else(|| attribute(attributes, "name"))
                .unwrap_or_default()
                .to_owned();
            let value = attribute(attributes, "value")
                .unwrap_or_default()
                .to_owned();
            text_inputs.insert(
                *node_id,
                TextInput {
                    path: index_path.clone(),
                    id,
                    value,
                },
            );
        }
        collect_focusable(
            children,
            path,
            index_path,
            next_node_id,
            output,
            button_actions,
            text_inputs,
        );
    }
}

fn tab_index(tag: &str, attributes: &[(String, Option<String>)]) -> Option<i32> {
    let native_control = tag.eq_ignore_ascii_case("button") || tag.eq_ignore_ascii_case("input");
    if tag.eq_ignore_ascii_case("input") && !is_text_input(tag, attributes) {
        return None;
    }
    if native_control && has_attribute(attributes, "disabled") {
        return None;
    }
    let explicit = attributes
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("tabindex"))
        .and_then(|(_, value)| value.as_deref())
        .and_then(|value| value.trim().parse::<i32>().ok());
    explicit.or_else(|| {
        (tag.eq_ignore_ascii_case("a")
            && attributes
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case("href")))
        .then_some(0)
        .or_else(|| {
            (tag.eq_ignore_ascii_case("button") || is_text_input(tag, attributes)).then_some(0)
        })
    })
}

fn is_text_input(tag: &str, attributes: &[(String, Option<String>)]) -> bool {
    tag.eq_ignore_ascii_case("input")
        && attribute(attributes, "type").is_none_or(|kind| {
            ["text", "search", "email", "url", "tel"]
                .iter()
                .any(|supported| kind.eq_ignore_ascii_case(supported))
        })
}

fn attribute<'a>(attributes: &'a [(String, Option<String>)], name: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|(attribute, _)| attribute.eq_ignore_ascii_case(name))
        .and_then(|(_, value)| value.as_deref())
}

fn has_attribute(attributes: &[(String, Option<String>)], name: &str) -> bool {
    attributes
        .iter()
        .any(|(attribute, _)| attribute.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::{FocusManager, TextInput};
    use crate::UiNode;

    fn element(tag: &str, attributes: Vec<(&str, Option<&str>)>, children: Vec<UiNode>) -> UiNode {
        UiNode::Element {
            tag: tag.to_owned(),
            class: None,
            id: None,
            attributes: attributes
                .into_iter()
                .map(|(name, value)| (name.to_owned(), value.map(str::to_owned)))
                .collect(),
            children,
        }
    }

    #[test]
    fn advances_in_document_order_and_wraps_in_both_directions() {
        let nodes = [
            element("a", vec![("href", Some("/first"))], vec![]),
            element("a", vec![("href", Some("/second"))], vec![]),
        ];
        let focus = FocusManager::new(&nodes);

        assert_eq!(focus.next(&[], false), Some(vec![1]));
        assert_eq!(focus.next(&[1], false), Some(vec![2]));
        assert_eq!(focus.next(&[2], false), Some(vec![1]));
        assert_eq!(focus.next(&[], true), Some(vec![2]));
    }

    #[test]
    fn honors_positive_tab_order_and_skips_negative_tab_index() {
        let nodes = [
            element("a", vec![("href", Some("/zero"))], vec![]),
            element("span", vec![("tabindex", Some("-1"))], vec![]),
            element("span", vec![("tabindex", Some("2"))], vec![]),
            element("span", vec![("tabindex", Some("1"))], vec![]),
        ];
        let focus = FocusManager::new(&nodes);

        assert_eq!(focus.next(&[], false), Some(vec![4]));
        assert_eq!(focus.next(&[4], false), Some(vec![3]));
        assert_eq!(focus.next(&[3], false), Some(vec![1]));
        assert!(focus.contains(&2));
        assert_eq!(focus.next(&[2], false), Some(vec![4]));
    }

    #[test]
    fn visits_nested_elements_in_document_order() {
        let nodes = [
            element(
                "section",
                vec![],
                vec![element("a", vec![("href", Some("/nested"))], vec![])],
            ),
            element("a", vec![("href", Some("/sibling"))], vec![]),
        ];
        let focus = FocusManager::new(&nodes);

        assert_eq!(focus.next(&[], false), Some(vec![1, 3]));
        assert_eq!(focus.next(&[1, 3], false), Some(vec![2]));
    }

    #[test]
    fn buttons_join_tab_order_and_expose_named_actions() {
        let nodes = [
            element("button", vec![("action", Some("save"))], vec![]),
            element(
                "button",
                vec![("disabled", None), ("action", Some("skip"))],
                vec![],
            ),
        ];
        let focus = FocusManager::new(&nodes);

        assert_eq!(focus.next(&[], false), Some(vec![1]));
        assert_eq!(focus.next(&[1], false), Some(vec![1]));
        assert_eq!(focus.action_for_path(&[1]), Some("save"));
        assert_eq!(focus.action_for_path(&[2]), None);
    }

    #[test]
    fn text_inputs_join_tab_order_and_keep_their_current_value() {
        let nodes = [element(
            "input",
            vec![
                ("id", Some("query")),
                ("type", Some("text")),
                ("value", Some("ctml")),
            ],
            vec![],
        )];
        let focus = FocusManager::new(&nodes);

        assert_eq!(focus.next(&[], false), Some(vec![1]));
        assert_eq!(
            focus.text_input_for_focus(&[1]),
            Some(&TextInput {
                path: vec![0],
                id: "query".to_owned(),
                value: "ctml".to_owned(),
            })
        );
    }
}
