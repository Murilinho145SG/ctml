use crate::{StyleDeclaration, StyleRule, UiNode};

pub(super) fn with_control_defaults(nodes: &[UiNode], stylesheet: &[StyleRule]) -> Vec<StyleRule> {
    let has_buttons = contains_tag(nodes, "button");
    let has_text_inputs = contains_text_input(nodes);
    if !has_buttons && !has_text_inputs {
        return stylesheet.to_vec();
    }

    let mut rules = Vec::new();
    if has_buttons {
        rules.extend([
            rule(
                "button",
                &[
                    ("display", "inline-block"),
                    ("padding", "6px 12px"),
                    ("border", "1px solid #9ca3af"),
                    ("border-radius", "4px"),
                    ("background-color", "#f3f4f6"),
                    ("color", "#1f2937"),
                    ("cursor", "pointer"),
                    ("user-select", "none"),
                ],
            ),
            rule("button:hover", &[("background-color", "#e5e7eb")]),
            rule("button:active", &[("background-color", "#d1d5db")]),
            rule(
                "button:focus-visible",
                &[("outline", "2px solid #2563eb"), ("outline-offset", "2px")],
            ),
        ]);
    }
    if has_text_inputs {
        rules.extend([
            rule(
                "input",
                &[
                    ("display", "inline-block"),
                    ("width", "180px"),
                    ("padding", "4px 6px"),
                    ("border", "1px solid #9ca3af"),
                    ("border-radius", "2px"),
                    ("background-color", "#ffffff"),
                    ("color", "#1f2937"),
                    ("user-select", "text"),
                ],
            ),
            rule(
                "input:focus-visible",
                &[("outline", "2px solid #2563eb"), ("outline-offset", "2px")],
            ),
        ]);
    }
    rules.extend_from_slice(stylesheet);
    rules
}

pub(super) fn with_control_content(nodes: &[UiNode]) -> Vec<UiNode> {
    nodes
        .iter()
        .map(|node| match node {
            UiNode::Text(text) => UiNode::Text(text.clone()),
            UiNode::Element {
                tag,
                class,
                id,
                attributes,
                children,
            } => {
                let children = if is_text_input(tag, attributes) {
                    let text = attribute(attributes, "value")
                        .filter(|value| !value.is_empty())
                        .or_else(|| attribute(attributes, "placeholder"));
                    text.map_or_else(Vec::new, |text| vec![UiNode::Text(text.to_owned())])
                } else {
                    with_control_content(children)
                };
                UiNode::Element {
                    tag: tag.clone(),
                    class: class.clone(),
                    id: id.clone(),
                    attributes: attributes.clone(),
                    children,
                }
            }
        })
        .collect()
}

fn contains_tag(nodes: &[UiNode], expected: &str) -> bool {
    nodes.iter().any(|node| match node {
        UiNode::Text(_) => false,
        UiNode::Element { tag, children, .. } => {
            tag.eq_ignore_ascii_case(expected) || contains_tag(children, expected)
        }
    })
}

fn contains_text_input(nodes: &[UiNode]) -> bool {
    nodes.iter().any(|node| match node {
        UiNode::Text(_) => false,
        UiNode::Element {
            tag,
            attributes,
            children,
            ..
        } => is_text_input(tag, attributes) || contains_text_input(children),
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

fn rule(selector: &str, declarations: &[(&str, &str)]) -> StyleRule {
    StyleRule {
        selectors: vec![selector.to_owned()],
        declarations: declarations
            .iter()
            .map(|(property, value)| StyleDeclaration {
                property: (*property).to_owned(),
                value: (*value).to_owned(),
            })
            .collect(),
        layer: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{with_control_content, with_control_defaults};
    use crate::{StyleDeclaration, StyleRule, UiNode};

    fn authored_rule() -> StyleRule {
        StyleRule {
            selectors: vec!["button".to_owned()],
            declarations: vec![StyleDeclaration {
                property: "color".to_owned(),
                value: "red".to_owned(),
            }],
            layer: None,
        }
    }

    #[test]
    fn adds_defaults_only_when_the_document_contains_a_button() {
        let authored = authored_rule();
        assert_eq!(
            with_control_defaults(&[], std::slice::from_ref(&authored)),
            [authored.clone()]
        );

        let nodes = [UiNode::Element {
            tag: "button".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: Vec::new(),
        }];
        let rules = with_control_defaults(&nodes, std::slice::from_ref(&authored));

        assert_eq!(rules.len(), 5);
        assert_eq!(rules[0].selectors, ["button"]);
        assert_eq!(rules.last(), Some(&authored));
    }

    #[test]
    fn inputs_render_their_value_or_placeholder_as_text() {
        let nodes = [UiNode::Element {
            tag: "input".to_owned(),
            class: None,
            id: None,
            attributes: vec![("placeholder".to_owned(), Some("Search".to_owned()))],
            children: Vec::new(),
        }];

        assert_eq!(
            with_control_content(&nodes),
            [UiNode::Element {
                tag: "input".to_owned(),
                class: None,
                id: None,
                attributes: vec![("placeholder".to_owned(), Some("Search".to_owned()))],
                children: vec![UiNode::Text("Search".to_owned())],
            }]
        );
    }

    #[test]
    fn input_defaults_do_not_enable_button_hover_rules() {
        let nodes = [UiNode::Element {
            tag: "input".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: Vec::new(),
        }];

        let rules = with_control_defaults(&nodes, &[]);
        assert_eq!(rules.len(), 2);
        assert!(rules.iter().all(|rule| {
            rule.selectors
                .iter()
                .all(|selector| selector.starts_with("input"))
        }));
    }
}
