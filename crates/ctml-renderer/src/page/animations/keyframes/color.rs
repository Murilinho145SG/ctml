use crate::style::TransitionEasing;

pub(super) fn collect_color_frames(
    rule: &ctml_css::CssKeyframes,
    properties: &[&str],
    base_color: [u8; 4],
    current_color: [u8; 4],
) -> Option<Vec<(f32, [u8; 4])>> {
    let frames = rule
        .frames
        .iter()
        .filter_map(|frame| {
            let declaration = frame
                .declarations
                .iter()
                .rev()
                .find(|declaration| properties.contains(&declaration.property.as_str()))?;
            let color = parse_color_value(&declaration.value, current_color)?;
            Some(frame.offsets.iter().map(move |offset| (*offset, color)))
        })
        .flatten()
        .collect::<Vec<_>>();
    normalize_frames(frames, base_color)
}

pub(super) fn collect_border_side_frames(
    rule: &ctml_css::CssKeyframes,
    side: usize,
    base_color: [u8; 4],
    current_color: [u8; 4],
) -> Option<Vec<(f32, [u8; 4])>> {
    let side_property = [
        "border-top-color",
        "border-right-color",
        "border-bottom-color",
        "border-left-color",
    ][side];
    let mut frames = Vec::new();
    for frame in &rule.frames {
        let color = frame.declarations.iter().rev().find_map(|declaration| {
            let value = if declaration.property == side_property {
                Some(declaration.value.as_str())
            } else if declaration.property == "border-color" {
                border_color_component(&declaration.value, side)
            } else {
                None
            }?;
            parse_color_value(value, current_color)
        });
        if let Some(color) = color {
            frames.extend(frame.offsets.iter().map(|offset| (*offset, color)));
        }
    }
    normalize_frames(frames, base_color)
}

fn border_color_component(value: &str, side: usize) -> Option<&str> {
    let components = split_css_components(value)?;
    let colors = components
        .iter()
        .map(|component| parse_color_value(component, [0; 4]))
        .collect::<Option<Vec<_>>>()?;
    let mapped_side = match colors.len() {
        1 => 0,
        2 => side % 2,
        3 => match side {
            0 => 0,
            1 | 3 => 1,
            _ => 2,
        },
        4 => side,
        _ => return None,
    };
    Some(components[mapped_side])
}

fn split_css_components(value: &str) -> Option<Vec<&str>> {
    let mut components = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
            }
            whitespace if whitespace.is_whitespace() && depth == 0 => {
                if let Some(component_start) = start.take() {
                    components.push(&value[component_start..index]);
                }
                continue;
            }
            _ => {}
        }
        start.get_or_insert(index);
    }
    if depth != 0 {
        return None;
    }
    if let Some(component_start) = start {
        components.push(&value[component_start..]);
    }
    (!components.is_empty()).then_some(components)
}

fn parse_color_value(value: &str, current_color: [u8; 4]) -> Option<[u8; 4]> {
    if value.eq_ignore_ascii_case("currentcolor") {
        Some(current_color)
    } else {
        ctml_css::parse_color(value)
    }
}

fn normalize_frames(
    mut frames: Vec<(f32, [u8; 4])>,
    base_color: [u8; 4],
) -> Option<Vec<(f32, [u8; 4])>> {
    if frames.is_empty() {
        return None;
    }
    frames.sort_by(|left, right| left.0.total_cmp(&right.0));
    let mut unique = Vec::<(f32, [u8; 4])>::new();
    for frame in frames {
        if unique.last().is_some_and(|last| last.0 == frame.0) {
            *unique.last_mut().expect("last frame exists") = frame;
        } else {
            unique.push(frame);
        }
    }
    if unique[0].0 > 0.0 {
        unique.insert(0, (0.0, base_color));
    }
    if unique.last().is_some_and(|last| last.0 < 1.0) {
        unique.push((1.0, base_color));
    }
    Some(unique)
}

pub(super) fn sample_color(
    frames: &[(f32, [u8; 4])],
    progress: f32,
    easing: TransitionEasing,
) -> [u8; 4] {
    let Some(first) = frames.first() else {
        return [0; 4];
    };
    if progress <= first.0 {
        return first.1;
    }
    for pair in frames.windows(2) {
        let (start, from) = pair[0];
        let (end, to) = pair[1];
        if progress <= end {
            let local = if end <= start {
                1.0
            } else {
                ((progress - start) / (end - start)).clamp(0.0, 1.0)
            };
            return interpolate(from, to, super::super::ease_transition(local, easing));
        }
    }
    frames.last().map_or(first.1, |frame| frame.1)
}

fn interpolate(from: [u8; 4], to: [u8; 4], progress: f32) -> [u8; 4] {
    let from_alpha = f32::from(from[3]) / 255.0;
    let to_alpha = f32::from(to[3]) / 255.0;
    let alpha = from_alpha + (to_alpha - from_alpha) * progress;
    let mut color = [0; 4];
    color[3] = (alpha * 255.0).round().clamp(0.0, 255.0) as u8;
    for (index, channel) in color.iter_mut().take(3).enumerate() {
        let from = f32::from(from[index]) / 255.0 * from_alpha;
        let to = f32::from(to[index]) / 255.0 * to_alpha;
        *channel = if alpha <= f32::EPSILON {
            0
        } else {
            ((from + (to - from) * progress) / alpha * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8
        };
    }
    color
}
