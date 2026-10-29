use cosmic_text::{Stretch, Style, Weight};

use crate::style::{FontVariantCaps, FontVariantPosition, TextStyle};
use ctml_css::parse_length;

pub(super) fn relative_font_weight(inherited: Weight, bolder: bool) -> Weight {
    let weight = inherited.0;
    let resolved = match (bolder, weight) {
        (true, weight) if weight < 100 => 400,
        (true, 100..350) => 400,
        (true, 350..550) => 700,
        (true, 550..750) => 900,
        (true, 750..900) => 900,
        (true, _) => weight,
        (false, weight) if weight < 100 => weight,
        (false, 100..350) => 100,
        (false, 350..550) => 100,
        (false, 550..750) => 400,
        (false, 750..900) => 700,
        (false, _) => 700,
    };
    Weight(resolved)
}

pub(super) fn parse_font_stretch(value: &str) -> Option<Stretch> {
    let keyword = match value.to_ascii_lowercase().as_str() {
        "ultra-condensed" => Some(Stretch::UltraCondensed),
        "extra-condensed" => Some(Stretch::ExtraCondensed),
        "condensed" => Some(Stretch::Condensed),
        "semi-condensed" => Some(Stretch::SemiCondensed),
        "normal" => Some(Stretch::Normal),
        "semi-expanded" => Some(Stretch::SemiExpanded),
        "expanded" => Some(Stretch::Expanded),
        "extra-expanded" => Some(Stretch::ExtraExpanded),
        "ultra-expanded" => Some(Stretch::UltraExpanded),
        _ => None,
    };
    if keyword.is_some() {
        return keyword;
    }

    let percentage = value.strip_suffix('%')?.trim().parse::<f32>().ok()?;
    if !percentage.is_finite() || percentage < 0.0 {
        return None;
    }
    [
        (50.0, Stretch::UltraCondensed),
        (62.5, Stretch::ExtraCondensed),
        (75.0, Stretch::Condensed),
        (87.5, Stretch::SemiCondensed),
        (100.0, Stretch::Normal),
        (112.5, Stretch::SemiExpanded),
        (125.0, Stretch::Expanded),
        (150.0, Stretch::ExtraExpanded),
        (200.0, Stretch::UltraExpanded),
    ]
    .into_iter()
    .min_by(|(left, _), (right, _)| {
        (left - percentage)
            .abs()
            .total_cmp(&(right - percentage).abs())
    })
    .map(|(_, stretch)| stretch)
}

pub(super) fn apply_font_shorthand(style: &mut TextStyle, value: &str) {
    let components = split_components(value);
    let Some(size_index) = components.iter().position(|(start, end)| {
        let component = &value[*start..*end];
        let (size, _) = component.split_once('/').unwrap_or((component, ""));
        parse_font_size(size, style.font_size).is_some()
    }) else {
        return;
    };

    let mut font_style = None;
    let mut variant_caps = None;
    let mut weight = None;
    let mut stretch = None;
    let mut saw_normal = false;
    for (start, end) in &components[..size_index] {
        let component = &value[*start..*end];
        let lower = component.to_ascii_lowercase();
        if lower == "normal" {
            if saw_normal {
                return;
            }
            saw_normal = true;
        } else if matches!(lower.as_str(), "italic" | "oblique") {
            if font_style.is_some() {
                return;
            }
            font_style = Some(if lower == "italic" {
                Style::Italic
            } else {
                Style::Oblique
            });
        } else if lower == "small-caps" {
            if variant_caps.replace(FontVariantCaps::SmallCaps).is_some() {
                return;
            }
        } else if matches!(lower.as_str(), "bold" | "bolder" | "lighter")
            || component
                .parse::<u16>()
                .is_ok_and(|value| (1..=1000).contains(&value))
        {
            if weight.is_some() {
                return;
            }
            weight = Some(match lower.as_str() {
                "bold" => Weight::BOLD,
                "bolder" => relative_font_weight(style.weight, true),
                "lighter" => relative_font_weight(style.weight, false),
                _ => Weight(component.parse::<u16>().ok().unwrap()),
            });
        } else if let Some(parsed_stretch) = parse_font_stretch(component) {
            if stretch.replace(parsed_stretch).is_some() {
                return;
            }
        } else {
            return;
        }
    }

    let (size_start, size_end) = components[size_index];
    let size_component = &value[size_start..size_end];
    let (size_value, inline_line_height) = split_top_level_slash(size_component);
    let Some(font_size) = parse_font_size(size_value, style.font_size) else {
        return;
    };
    let mut family_index = size_index + 1;
    let line_height = if inline_line_height.is_some() {
        let inline_value = inline_line_height.unwrap();
        if inline_value.is_empty() {
            let Some((line_height, next_index)) =
                parse_following_line_height(&components, value, family_index, font_size)
            else {
                return;
            };
            family_index = next_index;
            Some(line_height)
        } else {
            let Some(line_height) = parse_line_height(inline_value, font_size) else {
                return;
            };
            Some(line_height)
        }
    } else if let Some((start, end)) = components.get(family_index).copied() {
        let slash_component = &value[start..end];
        if slash_component == "/" || slash_component.starts_with('/') {
            let line_value = slash_component.trim_start_matches('/');
            if line_value.is_empty() {
                let Some((line_height, next_index)) =
                    parse_following_line_height(&components, value, family_index + 1, font_size)
                else {
                    return;
                };
                family_index = next_index;
                Some(line_height)
            } else {
                family_index += 1;
                let Some(line_height) = parse_line_height(line_value, font_size) else {
                    return;
                };
                Some(line_height)
            }
        } else {
            None
        }
    } else {
        None
    };
    let family_start = components.get(family_index).map(|(start, _)| *start);
    let Some(family_start) = family_start else {
        return;
    };
    let family = value[family_start..].trim();
    if family.is_empty() {
        return;
    }

    reset_font(style);
    style.family = family.to_owned();
    style.font_size = font_size;
    style.weight = weight.unwrap_or(Weight::NORMAL);
    style.font_style = font_style.unwrap_or(Style::Normal);
    style.stretch = stretch.unwrap_or(Stretch::Normal);
    style.font_variant_caps = variant_caps.unwrap_or(FontVariantCaps::Normal);
    if let Some(Some(line_height)) = line_height {
        style.line_height = line_height;
        style.line_height_multiplier = None;
        style.line_height_normal = false;
    }
}

