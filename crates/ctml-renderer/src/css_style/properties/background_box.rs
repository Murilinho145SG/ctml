use crate::style::{BackgroundBox, TextStyle};

pub(super) fn apply_background_origin(style: &mut TextStyle, value: &str) {
    if let Some(boxes) = parse_background_boxes(value) {
        style.background_origins = boxes;
    }
}

pub(super) fn apply_background_clip(style: &mut TextStyle, value: &str) {
    if let Some(boxes) = parse_background_boxes(value) {
        style.background_clips = boxes;
    }
}

pub(super) fn reset_background_image_layers(style: &mut TextStyle) {
    style.background_images.clear();
    style.background_repeats.clear();
    style.background_positions.clear();
    style.background_sizes.clear();
    style.background_attachments.clear();
    reset_background_box_properties(style);
}

pub(super) fn reset_background_box_properties(style: &mut TextStyle) {
    style.background_origins.clear();
    style.background_clips.clear();
}

fn parse_background_boxes(value: &str) -> Option<Vec<BackgroundBox>> {
    value
        .split(',')
        .map(|layer| match layer.trim().to_ascii_lowercase().as_str() {
            "border-box" => Some(BackgroundBox::BorderBox),
            "padding-box" => Some(BackgroundBox::PaddingBox),
            "content-box" => Some(BackgroundBox::ContentBox),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()
        .filter(|boxes| !boxes.is_empty())
}
