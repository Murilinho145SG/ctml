use crate::style::{
    BackgroundBlendMode, BackgroundFit, BackgroundImage, BackgroundPosition, BackgroundRepeat,
    BackgroundSize, BackgroundSizeAxis, TextStyle,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use cssparser::{Parser, ParserInput, Token};
use ctml_css::parse_color_with_opacity_and_profiles;

pub(super) fn apply_background_image(style: &mut TextStyle, value: &str) {
    let Some(images) = parse_background_images(style, value) else {
        return;
    };
    style.background_images = images;
}

pub(super) fn apply_background_blend_mode(style: &mut TextStyle, value: &str) {
    let modes = split_top_level_commas(value)
        .into_iter()
        .map(parse_blend_mode)
        .collect::<Option<Vec<_>>>();
    if let Some(modes) = modes.filter(|modes| !modes.is_empty()) {
        style.background_blend_modes = modes;
    }
}

pub(super) fn apply_mix_blend_mode(style: &mut TextStyle, value: &str) {
    if let Some(mode) = parse_blend_mode(value) {
        style.mix_blend_mode = mode;
    }
}

pub(super) fn parse_blend_mode(value: &str) -> Option<BackgroundBlendMode> {
    Some(match value.trim().to_ascii_lowercase().as_str() {
        "normal" => BackgroundBlendMode::Normal,
        "multiply" => BackgroundBlendMode::Multiply,
        "screen" => BackgroundBlendMode::Screen,
        "overlay" => BackgroundBlendMode::Overlay,
        "darken" => BackgroundBlendMode::Darken,
        "lighten" => BackgroundBlendMode::Lighten,
        "color-dodge" => BackgroundBlendMode::ColorDodge,
        "color-burn" => BackgroundBlendMode::ColorBurn,
        "hard-light" => BackgroundBlendMode::HardLight,
        "soft-light" => BackgroundBlendMode::SoftLight,
        "difference" => BackgroundBlendMode::Difference,
        "exclusion" => BackgroundBlendMode::Exclusion,
        "hue" => BackgroundBlendMode::Hue,
        "saturation" => BackgroundBlendMode::Saturation,
        "color" => BackgroundBlendMode::Color,
        "luminosity" => BackgroundBlendMode::Luminosity,
        _ => return None,
    })
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

pub(super) fn parse_background_size_layer(
    style: &TextStyle,
    value: &str,
) -> Option<BackgroundSize> {
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

pub(super) fn parse_background_position_layer(
    style: &TextStyle,
    value: &str,
) -> Option<BackgroundPosition> {
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

pub(super) fn parse_background_repeat_layer(value: &str) -> Option<BackgroundRepeat> {
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

pub(super) fn parse_background_images(
    style: &TextStyle,
    value: &str,
) -> Option<Vec<BackgroundImage>> {
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
            images.push(BackgroundImage::LinearGradient(
                super::background_gradients::parse_linear_gradient(style, layer)?,
            ));
        } else if layer_name.starts_with("radial-gradient(")
            || layer_name.starts_with("repeating-radial-gradient(")
        {
            images.push(BackgroundImage::RadialGradient(
                super::background_gradients::parse_radial_gradient(style, layer)?,
            ));
        } else if layer_name.starts_with("conic-gradient(")
            || layer_name.starts_with("repeating-conic-gradient(")
        {
            images.push(BackgroundImage::ConicGradient(
                super::background_gradients::parse_conic_gradient(style, layer)?,
            ));
        } else {
            images.push(BackgroundImage::Encoded(parse_data_url(layer)?));
        }
    }
    (!images.is_empty()).then_some(images)
}

pub(super) fn parse_data_url(value: &str) -> Option<std::sync::Arc<[u8]>> {
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

pub(super) fn split_top_level_commas(value: &str) -> Vec<&str> {
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

pub(super) fn parse_background_color(style: &TextStyle, value: &str) -> Option<([u8; 3], f32)> {
    if value.eq_ignore_ascii_case("currentcolor") {
        Some((
            [style.color[0], style.color[1], style.color[2]],
            style.color_opacity,
        ))
    } else {
        parse_color_with_opacity_and_profiles(value, &style.color_profiles)
    }
}

pub(super) fn set_background_color(style: &mut TextStyle, channels: [u8; 3], alpha: f32) {
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
