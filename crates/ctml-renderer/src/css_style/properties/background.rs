use crate::style::{BackgroundImage, LinearGradient, TextStyle};
use base64::{Engine, engine::general_purpose::STANDARD};
use cssparser::{Parser, ParserInput, Token};
use ctml_css::parse_color_with_opacity_and_profiles;

pub(super) fn apply_background_shorthand(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("none") {
        style.background_images.clear();
        style.background = [255, 255, 255];
        style.background_color = None;
    } else if let Some(images) = parse_background_images(style, value) {
        style.background_images = images;
        style.background = [255, 255, 255];
        style.background_color = None;
    } else if let Some((channels, alpha)) = parse_background_color(style, value) {
        style.background_images.clear();
        set_background_color(style, channels, alpha);
    } else {
        return;
    }
}

pub(super) fn apply_background_image(style: &mut TextStyle, value: &str) {
    let Some(images) = parse_background_images(style, value) else {
        return;
    };
    style.background_images = images;
}

fn parse_background_images(style: &TextStyle, value: &str) -> Option<Vec<BackgroundImage>> {
    if value.eq_ignore_ascii_case("none") {
        return Some(Vec::new());
    }
    let layers = split_top_level_commas(value);
    let mut images = Vec::with_capacity(layers.len());
    for layer in layers {
        if layer
            .trim_start()
            .get(..15)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("linear-gradient"))
        {
            images.push(BackgroundImage::LinearGradient(parse_linear_gradient(
                style, layer,
            )?));
        } else {
            images.push(BackgroundImage::Encoded(parse_data_url(layer)?));
        }
    }
    (!images.is_empty()).then_some(images)
}

fn parse_data_url(value: &str) -> Option<std::sync::Arc<[u8]>> {
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
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
    parser.expect_exhausted().ok()?;
    decode_data_image(&url).map(Into::into)
}

fn parse_linear_gradient(style: &TextStyle, value: &str) -> Option<LinearGradient> {
    let value = value.trim();
    let opening = value.find('(')?;
    if !value[..opening].eq_ignore_ascii_case("linear-gradient") || !value.ends_with(')') {
        return None;
    }
    let arguments = &value[opening + 1..value.len() - 1];
    let mut components = split_top_level_commas(arguments).into_iter();
    let mut stops = Vec::new();
    let mut angle = 180.0;
    let first = components.next()?;
    if let Some(parsed_angle) = parse_gradient_angle(first.trim()) {
        angle = parsed_angle;
    } else {
        stops.push(parse_gradient_stop(style, first)?);
    }
    stops.extend(
        components
            .map(|component| parse_gradient_stop(style, component))
            .collect::<Option<Vec<_>>>()?,
    );
    if stops.len() < 2 {
        return None;
    }
    resolve_gradient_stop_positions(&mut stops)?;
    let stops = stops
        .into_iter()
        .map(|(color, position)| Some((color, position?)))
        .collect::<Option<Vec<_>>>()?;
    Some(LinearGradient {
        angle_degrees: angle.rem_euclid(360.0),
        stops,
    })
}

fn parse_gradient_angle(value: &str) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    if let Some(direction) = value.strip_prefix("to ") {
        return match direction.trim() {
            "top" => Some(0.0),
            "top right" | "right top" => Some(45.0),
            "right" => Some(90.0),
            "bottom right" | "right bottom" => Some(135.0),
            "bottom" => Some(180.0),
            "bottom left" | "left bottom" => Some(225.0),
            "left" => Some(270.0),
            "top left" | "left top" => Some(315.0),
            _ => None,
        };
    }
    let (number, unit) = ["deg", "turn", "grad", "rad"]
        .into_iter()
        .find_map(|unit| value.strip_suffix(unit).map(|number| (number, unit)))?;
    let number = number.trim().parse::<f32>().ok()?;
    if !number.is_finite() {
        return None;
    }
    let degrees = match unit {
        "deg" => number,
        "turn" => number * 360.0,
        "grad" => number * 0.9,
        _ => number.to_degrees(),
    };
    degrees.is_finite().then_some(degrees)
}

fn parse_gradient_stop(style: &TextStyle, value: &str) -> Option<([u8; 4], Option<f32>)> {
    let value = value.trim();
    let split = last_top_level_whitespace(value);
    let (color_text, position) = if let Some(split) = split {
        let candidate = value[split..].trim();
        if let Some(position) = ctml_css::parse_css_percentage(candidate) {
            let color_text = value[..split].trim();
            if color_text.is_empty() {
                return None;
            }
            (color_text, Some(position / 100.0))
        } else {
            (value, None)
        }
    } else {
        (value, None)
    };
    let (rgb, alpha) = if color_text.eq_ignore_ascii_case("currentcolor") {
        (
            [style.color[0], style.color[1], style.color[2]],
            style.color_opacity,
        )
    } else {
        parse_color_with_opacity_and_profiles(color_text, &style.color_profiles)?
    };
    Some((
        [rgb[0], rgb[1], rgb[2], (alpha * 255.0).round() as u8],
        position,
    ))
}

fn resolve_gradient_stop_positions(stops: &mut [([u8; 4], Option<f32>)]) -> Option<()> {
    stops.first_mut()?.1.get_or_insert(0.0);
    stops.last_mut()?.1.get_or_insert(1.0);
    let mut previous = 0;
    for index in 1..stops.len() {
        if stops[index].1.is_none() {
            let start = stops[previous].1?;
            let end_index = (index + 1..stops.len()).find(|next| stops[*next].1.is_some())?;
            let end = stops[end_index].1?;
            let span = (end - start) / (end_index - previous) as f32;
            for (offset, stop) in stops
                .iter_mut()
                .enumerate()
                .take(end_index)
                .skip(previous + 1)
            {
                stop.1 = Some(start + span * (offset - previous) as f32);
            }
        }
        previous = index;
    }
    let mut previous_position = 0.0_f32;
    for (_, position) in stops.iter_mut() {
        let Some(current) = position.as_mut() else {
            return None;
        };
        if !current.is_finite() {
            return None;
        }
        *current = current.max(previous_position);
        previous_position = *current;
    }
    Some(())
}

fn split_top_level_commas(value: &str) -> Vec<&str> {
    let mut components = Vec::new();
    let mut depth = 0usize;
    let mut quote = None;
    let mut start = 0;
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
        match character {
            '"' | '\'' => quote = Some(character),
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                components.push(value[start..index].trim());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    components.push(value[start..].trim());
    components
}

fn last_top_level_whitespace(value: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut last = None;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            character if character.is_whitespace() && depth == 0 => last = Some(index),
            _ => {}
        }
    }
    last
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
    if let Some((channels, alpha)) = parse_background_color(style, value) {
        set_background_color(style, channels, alpha);
    }
}

fn parse_background_color(style: &TextStyle, value: &str) -> Option<([u8; 3], f32)> {
    if value.eq_ignore_ascii_case("currentcolor") {
        Some((
            [style.color[0], style.color[1], style.color[2]],
            style.color_opacity,
        ))
    } else {
        parse_color_with_opacity_and_profiles(value, &style.color_profiles)
    }
}

fn set_background_color(style: &mut TextStyle, channels: [u8; 3], alpha: f32) {
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