pub(super) fn reset_font(style: &mut TextStyle) {
    style.family = "serif".to_owned();
    style.font_size = 16.0;
    style.font_scale = 1.0;
    style.line_height = 16.0 * 1.2;
    style.line_height_multiplier = None;
    style.line_height_normal = true;
    style.weight = Weight::NORMAL;
    style.font_style = Style::Normal;
    style.stretch = Stretch::Normal;
    style.kerning = None;
    style.common_ligatures = None;
    style.discretionary_ligatures = None;
    style.historical_ligatures = None;
    style.contextual_alternates = None;
    style.font_variant_caps = FontVariantCaps::Normal;
    style.font_variant_position = FontVariantPosition::Normal;
    style.font_variant_numeric.clear();
    style.font_variant_east_asian.clear();
    style.font_feature_settings.clear();
}

pub(super) fn inherit_font(style: &mut TextStyle, inherited: &TextStyle) {
    style.family.clone_from(&inherited.family);
    style.font_size = inherited.font_size;
    style.font_scale = inherited.font_scale;
    style.line_height = inherited.line_height;
    style.line_height_multiplier = inherited.line_height_multiplier;
    style.line_height_normal = inherited.line_height_normal;
    style.weight = inherited.weight;
    style.font_style = inherited.font_style;
    style.stretch = inherited.stretch;
    style.kerning = inherited.kerning;
    style.common_ligatures = inherited.common_ligatures;
    style.discretionary_ligatures = inherited.discretionary_ligatures;
    style.historical_ligatures = inherited.historical_ligatures;
    style.contextual_alternates = inherited.contextual_alternates;
    style.font_variant_caps = inherited.font_variant_caps;
    style.font_variant_position = inherited.font_variant_position;
    style
        .font_variant_numeric
        .clone_from(&inherited.font_variant_numeric);
    style
        .font_variant_east_asian
        .clone_from(&inherited.font_variant_east_asian);
    style
        .font_feature_settings
        .clone_from(&inherited.font_feature_settings);
}

fn parse_following_line_height(
    components: &[(usize, usize)],
    source: &str,
    line_height_index: usize,
    font_size: f32,
) -> Option<(Option<f32>, usize)> {
    let (start, end) = components.get(line_height_index).copied()?;
    Some((
        parse_line_height(&source[start..end], font_size)?,
        line_height_index + 1,
    ))
}

fn parse_line_height(value: &str, font_size: f32) -> Option<Option<f32>> {
    if value.eq_ignore_ascii_case("normal") {
        return Some(None);
    }
    if let Ok(multiplier) = value.parse::<f32>() {
        return (multiplier.is_finite() && multiplier >= 0.0)
            .then_some(Some(font_size * multiplier));
    }
    parse_length(value, font_size).map(Some)
}

fn split_top_level_slash(value: &str) -> (&str, Option<&str>) {
    let mut depth: usize = 0;
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == active_quote {
                quote = None;
            }
        } else {
            match character {
                '\'' | '"' => quote = Some(character),
                '(' => depth += 1,
                ')' => depth = depth.saturating_sub(1),
                '/' if depth == 0 => return (&value[..index], Some(&value[index + 1..])),
                _ => {}
            }
        }
    }
    (value, None)
}

fn split_components(value: &str) -> Vec<(usize, usize)> {
    let mut components = Vec::new();
    let mut start = None;
    let mut depth: usize = 0;
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if start.is_none() && !character.is_whitespace() {
            start = Some(index);
        }
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == active_quote {
                quote = None;
            }
        } else {
            match character {
                '\'' | '"' => quote = Some(character),
                '(' => depth += 1,
                ')' => depth = depth.saturating_sub(1),
                character if character.is_whitespace() && depth == 0 => {
                    if let Some(start) = start.take() {
                        components.push((start, index));
                    }
                }
                _ => {}
            }
        }
    }
    if let Some(start) = start {
        components.push((start, value.len()));
    }
    components
}

pub(in crate::css_style) fn parse_font_size(value: &str, relative_to: f32) -> Option<f32> {
    let absolute_size = match value.trim().to_ascii_lowercase().as_str() {
        "xx-small" => Some(9.0),
        "x-small" => Some(10.0),
        "small" => Some(13.0),
        "medium" => Some(16.0),
        "large" => Some(18.0),
        "x-large" => Some(24.0),
        "xx-large" => Some(32.0),
        "xxx-large" => Some(48.0),
        "larger" => Some(relative_to * 1.2),
        "smaller" => Some(relative_to / 1.2),
        _ => None,
    };
    absolute_size.or_else(|| parse_length(value, relative_to))
}
