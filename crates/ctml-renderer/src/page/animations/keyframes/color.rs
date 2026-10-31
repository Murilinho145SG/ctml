use crate::style::TransitionEasing;

pub(super) fn collect_color_frames(
    rule: &ctml_css::CssKeyframes,
    properties: &[&str],
    base_color: [u8; 4],
    current_color: [u8; 4],
) -> Option<Vec<(f32, [u8; 4])>> {
    let mut frames = rule
        .frames
        .iter()
        .filter_map(|frame| {
            let declaration = frame
                .declarations
                .iter()
                .rev()
                .find(|declaration| properties.contains(&declaration.property.as_str()))?;
            let color = if declaration.value.eq_ignore_ascii_case("currentcolor") {
                current_color
            } else {
                ctml_css::parse_color(&declaration.value)?
            };
            Some(frame.offsets.iter().map(move |offset| (*offset, color)))
        })
        .flatten()
        .collect::<Vec<_>>();
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
