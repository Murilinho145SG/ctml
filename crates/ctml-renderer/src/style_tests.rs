use super::{
    FontVariantCaps, TextRun, TextStyle, append_text, compile_runs, synthesize_font_variant_caps,
};
use crate::{StyleDeclaration, StyleRule, UiNode};

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
        &[],
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
        &[],
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
        element_path: Vec::new(),
        selection_background: Some([255, 0, 0, 255]),
        selection_foreground: Some([0, 255, 0, 255]),
        first_line_block: Some(3),
        first_line_foreground: Some([0, 0, 255, 255]),
        first_line_background: Some([255, 255, 0, 255]),
        first_letter_block: Some(3),
        first_letter_foreground: Some([255, 0, 255, 255]),
        first_letter_background: Some([0, 255, 255, 255]),
    });

    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].text, "A");
    assert_eq!(runs[0].style.font_scale, 2.0 / 3.0);
    assert_eq!(runs[0].style.font_variant_caps, FontVariantCaps::Normal);
    assert_eq!(runs[0].style.line_height, 20.0);
    assert_eq!(runs[0].selection_background, Some([255, 0, 0, 255]));
    assert_eq!(runs[0].selection_foreground, Some([0, 255, 0, 255]));
    assert_eq!(runs[0].first_line_block, Some(3));
    assert_eq!(runs[0].first_line_foreground, Some([0, 0, 255, 255]));
    assert_eq!(runs[0].first_line_background, Some([255, 255, 0, 255]));
    assert_eq!(runs[0].first_letter_block, Some(3));
    assert_eq!(runs[0].first_letter_foreground, Some([255, 0, 255, 255]));
    assert_eq!(runs[0].first_letter_background, Some([0, 255, 255, 255]));
    assert_eq!(runs[1].text, "B");
    assert_eq!(runs[1].style.font_scale, 1.0);
    assert_eq!(runs[1].selection_background, Some([255, 0, 0, 255]));
    assert_eq!(runs[1].selection_foreground, Some([0, 255, 0, 255]));
    assert_eq!(runs[1].first_line_block, Some(3));
    assert_eq!(runs[1].first_line_foreground, Some([0, 0, 255, 255]));
    assert_eq!(runs[1].first_line_background, Some([255, 255, 0, 255]));
    assert_eq!(runs[1].first_letter_block, Some(3));
    assert_eq!(runs[1].first_letter_foreground, Some([255, 0, 255, 255]));
    assert_eq!(runs[1].first_letter_background, Some([0, 255, 255, 255]));
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
            element_path: Vec::new(),
            selection_background: None,
            selection_foreground: None,
            first_line_block: None,
            first_line_foreground: None,
            first_line_background: None,
            first_letter_block: None,
            first_letter_foreground: None,
            first_letter_background: None,
        });

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "A");
        assert_eq!(runs[0].style.font_size * runs[0].style.font_scale, 6.0);
    }
}

#[test]
fn generates_before_and_after_text_with_pseudo_element_styles() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("body".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p::before".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "content".to_owned(),
                    value: "\"before\"".to_owned(),
                },
                StyleDeclaration {
                    property: "color".to_owned(),
                    value: "blue".to_owned(),
                },
            ],
        },
        StyleRule {
            selectors: vec!["p:after".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "content".to_owned(),
                    value: "'after'".to_owned(),
                },
                StyleDeclaration {
                    property: "font-weight".to_owned(),
                    value: "700".to_owned(),
                },
            ],
        },
    ];
    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "beforebodyafter"
    );
    assert_eq!(runs[0].style.color, [0, 0, 255, 255]);
    assert_eq!(runs[2].style.weight, cosmic_text::Weight(700));
}
