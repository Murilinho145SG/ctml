use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn pointer_hover_and_active_states_restyle_text_runs() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Hover".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p:hover".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "color".to_owned(),
                value: "red".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p:active".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-weight".to_owned(),
                value: "700".to_owned(),
            }],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let normal = page.snapshot_pixels(100, 50);

    assert!(page.update_hover(10.0, 14.0));
    let hovered = page.snapshot_pixels(100, 50);
    assert_ne!(normal, hovered);
    assert!(!page.update_hover(10.0, 14.0));

    assert!(page.begin_selection(10.0, 14.0));
    let active = page.snapshot_pixels(100, 50);
    assert_ne!(hovered, active);
    assert!(page.end_active());
    assert_eq!(page.snapshot_pixels(100, 50), hovered);
}

#[test]
fn direction_rtl_sets_the_paragraph_base_bidi_level() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("abc אבג 123".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "direction".to_owned(),
            value: "rtl".to_owned(),
        }],
    }];
    let left_to_right = PageRasterizer::new(&nodes, &[]).snapshot_pixels(240, 50);
    let right_to_left = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(240, 50);

    assert_ne!(left_to_right, right_to_left);
}

#[test]
fn unicode_bidi_isolate_applies_to_nested_inline_runs_without_rendered_controls() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![
            UiNode::Text("אב ".to_owned()),
            UiNode::Element {
                tag: "span".to_owned(),
                class: Some("isolated".to_owned()),
                id: None,
                attributes: Vec::new(),
                children: vec![
                    UiNode::Text("(".to_owned()),
                    UiNode::Element {
                        tag: "b".to_owned(),
                        class: None,
                        id: None,
                        attributes: Vec::new(),
                        children: vec![UiNode::Text("abc".to_owned())],
                    },
                    UiNode::Text(")".to_owned()),
                ],
            },
            UiNode::Text(" גד".to_owned()),
        ],
    }];
    let base = vec![
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "direction".to_owned(),
                value: "rtl".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".isolated".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "direction".to_owned(),
                value: "ltr".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["b".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-weight".to_owned(),
                value: "700".to_owned(),
            }],
        },
    ];
    let isolated = [
        base[0].clone(),
        base[1].clone(),
        base[2].clone(),
        StyleRule {
            selectors: vec![".isolated".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "unicode-bidi".to_owned(),
                value: "isolate".to_owned(),
            }],
        },
    ];
    let mut isolated_page = PageRasterizer::new(&nodes, &isolated);
    isolated_page.snapshot_pixels(240, 60);

    assert!(
        isolated_page
            .runs
            .iter()
            .any(|run| { run.style.bidi_isolate_direction == Some(crate::style::Direction::Ltr) })
    );
    assert!(isolated_page.runs.iter().all(|run| {
        !run.text
            .chars()
            .any(|character| matches!(character, '\u{2066}' | '\u{2067}' | '\u{2069}'))
    }));
}

#[test]
fn focus_and_focus_within_restyle_clicked_links() {
    let nodes = [UiNode::Element {
        tag: "section".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "a".to_owned(),
            class: None,
            id: None,
            attributes: vec![("href".to_owned(), Some("/next".to_owned()))],
            children: vec![UiNode::Text("Focus".to_owned())],
        }],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["a:focus".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "color".to_owned(),
                value: "red".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["section:focus-within".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-weight".to_owned(),
                value: "700".to_owned(),
            }],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let normal = page.snapshot_pixels(100, 50);

    assert!(page.begin_selection(10.0, 14.0));
    assert_ne!(normal, page.snapshot_pixels(100, 50));

    assert!(page.begin_selection(90.0, 45.0));
    assert_eq!(normal, page.snapshot_pixels(100, 50));
}

