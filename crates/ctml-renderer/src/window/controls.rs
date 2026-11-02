use super::{NativeWindow, WindowApplication};
use crate::UiNode;
use unicode_segmentation::UnicodeSegmentation;

impl WindowApplication {
    pub(super) fn dispatch_action(&mut self, window: &NativeWindow, action: String) {
        let Some(handler) = self.config.on_action.clone() else {
            return;
        };
        handler(&action, &mut self.config.body);
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.replace_body(&self.config.body);
            window.request_redraw();
        }
    }

    pub(super) fn dispatch_input(
        &mut self,
        window: &NativeWindow,
        path: &[usize],
        identifier: &str,
        value: String,
    ) {
        if !set_input_value(&mut self.config.body, path, &value) {
            return;
        }
        if let Some(handler) = self.config.on_input.clone() {
            handler(identifier, &value, &mut self.config.body);
        }
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.replace_body(&self.config.body);
            window.request_redraw();
        }
    }
}

fn node_at_path_mut<'a>(nodes: &'a mut [UiNode], path: &[usize]) -> Option<&'a mut UiNode> {
    let (index, remaining) = path.split_first()?;
    let node = nodes.get_mut(*index)?;
    if remaining.is_empty() {
        return Some(node);
    }
    let UiNode::Element { children, .. } = node else {
        return None;
    };
    node_at_path_mut(children, remaining)
}

fn set_input_value(nodes: &mut [UiNode], path: &[usize], value: &str) -> bool {
    let Some(UiNode::Element { attributes, .. }) = node_at_path_mut(nodes, path) else {
        return false;
    };
    if let Some((_, current)) = attributes
        .iter_mut()
        .find(|(name, _)| name.eq_ignore_ascii_case("value"))
    {
        *current = Some(value.to_owned());
    } else {
        attributes.push(("value".to_owned(), Some(value.to_owned())));
    }
    true
}

pub(super) fn edit_focused_input(
    input: &(Vec<usize>, String, String),
    key_text: Option<&str>,
    backspace: bool,
) -> Option<String> {
    let mut value = input.2.clone();
    if backspace {
        let end = value
            .grapheme_indices(true)
            .next_back()
            .map_or(value.len(), |(index, _)| index);
        if end == value.len() {
            return None;
        }
        value.truncate(end);
        return Some(value);
    }
    let text = key_text?;
    if text.is_empty() || text.chars().any(char::is_control) {
        return None;
    }
    value.push_str(text);
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::{edit_focused_input, set_input_value};
    use crate::UiNode;

    #[test]
    fn appends_text_and_deletes_a_complete_grapheme() {
        let input = (vec![0], "query".to_owned(), "café".to_owned());
        assert_eq!(
            edit_focused_input(&input, Some("!"), false),
            Some("café!".to_owned())
        );
        assert_eq!(
            edit_focused_input(&input, None, true),
            Some("caf".to_owned())
        );
        assert_eq!(
            edit_focused_input(&(vec![0], String::new(), "e\u{301}".to_owned()), None, true),
            Some(String::new())
        );
    }

    #[test]
    fn updates_nested_input_values_without_changing_sibling_nodes() {
        let mut body = vec![
            UiNode::Text("before".to_owned()),
            UiNode::Element {
                tag: "section".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Element {
                    tag: "input".to_owned(),
                    class: None,
                    id: Some("query".to_owned()),
                    attributes: Vec::new(),
                    children: Vec::new(),
                }],
            },
        ];

        assert!(set_input_value(&mut body, &[1, 0], "updated"));
        let UiNode::Element { children, .. } = &body[1] else {
            panic!("expected section element");
        };
        let UiNode::Element { attributes, .. } = &children[0] else {
            panic!("expected input element");
        };
        assert_eq!(
            attributes,
            &[("value".to_owned(), Some("updated".to_owned()))]
        );
        assert_eq!(body[0], UiNode::Text("before".to_owned()));
    }
}
