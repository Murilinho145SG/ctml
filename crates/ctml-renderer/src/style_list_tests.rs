use super::compile_runs_with_font_system;
use crate::{StyleDeclaration, StyleRule, UiNode};

fn compile_runs(
    nodes: &[UiNode],
    stylesheet: &[StyleRule],
    hovered_path: &[usize],
    active_path: &[usize],
    focused_path: &[usize],
    keyboard_focus: bool,
) -> (Vec<super::TextRun>, super::TextStyle) {
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
fn list_items_render_bullets_and_ordered_numbers() {
    let nodes = [
        list_container("ul", &["apple", "pear"]),
        list_container("ol", &["first", "second"]),
    ];

    let (runs, _) = compile_runs(&nodes, &[], &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "• apple\n• pear\n1. first\n2. second"
    );
}

#[test]
fn list_style_type_inherits_shapes_and_alphabetic_numbering() {
    let nodes = [
        list_container("ul", &["apple", "pear"]),
        list_container("ol", &["first", "second"]),
    ];
    let stylesheet = [("ul", "square"), ("ol", "upper-alpha")].map(|(selector, value)| StyleRule {
        selectors: vec![selector.to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "list-style-type".to_owned(),
            value: value.to_owned(),
        }],
    });

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "▪ apple\n▪ pear\nA. first\nB. second"
    );
}

#[test]
fn list_style_type_formats_roman_greek_and_leading_zero_markers() {
    for (style_type, count, expected) in [
        ("lower-roman", 4, "i. ii. iii. iv. "),
        ("upper-roman", 4, "I. II. III. IV. "),
        ("lower-greek", 3, "α. β. γ. "),
        ("decimal-leading-zero", 3, "01. 02. 03. "),
    ] {
        let labels = (0..count)
            .map(|index| format!("item{index}"))
            .collect::<Vec<_>>();
        let nodes = [list_container(
            "ol",
            &labels.iter().map(String::as_str).collect::<Vec<_>>(),
        )];
        let stylesheet = [StyleRule {
            selectors: vec!["ol".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "list-style-type".to_owned(),
                value: style_type.to_owned(),
            }],
        }];
        let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);
        let text = runs.iter().map(|run| run.text.as_str()).collect::<String>();
        let expected_text = expected
            .split_whitespace()
            .zip(&labels)
            .map(|(marker, label)| format!("{marker} {label}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(text, expected_text, "unexpected {style_type} markers");
    }
}

#[test]
fn list_style_type_formats_hebrew_armenian_georgian_and_cjk_counters() {
    use super::list_marker::{format_counter_value, format_marker};

    for (style, value, expected) in [
        ("armenian", 2024, "ՍԻԴ"),
        ("lower-armenian", 2024, "սիդ"),
        ("hebrew", 5784, "ה׳תשפד"),
        ("georgian", 2025, "\u{10ea}\u{10d9}\u{10d4}"),
        ("cjk-decimal", 100, "一〇〇"),
        ("cjk-ideographic", 100, "一〇〇"),
        ("cjk-earthly-branch", 12, "亥"),
        ("cjk-heavenly-stem", 10, "癸"),
    ] {
        assert_eq!(
            format_counter_value(style, value).as_deref(),
            Some(expected),
            "{style}"
        );
    }

    assert_eq!(format_counter_value("hebrew", 0).as_deref(), Some("0"));
    assert_eq!(
        format_counter_value("armenian", 10_000).as_deref(),
        Some("10000")
    );
    assert_eq!(
        format_marker(Some("cjk-ideographic"), true, 1).as_deref(),
        Some("一、 ")
    );
}

#[test]
fn list_style_type_formats_localized_decimal_digits() {
    use super::list_marker::format_counter_value;

    for (style, value, expected) in [
        ("arabic-indic", 2048, "٢٠٤٨"),
        ("bengali", 2048, "২০৪৮"),
        ("devanagari", 2048, "२०४८"),
        ("persian", -42, "-۴۲"),
        ("thai", 100, "๑๐๐"),
        ("tibetan", 9, "༩"),
    ] {
        assert_eq!(
            format_counter_value(style, value).as_deref(),
            Some(expected),
            "{style}"
        );
    }
}

#[test]
fn list_style_type_formats_ethiopic_numeric_groups() {
    use super::list_marker::format_counter_value;

    for (value, expected) in [
        (1, "፩"),
        (10, "፲"),
        (100, "፻"),
        (1_000, "፲፻"),
        (78_010_092, "፸፰፻፩፼፺፪"),
    ] {
        assert_eq!(
            format_counter_value("ethiopic-numeric", value).as_deref(),
            Some(expected),
            "{value}"
        );
    }
    assert_eq!(
        format_counter_value("ethiopic-numeric", -12).as_deref(),
        Some("-12")
    );
}

