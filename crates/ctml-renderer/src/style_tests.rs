use super::{
    FontVariantCaps, TextRun, TextStyle, append_text, compile_runs_with_font_system,
    synthesize_font_variant_caps,
};
use crate::{StyleDeclaration, StyleRule, UiNode};

fn compile_runs(
    nodes: &[UiNode],
    stylesheet: &[StyleRule],
    hovered_path: &[usize],
    active_path: &[usize],
    focused_path: &[usize],
    keyboard_focus: bool,
) -> (Vec<TextRun>, TextStyle) {
    compile_runs_with_font_system(
        nodes,
        stylesheet,
        hovered_path,
        active_path,
        focused_path,
        keyboard_focus,
        None,
        ctml_css::ViewportSize::default(),
    )
}

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
        first_line_decoration: None,
        first_line_font_style: None,
        first_letter_block: Some(3),
        first_letter_foreground: Some([255, 0, 255, 255]),
        first_letter_background: Some([0, 255, 255, 255]),
        first_letter_decoration: None,
        first_letter_font_style: None,
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
            first_line_decoration: None,
            first_line_font_style: None,
            first_letter_block: None,
            first_letter_foreground: None,
            first_letter_background: None,
            first_letter_decoration: None,
            first_letter_font_style: None,
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
    assert_eq!(
        runs.iter()
            .find(|run| run.text == "after")
            .unwrap()
            .style
            .weight,
        cosmic_text::Weight(700)
    );
}

#[test]
fn display_none_hides_element_descendants_and_generated_content() {
    let nodes = [
        UiNode::Element {
            tag: "div".to_owned(),
            class: Some("hidden".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("hidden child".to_owned())],
        },
        UiNode::Element {
            tag: "p".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("visible".to_owned())],
        },
    ];
    let stylesheet = [
        StyleRule {
            selectors: vec![".hidden".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "display".to_owned(),
                value: "none".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".hidden::before".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "content".to_owned(),
                value: "'hidden generated'".to_owned(),
            }],
        },
    ];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "visible"
    );
}

#[test]
fn display_none_on_body_hides_all_rendered_content() {
    let nodes = [UiNode::Text("hidden body".to_owned())];
    let stylesheet = [StyleRule {
        selectors: vec!["body".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "display".to_owned(),
            value: "none".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert!(runs.is_empty());
}

#[test]
fn display_block_and_inline_override_default_text_flow() {
    let nodes = [
        UiNode::Text("before".to_owned()),
        UiNode::Element {
            tag: "span".to_owned(),
            class: Some("block".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("middle".to_owned())],
        },
    ];
    let stylesheet = [StyleRule {
        selectors: vec![".block".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "display".to_owned(),
            value: "block".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "before\nmiddle"
    );
}

#[test]
fn display_inline_overrides_block_tag_line_separation() {
    let nodes = [
        UiNode::Text("before".to_owned()),
        UiNode::Element {
            tag: "p".to_owned(),
            class: Some("inline".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("middle".to_owned())],
        },
        UiNode::Text("after".to_owned()),
    ];
    let stylesheet = [StyleRule {
        selectors: vec![".inline".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "display".to_owned(),
            value: "inline".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "beforemiddleafter"
    );
}

#[test]
fn display_contents_removes_tag_line_separation_but_keeps_children() {
    let nodes = [
        UiNode::Text("before".to_owned()),
        UiNode::Element {
            tag: "div".to_owned(),
            class: Some("contents".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("middle".to_owned())],
        },
        UiNode::Text("after".to_owned()),
    ];
    let stylesheet = [StyleRule {
        selectors: vec![".contents".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "display".to_owned(),
            value: "contents".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "beforemiddleafter"
    );
}

#[test]
fn display_flow_root_uses_block_text_flow() {
    assert_display_flow("span", "flow-root", "before\nmiddle\nafter");
}

#[test]
fn display_inline_block_uses_inline_text_flow() {
    assert_display_flow("p", "inline-block", "beforemiddleafter");
}

#[test]
fn explicit_list_item_uses_a_bullet_and_block_flow() {
    let nodes = [UiNode::Element {
        tag: "span".to_owned(),
        class: Some("item".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("task".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec![".item".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "display".to_owned(),
            value: "list-item".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "• task"
    );
}

fn assert_display_flow(tag: &str, display: &str, expected: &str) {
    let nodes = [
        UiNode::Text("before".to_owned()),
        UiNode::Element {
            tag: tag.to_owned(),
            class: Some("target".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("middle".to_owned())],
        },
        UiNode::Text("after".to_owned()),
    ];
    let stylesheet = [StyleRule {
        selectors: vec![".target".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "display".to_owned(),
            value: display.to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        expected
    );
}
