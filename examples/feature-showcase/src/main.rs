use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

mod generated {
    include!(concat!(env!("OUT_DIR"), "/window_config.rs"));
}

fn main() -> Result<(), ctml::WindowError> {
    let counter = Arc::new(AtomicUsize::new(0));
    let action_counter = Arc::clone(&counter);

    ctml::Window::create(ctml::WindowConfig {
        title: generated::WINDOW_TITLE.to_owned(),
        width: generated::WINDOW_WIDTH,
        height: generated::WINDOW_HEIGHT,
        body: generated::body(),
        stylesheet: generated::stylesheet(),
        keyframes: generated::keyframes(),
        fonts: generated::fonts(),
        color_profiles: generated::color_profiles(),
        on_action: Some(Arc::new(move |action, body| match action {
            "increment" => {
                let value = action_counter.fetch_add(1, Ordering::Relaxed) + 1;
                set_text(body, "counter-value", value.to_string());
                set_text(
                    body,
                    "action-status",
                    "Button action received: increment".to_owned(),
                );
            }
            "reset" => {
                action_counter.store(0, Ordering::Relaxed);
                set_text(body, "counter-value", "0".to_owned());
                set_text(body, "action-status", "Demo state reset.".to_owned());
                set_text(
                    body,
                    "input-feedback",
                    "Your input value will appear here.".to_owned(),
                );
                set_attribute(body, "search-input", "value", String::new());
            }
            "explore" => {
                set_text(
                    body,
                    "action-status",
                    "Explore action received. Keep scrolling through the feature tour.".to_owned(),
                );
            }
            _ => {}
        })),
        on_input: Some(Arc::new(|identifier, value, body| {
            if identifier == "search-input" {
                let feedback = if value.is_empty() {
                    "Your input value will appear here.".to_owned()
                } else {
                    format!("Searching for: {value}")
                };
                set_text(body, "input-feedback", feedback);
            }
        })),
        ..ctml::WindowConfig::default()
    })
}

fn set_text(nodes: &mut [ctml::UiNode], target_id: &str, text: String) -> bool {
    for node in nodes {
        let ctml::UiNode::Element { id, children, .. } = node else {
            continue;
        };
        if id.as_deref() == Some(target_id) {
            if let Some(ctml::UiNode::Text(current)) = children.first_mut() {
                *current = text;
            } else {
                children.clear();
                children.push(ctml::UiNode::Text(text));
            }
            return true;
        }
        if set_text(children, target_id, text.clone()) {
            return true;
        }
    }
    false
}

fn set_attribute(nodes: &mut [ctml::UiNode], target_id: &str, name: &str, value: String) -> bool {
    for node in nodes {
        let ctml::UiNode::Element {
            id,
            attributes,
            children,
            ..
        } = node
        else {
            continue;
        };
        if id.as_deref() == Some(target_id) {
            if let Some((_, current)) = attributes
                .iter_mut()
                .find(|(attribute, _)| attribute.eq_ignore_ascii_case(name))
            {
                *current = Some(value);
            } else {
                attributes.push((name.to_owned(), Some(value)));
            }
            return true;
        }
        if set_attribute(children, target_id, name, value.clone()) {
            return true;
        }
    }
    false
}
