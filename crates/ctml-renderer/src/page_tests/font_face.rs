use super::parse_font_stretch;
use cosmic_text::fontdb::Stretch;

#[test]
fn maps_font_face_stretch_descriptors_to_available_widths() {
    assert_eq!(parse_font_stretch("condensed"), Some(Stretch::Condensed));
    assert_eq!(parse_font_stretch("87.5%"), Some(Stretch::SemiCondensed));
    assert_eq!(parse_font_stretch("112%"), Some(Stretch::SemiExpanded));
    assert_eq!(parse_font_stretch("0%"), None);
    assert_eq!(parse_font_stretch("narrow"), None);
}
