use crate::style::{
    BackgroundFit, BackgroundImage, BackgroundPosition, BackgroundRepeat, BackgroundSize,
    BackgroundSizeAxis, LinearGradient, TextStyle,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use cssparser::{Parser, ParserInput, Token};
use ctml_css::parse_color_with_opacity_and_profiles;

pub(super) fn apply_background_shorthand(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("none") {
        super::background_box::reset_background_image_layers(style);
        style.background = [255, 255, 255];
        style.background_color = None;
    } else if let Some(images) = parse_background_images(style, value) {
        super::background_box::reset_background_image_layers(style);
        style.background_images = images;
        style.background = [255, 255, 255];
        style.background_color = None;
    } else if let Some((channels, alpha)) = parse_background_color(style, value) {
        super::background_box::reset_background_image_layers(style);
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

pub(super) fn apply_background_repeat(style: &mut TextStyle, value: &str) {
    let repeats = split_top_level_commas(value)
        .into_iter()
        .map(parse_background_repeat_layer)
        .collect::<Option<Vec<_>>>();
    if let Some(repeats) = repeats.filter(|repeats| !repeats.is_empty()) {
        style.background_repeats = repeats;
    }
}

pub(super) fn apply_background_position(style: &mut TextStyle, value: &str) {
    let positions = split_top_level_commas(value)
        .into_iter()
        .map(|layer| parse_background_position_layer(style, layer))
        .collect::<Option<Vec<_>>>();
    if let Some(positions) = positions.filter(|positions| !positions.is_empty()) {
        style.background_positions = positions;
    }
}

pub(super) fn apply_background_size(style: &mut TextStyle, value: &str) {
    let sizes = split_top_level_commas(value)
        .into_iter()
        .map(|layer| parse_background_size_layer(style, layer))
        .collect::<Option<Vec<_>>>();
    if let Some(sizes) = sizes.filter(|sizes| !sizes.is_empty()) {
        style.background_sizes = sizes;
    }
}

fn parse_background_size_layer(style: &TextStyle, value: &str) -> Option<BackgroundSize> {
    let mut components = value.split_ascii_whitespace();
    let first = components.next()?;
    let second = components.next();
    if components.next().is_some() {
        return None;
    }
    if second.is_none() {
        match first.to_ascii_lowercase().as_str() {
            "cover" => {
                return Some(BackgroundSize {
                    fit: BackgroundFit::Cover,
                    ..BackgroundSize::default()
                });
            }
            "contain" => {
                return Some(BackgroundSize {
                    fit: BackgroundFit::Contain,
                    ..BackgroundSize::default()
                });
            }
            _ => {}
        }
    }
    let width = parse_background_size_axis(style, first)?;
    let height = if let Some(value) = second {
        parse_background_size_axis(style, value)?
    } else {
        None
    };
    Some(BackgroundSize {
        width,
        height,
        fit: BackgroundFit::Auto,
    })
}

fn parse_background_size_axis(
    style: &TextStyle,
    value: &str,
) -> Option<Option<BackgroundSizeAxis>> {
    if value.eq_ignore_ascii_case("auto") {
        return Some(None);
    }
    if let Some(percentage) = ctml_css::parse_css_percentage(value) {
        return (percentage >= 0.0).then_some(Some(BackgroundSizeAxis {
            value: percentage / 100.0,
            percentage: true,
        }));
    }
    ctml_css::parse_nonnegative_font_relative_length_with_viewport(
        value,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    )
    .filter(|length| length.is_finite())
    .map(|value| {
        Some(BackgroundSizeAxis {
            value,
            percentage: false,
        })
    })
}

fn parse_background_position_layer(style: &TextStyle, value: &str) -> Option<BackgroundPosition> {
    let mut components = value.split_ascii_whitespace().collect::<Vec<_>>();
    if components.is_empty() || components.len() > 2 {
        return None;
    }
    if components.len() == 1 {
        let value = components[0];
        return if matches!(value.to_ascii_lowercase().as_str(), "top" | "bottom") {
            Some(BackgroundPosition {
                x_percent: 0.5,
                y_percent: parse_position_axis(style, value, false)?.0,
                y_offset: parse_position_axis(style, value, false)?.1,
                ..BackgroundPosition::default()
            })
        } else {
            Some(BackgroundPosition {
                x_percent: parse_position_axis(style, value, true)?.0,
                x_offset: parse_position_axis(style, value, true)?.1,
                y_percent: 0.5,
                ..BackgroundPosition::default()
            })
        };
    }
    let first = components[0].to_ascii_lowercase();
    let second = components[1].to_ascii_lowercase();
    if matches!(first.as_str(), "top" | "bottom")
        || (first == "center" && matches!(second.as_str(), "left" | "right"))
    {
        components.swap(0, 1);
    }
    let (x_percent, x_offset) = parse_position_axis(style, components[0], true)?;
    let (y_percent, y_offset) = parse_position_axis(style, components[1], false)?;
    Some(BackgroundPosition {
        x_percent,
        x_offset,
        y_percent,
        y_offset,
    })
}

fn parse_position_axis(style: &TextStyle, value: &str, horizontal: bool) -> Option<(f32, f32)> {
    match value.to_ascii_lowercase().as_str() {
        "left" if horizontal => Some((0.0, 0.0)),
        "right" if horizontal => Some((1.0, 0.0)),
        "top" if !horizontal => Some((0.0, 0.0)),
        "bottom" if !horizontal => Some((1.0, 0.0)),
        "center" => Some((0.5, 0.0)),
        _ if ctml_css::parse_css_percentage(value).is_some() => {
            Some((ctml_css::parse_css_percentage(value)? / 100.0, 0.0))
        }
        _ => Some((
            0.0,
            ctml_css::parse_signed_font_relative_length_with_viewport(
                value,
                style.font_size,
                style.line_height,
                style.font_relative_units,
                style.viewport_size,
            )?,
        )),
    }
}

fn parse_background_repeat_layer(value: &str) -> Option<BackgroundRepeat> {
    let mut values = value.split_ascii_whitespace();
    let first = values.next()?.to_ascii_lowercase();
    let second = values.next().map(str::to_ascii_lowercase);
    if values.next().is_some() {
        return None;
    }
    let repeat_axis = |value: &str| match value {
        "repeat" => Some(true),
        "no-repeat" => Some(false),
        _ => None,
    };
    match (first.as_str(), second) {
        ("repeat-x", None) => Some(BackgroundRepeat::REPEAT_X),
        ("repeat-y", None) => Some(BackgroundRepeat::REPEAT_Y),
        ("repeat", None) => Some(BackgroundRepeat::REPEAT),
        ("no-repeat", None) => Some(BackgroundRepeat::NO_REPEAT),
        (x, Some(y)) => Some(BackgroundRepeat {
            x: repeat_axis(x)?,
            y: repeat_axis(&y)?,
        }),
        _ => None,
    }
}

fn parse_background_images(style: &TextStyle, value: &str) -> Option<Vec<BackgroundImage>> {
    if value.eq_ignore_ascii_case("none") {
        return Some(Vec::new());
    }
    let layers = split_top_level_commas(value);
    let mut images = Vec::with_capacity(layers.len());
    for layer in layers {
        let layer_name = layer.trim_start().to_ascii_lowercase();
        if layer_name.starts_with("linear-gradient(")
            || layer_name.starts_with("repeating-linear-gradient(")
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
    let repeating = match value[..opening].to_ascii_lowercase().as_str() {
        "linear-gradient" => false,
        "repeating-linear-gradient" => true,
        _ => return None,
    };
    if !value.ends_with(')') {
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
        repeating,
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
