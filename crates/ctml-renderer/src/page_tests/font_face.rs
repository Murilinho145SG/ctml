use super::font_faces::{parse_font_stretch, register_font_faces};
use crate::{FontFace, FontFaceSource};
use cosmic_text::{FontSystem, fontdb, fontdb::Stretch};

#[test]
fn maps_font_face_stretch_descriptors_to_available_widths() {
    assert_eq!(parse_font_stretch("condensed"), Some(Stretch::Condensed));
    assert_eq!(parse_font_stretch("87.5%"), Some(Stretch::SemiCondensed));
    assert_eq!(parse_font_stretch("112%"), Some(Stretch::SemiExpanded));
    assert_eq!(parse_font_stretch("0%"), None);
    assert_eq!(parse_font_stretch("narrow"), None);
}

#[test]
fn local_font_face_sources_skip_missing_candidates_and_register_aliases() {
    let mut font_system = FontSystem::new();
    let Some(local_family) = font_system
        .db()
        .faces()
        .find_map(|face| face.families.first().map(|family| family.0.clone()))
    else {
        return;
    };
    let alias = "CTML Font Face Test";
    register_font_faces(
        &mut font_system,
        &[FontFace {
            family: alias.to_owned(),
            sources: vec![
                FontFaceSource::Local("missing-font-family-for-test".to_owned()),
                FontFaceSource::Local(local_family),
            ],
            weight: Some(650),
            style: Some("italic".to_owned()),
            stretch: Some("condensed".to_owned()),
        }],
    );

    let families = [fontdb::Family::Name(alias)];
    let face_id = font_system
        .db()
        .query(&fontdb::Query {
            families: &families,
            weight: fontdb::Weight(650),
            stretch: fontdb::Stretch::Condensed,
            style: fontdb::Style::Italic,
        })
        .expect("local candidate should be registered under its CSS family");
    assert_eq!(font_system.db().face(face_id).unwrap().families[0].0, alias);
}
