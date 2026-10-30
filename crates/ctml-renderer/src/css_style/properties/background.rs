use crate::style::TextStyle;
use base64::{Engine, engine::general_purpose::STANDARD};
use cssparser::{Parser, ParserInput, Token};
use ctml_css::parse_color_with_opacity_and_profiles;
use std::sync::Arc;

pub(super) fn apply_background_shorthand(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("none") {
        style.background_images.clear();
        style.background = [255, 255, 255];
        style.background_color = None;
    } else if let Some(images) = parse_background_images(value) {
        style.background_images = images;
        style.background = [255, 255, 255];
        style.background_color = None;
    } else {
        let previous_images = style.background_images.clone();
        apply_background_color(style, value);
        if style.background_color.is_some() {
            style.background_images.clear();
        } else {
            style.background_images = previous_images;
        }
    }
}

pub(super) fn apply_background_image(style: &mut TextStyle, value: &str) {
    let Some(images) = parse_background_images(value) else {
        return;
    };
    style.background_images = images;
}

fn parse_background_images(value: &str) -> Option<Vec<Arc<[u8]>>> {
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    if parser
        .try_parse(|parser| parser.expect_ident_matching("none"))
        .is_ok()
    {
        return parser.is_exhausted().then(Vec::new);
    }
    let mut images = Vec::new();
    while !parser.is_exhausted() {
        let url = match parser.next().ok()?.clone() {
            Token::UnquotedUrl(url) => url.to_string(),
            Token::Function(name) if name.eq_ignore_ascii_case("url") => parser
                .parse_nested_block(|nested| {
                    let url = nested.expect_string()?.to_string();
                    nested.expect_exhausted()?;
                    Ok::<_, cssparser::ParseError<'_, ()>>(url)
                })
                .ok()?,
            _ => return None,
        };
        images.push(decode_data_image(&url)?.into());
        if !parser.is_exhausted() {
            parser.expect_comma().ok()?;
            if parser.is_exhausted() {
                return None;
            }
        }
    }
    (!images.is_empty()).then_some(images)
}

fn decode_data_image(url: &str) -> Option<Vec<u8>> {
    let (metadata, data) = url.strip_prefix("data:")?.split_once(',')?;
    let (media_type, encoding) = metadata.split_once(';')?;
    if !matches!(
        media_type.to_ascii_lowercase().as_str(),
        "image/png" | "image/jpeg"
    ) || !encoding.eq_ignore_ascii_case("base64")
        || data.len() > 24 * 1024 * 1024
    {
        return None;
    }
    let decoded = STANDARD.decode(data).ok()?;
    (decoded.len() <= 16 * 1024 * 1024).then_some(decoded)
}

pub(super) fn apply_background_color(style: &mut TextStyle, value: &str) {
    let color = if value.eq_ignore_ascii_case("currentcolor") {
        Some((
            [style.color[0], style.color[1], style.color[2]],
            style.color_opacity,
        ))
    } else {
        parse_color_with_opacity_and_profiles(value, &style.color_profiles)
    };
    let Some((channels, alpha)) = color else {
        return;
    };
    style.background_color = Some([
        channels[0],
        channels[1],
        channels[2],
        (alpha * 255.0).round() as u8,
    ]);
    style.background = [
        composite_over_white(channels[0], alpha),
        composite_over_white(channels[1], alpha),
        composite_over_white(channels[2], alpha),
    ];
}

fn composite_over_white(channel: u8, alpha: f32) -> u8 {
    (channel as f32 * alpha + 255.0 * (1.0 - alpha)).round() as u8
}
