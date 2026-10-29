use crate::UiNode;

#[derive(Default)]
pub(super) struct FocusManager {
    focusable_ids: Vec<usize>,
    tab_order: Vec<Vec<usize>>,
}

impl FocusManager {
    pub(super) fn new(nodes: &[UiNode]) -> Self {
        let mut focusable = Vec::new();
        let mut next_node_id = 1;
        collect_focusable(nodes, &[], &mut next_node_id, &mut focusable);
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
        }
    }

    pub(super) fn contains(&self, node_id: &usize) -> bool {
        self.focusable_ids.contains(node_id)
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
    next_node_id: &mut usize,
    output: &mut Vec<FocusableNode>,
) {
    let mut siblings = Vec::new();
    for node in nodes {
        let UiNode::Element {
            tag,
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
        siblings.push((node_id, path, tag, attributes, children));
    }

    for (node_id, path, tag, attributes, children) in &siblings {
        if let Some(tab_index) = tab_index(tag, attributes) {
            output.push(FocusableNode {
                node_id: *node_id,
                path: path.clone(),
                tab_index,
                order: output.len(),
            });
        }
        collect_focusable(children, path, next_node_id, output);
    }
}

fn tab_index(tag: &str, attributes: &[(String, Option<String>)]) -> Option<i32> {
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
    })
}

#[cfg(test)]
mod tests {
    use super::FocusManager;
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
}