#[test]
fn keyboard_focus_activates_focus_visible_and_pointer_focus_does_not() {
    let nodes = [UiNode::Element {
        tag: "a".to_owned(),
        class: None,
        id: None,
        attributes: vec![("href".to_owned(), Some("/next".to_owned()))],
        children: vec![UiNode::Text("Focus".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["a:focus".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "color".to_owned(),
                value: "blue".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["a:focus-visible".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "color".to_owned(),
                value: "red".to_owned(),
            }],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);

    assert!(page.focus_next(false));
    let keyboard = page.snapshot_pixels(100, 50);
    assert!(keyboard.chunks_exact(4).any(|pixel| pixel[0] > pixel[2]));

    assert!(page.begin_selection(10.0, 14.0));
    let pointer = page.snapshot_pixels(100, 50);
    assert!(pointer.chunks_exact(4).any(|pixel| pixel[2] > pixel[0]));
    assert!(!pointer.chunks_exact(4).any(|pixel| pixel[0] > pixel[2]));
}

#[test]
fn numeric_tabindex_makes_generic_elements_focusable() {
    let nodes = [UiNode::Element {
        tag: "span".to_owned(),
        class: None,
        id: None,
        attributes: vec![("tabindex".to_owned(), Some("0".to_owned()))],
        children: vec![UiNode::Text("Focusable".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["span:focus".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "color".to_owned(),
            value: "red".to_owned(),
        }],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let normal = page.snapshot_pixels(120, 50);

    assert!(page.begin_selection(10.0, 14.0));
    assert_ne!(normal, page.snapshot_pixels(120, 50));
}

#[test]
fn text_shadow_is_rasterized_behind_text() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Shadow".to_owned())],
    }];
    let baseline = PageRasterizer::new(&nodes, &[]).snapshot_pixels(120, 50);
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "text-shadow".to_owned(),
            value: "4px 3px 2px red".to_owned(),
        }],
    }];
    let shadow = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(120, 50);

    assert_ne!(baseline, shadow);
    assert!(
        shadow
            .chunks_exact(4)
            .any(|pixel| u16::from(pixel[0]) > u16::from(pixel[1]) + 32)
    );
}

#[test]
fn text_emphasis_draws_marks_on_non_whitespace_glyphs() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("A B".to_owned())],
    }];
    let baseline = PageRasterizer::new(&nodes, &[]).snapshot_pixels(100, 50);
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "text-emphasis".to_owned(),
            value: "filled dot red".to_owned(),
        }],
    }];
    let emphasized = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(100, 50);

    assert_ne!(baseline, emphasized);
    let red_rows = |pixels: &[u8]| {
        pixels
            .chunks_exact(100 * 4)
            .enumerate()
            .filter(|(_, row)| {
                row.chunks_exact(4).any(|pixel| {
                    u16::from(pixel[0]) > u16::from(pixel[1]) + 32
                        && u16::from(pixel[0]) > u16::from(pixel[2]) + 32
                })
            })
            .map(|(row, _)| row)
            .collect::<Vec<_>>()
    };
    let over_rows = red_rows(&emphasized);
    assert!(!over_rows.is_empty());

    let under_stylesheet = [
        stylesheet[0].clone(),
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "text-emphasis-position".to_owned(),
                value: "under".to_owned(),
            }],
        },
    ];
    let under = PageRasterizer::new(&nodes, &under_stylesheet).snapshot_pixels(100, 50);
    let under_rows = red_rows(&under);
    assert!(!under_rows.is_empty());
    assert!(
        over_rows.iter().max().unwrap() < under_rows.iter().min().unwrap(),
        "over marks should appear above under marks"
    );
}

#[test]
fn text_emphasis_renders_custom_string_marks() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("A".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "text-emphasis".to_owned(),
            value: "'*' red".to_owned(),
        }],
    }];
    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(100, 50);
    assert!(pixels.chunks_exact(4).any(|pixel| {
        u16::from(pixel[0]) > u16::from(pixel[1]) + 32
            && u16::from(pixel[0]) > u16::from(pixel[2]) + 32
    }));
}
