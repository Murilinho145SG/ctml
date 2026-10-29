use super::{FontVariantCaps, TextRun, TextStyle, append_text};

#[test]
fn synthesizes_small_caps_without_changing_element_line_height() {
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

    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].text, "A");
    assert_eq!(runs[0].style.font_scale, 0.8);
    assert_eq!(runs[0].style.font_size, 12.0);
    assert_eq!(runs[0].style.line_height, 20.0);
    assert_eq!(runs[1].text, "B");
    assert_eq!(runs[1].style.font_scale, 1.0);
}

#[test]
fn synthesizes_all_small_caps_for_lowercase_and_uppercase_letters() {
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
    assert_eq!(runs[0].text, "AB");
    assert_eq!(runs[0].style.font_scale, 0.8);
}
