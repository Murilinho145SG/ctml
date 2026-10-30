use super::background::{
    parse_background_color, parse_background_images, parse_background_position_layer,
    parse_background_repeat_layer, parse_background_size_layer, set_background_color,
    split_top_level_commas,
};
use crate::style::{
    BackgroundImage, BackgroundPosition, BackgroundRepeat, BackgroundSize, TextStyle,
};

pub(super) fn apply_background_shorthand(style: &mut TextStyle, value: &str) {
    let layers = split_top_level_commas(value);
    let Some(parsed) = layers
        .iter()
        .enumerate()
        .map(|(index, layer)| {
            parse_background_shorthand_layer(style, layer, index + 1 == layers.len())
        })
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };

    super::background_box::reset_background_image_layers(style);
    style.background = [255, 255, 255];
    style.background_color = None;
    style.background_images = parsed.iter().map(|layer| layer.image.clone()).collect();
    style.background_repeats = parsed.iter().map(|layer| layer.repeat).collect();
    style.background_positions = parsed.iter().map(|layer| layer.position).collect();
    style.background_sizes = parsed.iter().map(|layer| layer.size).collect();
    style.background_attachments = parsed.iter().map(|layer| layer.attachment).collect();
    style.background_origins = parsed.iter().map(|layer| layer.origin).collect();
    style.background_clips = parsed.iter().map(|layer| layer.clip).collect();
    if let Some((channels, alpha)) = parsed.last().and_then(|layer| layer.color) {
        set_background_color(style, channels, alpha);
    }
}

struct BackgroundShorthandLayer {
    image: BackgroundImage,
    repeat: BackgroundRepeat,
    position: BackgroundPosition,
    size: BackgroundSize,
    attachment: crate::style::BackgroundAttachment,
    origin: crate::style::BackgroundBox,
    clip: crate::style::BackgroundBox,
    color: Option<([u8; 3], f32)>,
}

fn parse_background_shorthand_layer(
    style: &TextStyle,
    value: &str,
    is_final_layer: bool,
) -> Option<BackgroundShorthandLayer> {
    use crate::style::{BackgroundAttachment, BackgroundBox};

    let mut tokens = split_top_level_whitespace(value);
    if tokens.is_empty() {
        return None;
    }
    let mut image = None;
    let mut repeat = BackgroundRepeat::REPEAT;
    let mut attachment = BackgroundAttachment::Scroll;
    let mut origin = BackgroundBox::PaddingBox;
    let mut clip = BackgroundBox::BorderBox;
    let mut boxes = Vec::new();
    let mut position_tokens = Vec::new();
    let mut position = None;
    let mut size = BackgroundSize::default();
    let mut color = None;
    let mut saw_slash = false;

    while let Some(token) = tokens.first().copied() {
        tokens.remove(0);
        if token == "/" {
            if saw_slash || position_tokens.is_empty() {
                return None;
            }
            saw_slash = true;
            position = Some(parse_background_position_layer(
                style,
                &position_tokens.join(" "),
            )?);
            position_tokens.clear();
            let size_tokens = take_background_size_tokens(style, &mut tokens);
            if size_tokens.is_empty() {
                return None;
            }
            size = parse_background_size_layer(style, &size_tokens.join(" "))?;
            continue;
        }
        let lower = token.to_ascii_lowercase();
        if lower == "none" || lower.starts_with("url(") || is_gradient_function(&lower) {
            if image.is_some() {
                return None;
            }
            image = Some(if lower == "none" {
                BackgroundImage::None
            } else {
                parse_background_images(style, token)?.into_iter().next()?
            });
        } else if let Some(value) = parse_background_repeat_layer(token) {
            if repeat != BackgroundRepeat::REPEAT {
                return None;
            }
            repeat = value;
        } else if let Some(value) = parse_background_attachment(token) {
            if attachment != BackgroundAttachment::Scroll {
                return None;
            }
            attachment = value;
        } else if let Some(value) = parse_background_box(token) {
            boxes.push(value);
            if boxes.len() > 2 {
                return None;
            }
        } else if let Some(value) = parse_background_color(style, token) {
            if !is_final_layer || color.is_some() {
                return None;
            }
            color = Some(value);
        } else {
            position_tokens.push(token);
            if position_tokens.len() > 2 {
                return None;
            }
        }
    }
    let position = if let Some(position) = position {
        position
    } else if position_tokens.is_empty() {
        BackgroundPosition::default()
    } else {
        parse_background_position_layer(style, &position_tokens.join(" "))?
    };
    if boxes.len() == 1 {
        origin = boxes[0];
        clip = boxes[0];
    } else if boxes.len() == 2 {
        origin = boxes[0];
        clip = boxes[1];
    }
    Some(BackgroundShorthandLayer {
        image: image.unwrap_or(BackgroundImage::None),
        repeat,
        position,
        size,
        attachment,
        origin,
        clip,
        color,
    })
}

