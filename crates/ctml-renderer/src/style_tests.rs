use super::{FontVariantCaps, TextRun, TextStyle, append_text, synthesize_font_variant_caps};

#[test]
fn preserves_small_caps_for_font_feature_shaping() {
    let style = TextStyle {
        font_size: 12.0,
        line_height: 20.0,
        font_variant_caps: FontVariantCaps::SmallCaps,
        ..TextStyle::default()
    };
    let mut runs = Vec::<TextRun>::new();
    let (mut last_was_space, mut capitalize_next) = (false, false);

    append_text(
        "aB",
        &style,
        &mut runs,
        &mut last_was_space,
        &mut capitalize_next,
    );

    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].text, "aB");
    assert_eq!(runs[0].style.font_scale, 1.0);
    assert_eq!(runs[0].style.font_size, 12.0);
    assert_eq!(runs[0].style.line_height, 20.0);
    assert_eq!(runs[0].style.font_variant_caps, FontVariantCaps::SmallCaps);
}

#[test]
fn preserves_all_small_caps_for_font_feature_shaping() {
    let style = TextStyle {
        font_variant_caps: FontVariantCaps::AllSmallCaps,
        ..TextStyle::default()
    };
    let mut runs = Vec::<TextRun>::new();
    let (mut last_was_space, mut capitalize_next) = (false, false);

    append_text(
        "aB",
        &style,
        &mut runs,
        &mut last_was_space,
        &mut capitalize_next,
    );

    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].text, "aB");
    assert_eq!(runs[0].style.font_scale, 1.0);
    assert_eq!(
        runs[0].style.font_variant_caps,
        FontVariantCaps::AllSmallCaps
    );
}

#[test]
fn synthesizes_small_caps_when_the_font_has_no_feature() {
    let style = TextStyle {
        font_size: 12.0,
        line_height: 20.0,
        font_variant_caps: FontVariantCaps::SmallCaps,
        ..TextStyle::default()
    };
    let runs = synthesize_font_variant_caps(TextRun {
        text: "aB".to_owned(),
        style,
    });

    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].text, "A");
    assert_eq!(runs[0].style.font_scale, 2.0 / 3.0);
    assert_eq!(runs[0].style.font_variant_caps, FontVariantCaps::Normal);
    assert_eq!(runs[0].style.line_height, 20.0);
    assert_eq!(runs[1].text, "B");
    assert_eq!(runs[1].style.font_scale, 1.0);
}

#[test]
fn rounds_synthetic_caps_to_a_pixel_size() {
    for caps in [FontVariantCaps::SmallCaps, FontVariantCaps::PetiteCaps] {
        let style = TextStyle {
            font_size: 8.0,
            font_variant_caps: caps,
            ..TextStyle::default()
        };
        let runs = synthesize_font_variant_caps(TextRun {
            text: "a".to_owned(),
            style,
        });

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "A");
        assert_eq!(runs[0].style.font_size * runs[0].style.font_scale, 6.0);
    }
}