#[test]
fn list_style_type_formats_latin_and_kana_alphabets() {
    use super::list_marker::{format_counter_value, format_marker};

    for (style, value, expected) in [
        ("lower-latin", 27, "aa"),
        ("upper-latin", 28, "AB"),
        ("hiragana", 1, "あ"),
        ("hiragana-iroha", 2, "ろ"),
        ("katakana", 1, "ア"),
        ("katakana-iroha", 2, "ロ"),
    ] {
        assert_eq!(
            format_counter_value(style, value).as_deref(),
            Some(expected),
            "{style}"
        );
    }
    assert_eq!(
        format_marker(Some("hiragana"), true, 1).as_deref(),
        Some("あ、 ")
    );
    assert_eq!(
        format_marker(Some("lower-latin"), true, 27).as_deref(),
        Some("aa. ")
    );
}

#[test]
fn list_style_type_formats_disclosure_markers() {
    use super::list_marker::{format_counter_value, format_marker};

    for (style, expected_symbol, expected_marker) in [
        ("disclosure-open", "▾", "▾ "),
        ("disclosure-closed", "▸", "▸ "),
    ] {
        assert_eq!(
            format_counter_value(style, 1).as_deref(),
            Some(expected_symbol)
        );
        assert_eq!(
            format_marker(Some(style), true, 1).as_deref(),
            Some(expected_marker)
        );

        let (runs, _) = compile_runs(
            &[list_container("ol", &["item"])],
            &[StyleRule {
                selectors: vec!["ol".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: "list-style-type".to_owned(),
                    value: style.to_owned(),
                }],
            }],
            &[],
            &[],
            &[],
            false,
        );
        assert_eq!(
            runs.iter().map(|run| run.text.as_str()).collect::<String>(),
            format!("{expected_marker}item")
        );
    }
}

#[test]
fn list_style_types_accept_hebrew_armenian_and_cjk_names() {
    for style_type in [
        "armenian",
        "upper-armenian",
        "lower-armenian",
        "hebrew",
        "georgian",
        "disclosure-open",
        "disclosure-closed",
        "cjk-decimal",
        "cjk-ideographic",
        "cjk-earthly-branch",
        "cjk-heavenly-stem",
        "arabic-indic",
        "bengali",
        "cambodian",
        "khmer",
        "devanagari",
        "gujarati",
        "gurmukhi",
        "kannada",
        "lao",
        "malayalam",
        "mongolian",
        "myanmar",
        "oriya",
        "persian",
        "tamil",
        "telugu",
        "thai",
        "tibetan",
        "ethiopic-numeric",
        "lower-latin",
        "upper-latin",
        "hiragana",
        "hiragana-iroha",
        "katakana",
        "katakana-iroha",
    ] {
        let stylesheet = [StyleRule {
            selectors: vec!["ol".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "list-style-type".to_owned(),
                value: style_type.to_owned(),
            }],
        }];
        let (runs, _) = compile_runs(
            &[list_container("ol", &["item"])],
            &stylesheet,
            &[],
            &[],
            &[],
            false,
        );
        assert_eq!(
            runs.first()
                .and_then(|run| run.style.list_style_type.as_deref()),
            Some(style_type),
            "{style_type}"
        );
    }
}

#[test]
fn list_style_type_none_suppresses_markers_without_hiding_items() {
    let nodes = [list_container("ul", &["apple", "pear"])];
    let stylesheet = [StyleRule {
        selectors: vec!["ul".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "list-style-type".to_owned(),
            value: "none".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "apple\npear"
    );
}

#[test]
fn list_style_shorthand_and_type_longhand_follow_source_order() {
    let nodes = [list_container("ul", &["apple"])];
    let declarations = [("list-style-type", "square"), ("list-style", "upper-alpha")];
    let stylesheet = declarations.map(|(property, value)| StyleRule {
        selectors: vec!["ul".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    });
    let (shorthand_last, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);
    assert_eq!(
        shorthand_last
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>(),
        "A. apple"
    );

    let stylesheet =
        [("list-style", "upper-alpha"), ("list-style-type", "square")].map(|(property, value)| {
            StyleRule {
                selectors: vec!["ul".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: property.to_owned(),
                    value: value.to_owned(),
                }],
            }
        });
    let (longhand_last, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);
    assert_eq!(
        longhand_last
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>(),
        "▪ apple"
    );
}

fn list_container(tag: &str, labels: &[&str]) -> UiNode {
    UiNode::Element {
        tag: tag.to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: labels
            .iter()
            .map(|label| UiNode::Element {
                tag: "li".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text((*label).to_owned())],
            })
            .collect(),
    }
}

#[test]
fn list_style_type_accepts_custom_string_markers() {
    let nodes = [list_container("ul", &["apple", "pear"])];
    let stylesheet = [StyleRule {
        selectors: vec!["ul".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "list-style-type".to_owned(),
            value: "\"→\"".to_owned(),
        }],
    }];

    let (runs, _) = compile_runs(&nodes, &stylesheet, &[], &[], &[], false);

    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        "→ apple\n→ pear"
    );
}
