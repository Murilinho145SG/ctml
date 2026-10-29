use super::{ElementIdentity, resolve_style, tests::style_for_declarations};
use crate::{
    StyleDeclaration, StyleRule,
    style::{TextEmphasisPosition, TextEmphasisStyle, TextStyle},
};

#[test]
fn parses_text_emphasis_styles_colors_and_positions() {
    let style = style_for_declarations(&[
        ("color", "blue"),
        ("text-emphasis", "open sesame red"),
        ("text-emphasis-position", "under right"),
    ]);
    assert_eq!(style.text_emphasis_style, TextEmphasisStyle::OpenSesame);
    assert_eq!(style.text_emphasis_color, Some([255, 0, 0, 255]));
    assert_eq!(style.text_emphasis_position, TextEmphasisPosition::Under);

    let shorthand_preserves_position = style_for_declarations(&[
        ("text-emphasis-position", "under"),
        ("text-emphasis", "filled circle"),
    ]);
    assert_eq!(
        shorthand_preserves_position.text_emphasis_position,
        TextEmphasisPosition::Under
    );

    let current_color = style_for_declarations(&[("color", "blue"), ("text-emphasis", "dot")]);
    assert_eq!(
        current_color.text_emphasis_style,
        TextEmphasisStyle::FilledDot
    );
    assert_eq!(current_color.text_emphasis_color, None);

    let custom = style_for_declarations(&[("text-emphasis-style", "'\\2a'")]);
    assert_eq!(
        custom.text_emphasis_style,
        TextEmphasisStyle::Custom("*".to_owned())
    );
}

#[test]
fn emphasis_properties_inherit_and_support_css_wide_keywords() {
    let inherited = TextStyle {
        text_emphasis_style: TextEmphasisStyle::FilledDoubleCircle,
        text_emphasis_color: Some([1, 2, 3, 255]),
        text_emphasis_position: TextEmphasisPosition::Under,
        ..TextStyle::default()
    };
    let current = ElementIdentity {
        tag: "span".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };
    let rule = |property: &str, value: &str| StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    };

    let unset = resolve_style(
        &inherited,
        &current,
        &[],
        &[rule("text-emphasis-style", "unset")],
    );
    assert_eq!(unset.text_emphasis_style, inherited.text_emphasis_style);
    assert_eq!(
        unset.text_emphasis_position,
        inherited.text_emphasis_position
    );
    let initial = resolve_style(
        &inherited,
        &current,
        &[],
        &[rule("text-emphasis", "initial")],
    );
    assert_eq!(initial.text_emphasis_style, TextEmphasisStyle::None);
    assert_eq!(initial.text_emphasis_color, None);
    assert_eq!(initial.text_emphasis_position, TextEmphasisPosition::Under);
}

#[test]
fn malformed_emphasis_declarations_leave_the_previous_values() {
    let inherited = TextStyle {
        text_emphasis_style: TextEmphasisStyle::FilledCircle,
        ..TextStyle::default()
    };
    let current = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };
    let invalid = StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "text-emphasis".to_owned(),
            value: "open filled triangle".to_owned(),
        }],
    };
    assert_eq!(
        resolve_style(&inherited, &current, &[], &[invalid]).text_emphasis_style,
        TextEmphasisStyle::FilledCircle
    );
}