fn take_background_size_tokens<'a>(style: &TextStyle, tokens: &mut Vec<&'a str>) -> Vec<&'a str> {
    let mut result = Vec::new();
    while tokens.first().is_some_and(|token| *token != "/") && result.len() < 2 {
        let candidate = [result.as_slice(), &tokens[..1]].concat();
        if parse_background_size_layer(style, &candidate.join(" ")).is_none() {
            break;
        }
        result.push(tokens.remove(0));
    }
    result
}

fn split_top_level_whitespace(value: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut depth = 0usize;
    let mut quote = None;
    let mut start = None;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            }
            continue;
        }
        if depth == 0 && character.is_ascii_whitespace() {
            if let Some(begin) = start.take() {
                tokens.push(&value[begin..index]);
            }
            continue;
        }
        if depth == 0 && character == '/' {
            if let Some(begin) = start.take() {
                tokens.push(&value[begin..index]);
            }
            tokens.push("/");
            continue;
        }
        start.get_or_insert(index);
        match character {
            '"' | '\'' => quote = Some(character),
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    if let Some(begin) = start {
        tokens.push(&value[begin..]);
    }
    tokens
}

fn is_gradient_function(value: &str) -> bool {
    value.starts_with("linear-gradient(")
        || value.starts_with("repeating-linear-gradient(")
        || value.starts_with("radial-gradient(")
        || value.starts_with("repeating-radial-gradient(")
        || value.starts_with("conic-gradient(")
        || value.starts_with("repeating-conic-gradient(")
}

fn parse_background_attachment(value: &str) -> Option<crate::style::BackgroundAttachment> {
    use crate::style::BackgroundAttachment;
    match value.to_ascii_lowercase().as_str() {
        "scroll" => Some(BackgroundAttachment::Scroll),
        "fixed" => Some(BackgroundAttachment::Fixed),
        "local" => Some(BackgroundAttachment::Local),
        _ => None,
    }
}

fn parse_background_box(value: &str) -> Option<crate::style::BackgroundBox> {
    use crate::style::BackgroundBox;
    match value.to_ascii_lowercase().as_str() {
        "border-box" => Some(BackgroundBox::BorderBox),
        "padding-box" => Some(BackgroundBox::PaddingBox),
        "content-box" => Some(BackgroundBox::ContentBox),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::{BackgroundAttachment, BackgroundBox};

    #[test]
    fn parses_background_shorthand_components_and_layer_defaults() {
        let style = TextStyle::default();
        let layer = parse_background_shorthand_layer(
            &style,
            "linear-gradient(red, blue) no-repeat right bottom / 10px 20px fixed content-box padding-box blue",
            true,
        )
        .expect("supported background shorthand should parse");

        assert!(matches!(layer.image, BackgroundImage::LinearGradient(_)));
        assert_eq!(layer.repeat, BackgroundRepeat::NO_REPEAT);
        assert_eq!(layer.position.x_percent, 1.0);
        assert_eq!(layer.position.y_percent, 1.0);
        assert_eq!(layer.size.width.unwrap().value, 10.0);
        assert_eq!(layer.size.height.unwrap().value, 20.0);
        assert_eq!(layer.attachment, BackgroundAttachment::Fixed);
        assert_eq!(layer.origin, BackgroundBox::ContentBox);
        assert_eq!(layer.clip, BackgroundBox::PaddingBox);
        assert_eq!(layer.color.unwrap().0, [0, 0, 255]);
    }
}
