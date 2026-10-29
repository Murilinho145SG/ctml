use cosmic_text::{FontSystem, fontdb};

use crate::style::{FontVariantCaps, TextStyle};

pub(super) fn supports_caps_features(
    font_system: &FontSystem,
    family: &str,
    style: &TextStyle,
) -> bool {
    let required = match style.font_variant_caps {
        FontVariantCaps::SmallCaps => &[b"smcp" as &[u8; 4]][..],
        FontVariantCaps::AllSmallCaps => &[b"smcp", b"c2sc"][..],
        FontVariantCaps::PetiteCaps => &[b"pcap"][..],
        FontVariantCaps::AllPetiteCaps => &[b"pcap", b"c2pc"][..],
        FontVariantCaps::Normal | FontVariantCaps::Unicase | FontVariantCaps::TitlingCaps => {
            return true;
        }
    };

    let family = match family.to_ascii_lowercase().as_str() {
        "serif" => fontdb::Family::Serif,
        "sans-serif" => fontdb::Family::SansSerif,
        "monospace" => fontdb::Family::Monospace,
        "cursive" => fontdb::Family::Cursive,
        "fantasy" => fontdb::Family::Fantasy,
        _ => fontdb::Family::Name(family),
    };
    let families = [family];
    let query = fontdb::Query {
        families: &families,
        weight: style.weight,
        stretch: style.stretch,
        style: style.font_style,
    };
    let Some(id) = font_system.db().query(&query) else {
        return false;
    };

    font_system
        .db()
        .with_face_data(id, |data, index| {
            required
                .iter()
                .all(|tag| has_opentype_feature(data, index, tag))
        })
        .unwrap_or(false)
}

fn has_opentype_feature(data: &[u8], face_index: u32, tag: &[u8; 4]) -> bool {
    let face_offset = if data.get(..4) == Some(b"ttcf") {
        let Some(count) = read_u32(data, 8) else {
            return false;
        };
        if face_index >= count {
            return false;
        }
        let Some(offset) = read_u32(data, 12 + face_index as usize * 4) else {
            return false;
        };
        offset as usize
    } else if face_index == 0 {
        0
    } else {
        return false;
    };

    let Some(table_count) = read_u16(data, face_offset + 4) else {
        return false;
    };
    let mut gsub = None;
    for index in 0..table_count as usize {
        let record = face_offset + 12 + index * 16;
        if data.get(record..record + 4) == Some(b"GSUB") {
            let Some(offset) = read_u32(data, record + 8) else {
                return false;
            };
            let Some(length) = read_u32(data, record + 12) else {
                return false;
            };
            let start = offset as usize;
            let end = start.saturating_add(length as usize);
            gsub = data.get(start..end);
            break;
        }
    }
    let Some(gsub) = gsub else {
        return false;
    };
    gsub_has_feature(gsub, tag)
}

fn gsub_has_feature(gsub: &[u8], tag: &[u8; 4]) -> bool {
    let Some(feature_offset) = read_u16(gsub, 6).map(usize::from) else {
        return false;
    };
    let Some(feature_count) = read_u16(gsub, feature_offset).map(usize::from) else {
        return false;
    };
    (0..feature_count).any(|index| {
        let record = feature_offset + 2 + index * 6;
        gsub.get(record..record + 4) == Some(tag.as_slice())
    })
}

fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes(
        data.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        data.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn checks_feature_tags_in_the_gsub_feature_list() {
        let mut gsub = vec![0; 24];
        gsub[6..8].copy_from_slice(&10u16.to_be_bytes());
        gsub[10..12].copy_from_slice(&2u16.to_be_bytes());
        gsub[12..16].copy_from_slice(b"smcp");
        gsub[18..22].copy_from_slice(b"c2sc");

        assert!(super::gsub_has_feature(&gsub, b"smcp"));
        assert!(super::gsub_has_feature(&gsub, b"c2sc"));
        assert!(!super::gsub_has_feature(&gsub, b"pcap"));
    }
}
