use crate::style::{BackgroundAttachment, TextStyle};

pub(super) fn apply_background_attachment(style: &mut TextStyle, value: &str) {
    let attachments = value
        .split(',')
        .map(|layer| match layer.trim().to_ascii_lowercase().as_str() {
            "scroll" => Some(BackgroundAttachment::Scroll),
            "fixed" => Some(BackgroundAttachment::Fixed),
            "local" => Some(BackgroundAttachment::Local),
            _ => None,
        })
        .collect::<Option<Vec<_>>>();
    if let Some(attachments) = attachments.filter(|layers| !layers.is_empty()) {
        style.background_attachments = attachments;
    }
}
