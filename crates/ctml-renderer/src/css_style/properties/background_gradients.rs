use crate::style::{
    ConicGradient, LinearGradient, RadialGradient, RadialGradientExtent, RadialGradientShape,
    TextStyle,
};
use ctml_css::parse_color_with_opacity_and_profiles;

use super::background::split_top_level_commas;

pub(super) fn parse_linear_gradient(style: &TextStyle, value: &str) -> Option<LinearGradient> {
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

pub(super) fn parse_radial_gradient(style: &TextStyle, value: &str) -> Option<RadialGradient> {
    let value = value.trim();
    let opening = value.find('(')?;
    let name = value[..opening].trim();
    if !name.eq_ignore_ascii_case("radial-gradient") || !value.ends_with(')') {
        return None;
    }
    let arguments = &value[opening + 1..value.len() - 1];
    let mut components = split_top_level_commas(arguments).into_iter();
    let first = components.next()?;
    let (shape, extent, center_x_percent, center_y_percent, first_stop) =
        match parse_radial_descriptor(first)? {
            Some((shape, extent, center)) => (shape, extent, center.0, center.1, None),
            None => (
                RadialGradientShape::Ellipse,
                RadialGradientExtent::FarthestCorner,
                0.5,
                0.5,
                Some(first),
            ),
        };
    let mut stops = Vec::new();
    if let Some(first_stop) = first_stop {
        stops.push(parse_gradient_stop(style, first_stop)?);
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
    Some(RadialGradient {
        shape,
        extent,
        center_x_percent,
        center_y_percent,
        stops,
    })
}

pub(super) fn parse_conic_gradient(style: &TextStyle, value: &str) -> Option<ConicGradient> {
    let value = value.trim();
    let opening = value.find('(')?;
    let repeating = value[..opening]
        .trim()
        .eq_ignore_ascii_case("repeating-conic-gradient");
    if !(repeating
        || value[..opening]
            .trim()
            .eq_ignore_ascii_case("conic-gradient"))
        || !value.ends_with(')')
    {
        return None;
    }
    let arguments = &value[opening + 1..value.len() - 1];
    let mut components = split_top_level_commas(arguments).into_iter();
    let first = components.next()?;
    let (from_degrees, center_x_percent, center_y_percent, first_stop) =
        match parse_conic_descriptor(first)? {
            Some((angle, center)) => (angle, center.0, center.1, None),
            None => (0.0, 0.5, 0.5, Some(first)),
        };
    let mut stops = Vec::new();
    if let Some(first_stop) = first_stop {
        stops.push(parse_conic_stop(style, first_stop)?);
    }
    stops.extend(
        components
            .map(|component| parse_conic_stop(style, component))
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
    Some(ConicGradient {
        from_degrees: from_degrees.rem_euclid(360.0),
        center_x_percent,
        center_y_percent,
        stops,
        repeating,
    })
}

fn parse_conic_descriptor(value: &str) -> Option<Option<(f32, (f32, f32))>> {
    let tokens = value.split_ascii_whitespace().collect::<Vec<_>>();
    let at = tokens
        .iter()
        .position(|token| token.eq_ignore_ascii_case("at"));
    let prefix = &tokens[..at.unwrap_or(tokens.len())];
    let has_from = prefix
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("from"));
    if at.is_none() && !has_from {
        return Some(None);
    }
    let angle = if has_from {
        if prefix.len() != 2 {
            return None;
        }
        parse_css_angle(prefix[1])?
    } else {
        if !prefix.is_empty() {
            return None;
        }
        0.0
    };
    let center = if let Some(at) = at {
        parse_radial_position(&tokens[at + 1..])?
    } else {
        (0.5, 0.5)
    };
    Some(Some((angle, center)))
}

fn parse_conic_stop(style: &TextStyle, value: &str) -> Option<([u8; 4], Option<f32>)> {
    let value = value.trim();
    let split = last_top_level_whitespace(value);
    if let Some(split) = split {
        let candidate = value[split..].trim();
        if let Some(position) = parse_conic_stop_position(candidate) {
            let (color, _) = parse_gradient_stop(style, value[..split].trim())?;
            return Some((color, Some(position)));
        }
    }
    let (color, position) = parse_gradient_stop(style, value)?;
    Some((color, position))
}

fn parse_conic_stop_position(value: &str) -> Option<f32> {
    ctml_css::parse_css_percentage(value)
        .map(|percentage| percentage / 100.0)
        .or_else(|| parse_css_angle(value).map(|degrees| degrees / 360.0))
}

fn parse_css_angle(value: &str) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
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

fn parse_radial_descriptor(
    value: &str,
) -> Option<Option<(RadialGradientShape, RadialGradientExtent, (f32, f32))>> {
    let tokens = value.split_ascii_whitespace().collect::<Vec<_>>();
    let at = tokens
        .iter()
        .position(|token| token.eq_ignore_ascii_case("at"));
    let descriptor_tokens = &tokens[..at.unwrap_or(tokens.len())];
    let is_descriptor = at.is_some()
        || descriptor_tokens.iter().any(|token| {
            matches!(
                token.to_ascii_lowercase().as_str(),
                "circle"
                    | "ellipse"
                    | "closest-side"
                    | "farthest-side"
                    | "closest-corner"
                    | "farthest-corner"
            )
        });
    if !is_descriptor {
        return Some(None);
    }
    let mut shape = RadialGradientShape::Ellipse;
    let mut extent = RadialGradientExtent::FarthestCorner;
    let mut has_shape = false;
    let mut has_extent = false;
    for token in descriptor_tokens {
        match token.to_ascii_lowercase().as_str() {
            "circle" if !has_shape => {
                shape = RadialGradientShape::Circle;
                has_shape = true;
            }
            "ellipse" if !has_shape => {
                shape = RadialGradientShape::Ellipse;
                has_shape = true;
            }
            "closest-side" if !has_extent => {
                extent = RadialGradientExtent::ClosestSide;
                has_extent = true;
            }
            "farthest-side" if !has_extent => {
                extent = RadialGradientExtent::FarthestSide;
                has_extent = true;
            }
            "closest-corner" if !has_extent => {
                extent = RadialGradientExtent::ClosestCorner;
                has_extent = true;
            }
            "farthest-corner" if !has_extent => {
                extent = RadialGradientExtent::FarthestCorner;
                has_extent = true;
            }
            _ => return None,
        }
    }
    let center = if let Some(at) = at {
        parse_radial_position(&tokens[at + 1..])?
    } else {
        (0.5, 0.5)
    };
    Some(Some((shape, extent, center)))
}

fn parse_radial_position(tokens: &[&str]) -> Option<(f32, f32)> {
    if tokens.is_empty() || tokens.len() > 2 {
        return None;
    }
    if tokens.len() == 1 {
        let value = tokens[0];
        return if matches!(value.to_ascii_lowercase().as_str(), "top" | "bottom") {
            Some((0.5, parse_radial_axis(value, false)?))
        } else {
            Some((parse_radial_axis(value, true)?, 0.5))
        };
    }
    let (mut x, mut y) = (tokens[0], tokens[1]);
    if matches!(x.to_ascii_lowercase().as_str(), "top" | "bottom")
        || (x.eq_ignore_ascii_case("center")
            && matches!(y.to_ascii_lowercase().as_str(), "left" | "right"))
    {
        std::mem::swap(&mut x, &mut y);
    }
    Some((parse_radial_axis(x, true)?, parse_radial_axis(y, false)?))
}

fn parse_radial_axis(value: &str, horizontal: bool) -> Option<f32> {
    match value.to_ascii_lowercase().as_str() {
        "left" if horizontal => Some(0.0),
        "right" if horizontal => Some(1.0),
        "top" if !horizontal => Some(0.0),
        "bottom" if !horizontal => Some(1.0),
        "center" => Some(0.5),
        _ => ctml_css::parse_css_percentage(value).map(|value| value / 100.0),
    }
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
