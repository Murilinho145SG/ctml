use cosmic_text::{FontSystem, fontdb};

use crate::{FontFace, FontFaceSource};

pub(super) fn parse_font_stretch(value: &str) -> Option<fontdb::Stretch> {
    let lower = value.trim().to_ascii_lowercase();
    Some(match lower.as_str() {
        "ultra-condensed" => fontdb::Stretch::UltraCondensed,
        "extra-condensed" => fontdb::Stretch::ExtraCondensed,
        "condensed" => fontdb::Stretch::Condensed,
        "semi-condensed" => fontdb::Stretch::SemiCondensed,
        "normal" => fontdb::Stretch::Normal,
        "semi-expanded" => fontdb::Stretch::SemiExpanded,
        "expanded" => fontdb::Stretch::Expanded,
        "extra-expanded" => fontdb::Stretch::ExtraExpanded,
        "ultra-expanded" => fontdb::Stretch::UltraExpanded,
        _ => {
            let percentage = lower.strip_suffix('%')?.trim().parse::<f32>().ok()?;
            if !percentage.is_finite() || percentage <= 0.0 {
                return None;
            }
            let ranges = [
                (50.0, fontdb::Stretch::UltraCondensed),
                (62.5, fontdb::Stretch::ExtraCondensed),
                (75.0, fontdb::Stretch::Condensed),
                (87.5, fontdb::Stretch::SemiCondensed),
                (100.0, fontdb::Stretch::Normal),
                (112.5, fontdb::Stretch::SemiExpanded),
                (125.0, fontdb::Stretch::Expanded),
                (150.0, fontdb::Stretch::ExtraExpanded),
                (200.0, fontdb::Stretch::UltraExpanded),
            ];
            return ranges
                .into_iter()
                .min_by(|(left, _), (right, _)| {
                    (left - percentage)
                        .abs()
                        .total_cmp(&(right - percentage).abs())
                })
                .map(|(_, stretch)| stretch);
        }
    })
}

fn font_face_query_values(font: &FontFace) -> (fontdb::Weight, fontdb::Stretch, fontdb::Style) {
    let weight = font.weight.map_or(fontdb::Weight::NORMAL, fontdb::Weight);
    let style = match font
        .style
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("italic") => fontdb::Style::Italic,
        Some("oblique") => fontdb::Style::Oblique,
        _ => fontdb::Style::Normal,
    };
    let stretch = font
        .stretch
        .as_deref()
        .and_then(parse_font_stretch)
        .unwrap_or(fontdb::Stretch::Normal);
    (weight, stretch, style)
}

fn apply_font_face_descriptors(face: &mut fontdb::FaceInfo, font: &FontFace) {
    if let Some(weight) = font.weight {
        face.weight = fontdb::Weight(weight);
    }
    if let Some(style) = font.style.as_deref() {
        face.style = match style.trim().to_ascii_lowercase().as_str() {
            "italic" => fontdb::Style::Italic,
            "oblique" => fontdb::Style::Oblique,
            _ => fontdb::Style::Normal,
        };
    }
    if let Some(stretch) = font.stretch.as_deref().and_then(parse_font_stretch) {
        face.stretch = stretch;
    }
}

pub(super) fn register_font_faces(font_system: &mut FontSystem, fonts: &[FontFace]) {
    for font in fonts {
        let (weight, stretch, style) = font_face_query_values(font);
        for source in &font.sources {
            let candidates = match source {
                FontFaceSource::Embedded(bytes) => {
                    let database = font_system.db_mut();
                    let ids = database.load_font_source(fontdb::Source::Binary(
                        std::sync::Arc::new(bytes.clone()),
                    ));
                    ids.iter()
                        .filter_map(|id| database.face(*id).cloned())
                        .collect::<Vec<_>>()
                }
                FontFaceSource::Local(name) => {
                    let database = font_system.db();
                    let families = [fontdb::Family::Name(name)];
                    database
                        .query(&fontdb::Query {
                            families: &families,
                            weight,
                            stretch,
                            style,
                        })
                        .and_then(|id| database.face(id).cloned())
                        .into_iter()
                        .collect()
                }
            };
            if candidates.is_empty() {
                continue;
            }
            let database = font_system.db_mut();
            for mut face in candidates {
                if let Some((_, language)) = face.families.first() {
                    face.families = vec![(font.family.clone(), language.clone())];
                }
                apply_font_face_descriptors(&mut face, font);
                database.push_face_info(face);
            }
            break;
        }
    }
}
