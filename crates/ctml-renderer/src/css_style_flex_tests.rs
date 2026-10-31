use crate::{StyleDeclaration, StyleRule, style::TextStyle};

use super::{ElementIdentity, resolve_style};

fn style_with_order(value: &str) -> i32 {
    resolve_style(
        &TextStyle::default(),
        &ElementIdentity {
            tag: "p".to_owned(),
            class: None,
            id: None,
        },
        &[],
        &[rule("order", value)],
    )
    .flex_order
}

fn rule(property: &str, value: &str) -> StyleRule {
    StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    }
}

fn style_with_declarations(declarations: &[(&str, &str)]) -> TextStyle {
    let rule = StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: declarations
            .iter()
            .map(|(property, value)| StyleDeclaration {
                property: (*property).to_owned(),
                value: (*value).to_owned(),
            })
            .collect(),
    };
    resolve_style(
        &TextStyle::default(),
        &ElementIdentity {
            tag: "p".to_owned(),
            class: None,
            id: None,
        },
        &[],
        &[rule],
    )
}

#[test]
fn order_parses_integers_and_clamps_values_outside_the_style_range() {
    assert_eq!(style_with_order("-3"), -3);
    assert_eq!(style_with_order("2"), 2);
    assert_eq!(style_with_order("1.5"), 0);
    assert_eq!(style_with_order("999999999999999999999"), i32::MAX);
    assert_eq!(style_with_order("-999999999999999999999"), i32::MIN);
}

#[test]
fn flex_direction_supports_row_reverse_and_css_wide_resets() {
    let row = style_with_declarations(&[("flex-direction", "row")]);
    assert!(!row.flex_direction_column);
    assert!(!row.flex_direction_reverse);

    let reversed = style_with_declarations(&[("flex-direction", "row-reverse")]);
    assert!(!reversed.flex_direction_column);
    assert!(reversed.flex_direction_reverse);

    let column = style_with_declarations(&[("flex-direction", "column")]);
    assert!(column.flex_direction_column);
    assert!(!column.flex_direction_reverse);

    let reversed_column = style_with_declarations(&[("flex-direction", "column-reverse")]);
    assert!(reversed_column.flex_direction_column);
    assert!(reversed_column.flex_direction_reverse);

    let inherited = TextStyle {
        flex_direction_reverse: true,
        ..TextStyle::default()
    };
    let element = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };
    assert!(
        resolve_style(
            &inherited,
            &element,
            &[],
            &[rule("flex-direction", "inherit")]
        )
        .flex_direction_reverse
    );
    for keyword in ["initial", "unset", "revert"] {
        assert!(
            !resolve_style(
                &inherited,
                &element,
                &[],
                &[rule("flex-direction", keyword)]
            )
            .flex_direction_reverse,
            "{keyword} should restore row direction"
        );
    }
}

#[test]
fn order_is_non_inherited_and_supports_css_wide_resets() {
    let inherited = TextStyle {
        flex_order: 7,
        ..TextStyle::default()
    };
    let element = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };

    assert_eq!(resolve_style(&inherited, &element, &[], &[]).flex_order, 0);
    assert_eq!(
        resolve_style(&inherited, &element, &[], &[rule("order", "inherit")]).flex_order,
        7
    );
    for keyword in ["initial", "unset"] {
        assert_eq!(
            resolve_style(&inherited, &element, &[], &[rule("order", keyword)]).flex_order,
            0
        );
    }
    assert_eq!(
        resolve_style(
            &inherited,
            &element,
            &[],
            &[rule("order", "3"), rule("all", "initial")]
        )
        .flex_order,
        0
    );
}

#[test]
fn justify_content_parses_supported_alignment_and_css_wide_values() {
    use crate::style::FlexAlignment::{Center, FlexEnd, FlexStart};

    assert_eq!(style_with_declarations(&[]).justify_content, FlexStart);
    assert_eq!(
        style_with_declarations(&[("justify-content", "center")]).justify_content,
        Center
    );
    assert_eq!(
        style_with_declarations(&[("justify-content", "flex-end")]).justify_content,
        FlexEnd
    );
    assert_eq!(
        style_with_declarations(&[("justify-content", "start")]).justify_content,
        crate::style::FlexAlignment::Start
    );
    assert_eq!(
        style_with_declarations(&[("justify-content", "end")]).justify_content,
        crate::style::FlexAlignment::End
    );
    assert_eq!(
        style_with_declarations(&[("justify-content", "space-between")]).justify_content,
        crate::style::FlexAlignment::SpaceBetween
    );
    assert_eq!(
        style_with_declarations(&[("justify-content", "space-around")]).justify_content,
        crate::style::FlexAlignment::SpaceAround
    );
    assert_eq!(
        style_with_declarations(&[("justify-content", "space-evenly")]).justify_content,
        crate::style::FlexAlignment::SpaceEvenly
    );

    let inherited = TextStyle {
        justify_content: Center,
        ..TextStyle::default()
    };
    let element = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };
    assert_eq!(
        resolve_style(&inherited, &element, &[], &[]).justify_content,
        FlexStart
    );
    assert_eq!(
        resolve_style(
            &inherited,
            &element,
            &[],
            &[rule("justify-content", "inherit")]
        )
        .justify_content,
        Center
    );
    assert_eq!(
        resolve_style(
            &inherited,
            &element,
            &[],
            &[rule("justify-content", "initial")]
        )
        .justify_content,
        FlexStart
    );
    assert_eq!(
        style_with_declarations(&[("justify-content", "center"), ("all", "initial")])
            .justify_content,
        FlexStart
    );
}

#[test]
fn align_items_parses_supported_alignment_and_css_wide_values() {
    use crate::style::FlexAlignment::{Center, FlexEnd, FlexStart};

    assert_eq!(style_with_declarations(&[]).align_items, None);
    assert_eq!(
        style_with_declarations(&[("align-items", "center")]).align_items,
        Some(Center)
    );
    assert_eq!(
        style_with_declarations(&[("align-items", "flex-end")]).align_items,
        Some(FlexEnd)
    );
    assert_eq!(
        style_with_declarations(&[("align-items", "stretch")]).align_items,
        None
    );
    assert_eq!(
        style_with_declarations(&[("align-items", "flex-start")]).align_items,
        Some(FlexStart)
    );
    assert_eq!(
        style_with_declarations(&[("align-items", "baseline")]).align_items,
        None
    );

    let inherited = TextStyle {
        align_items: Some(Center),
        ..TextStyle::default()
    };
    let element = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };
    assert_eq!(
        resolve_style(&inherited, &element, &[], &[]).align_items,
        None
    );
    assert_eq!(
        resolve_style(&inherited, &element, &[], &[rule("align-items", "inherit")]).align_items,
        Some(Center)
    );
    assert_eq!(
        resolve_style(&inherited, &element, &[], &[rule("align-items", "initial")]).align_items,
        None
    );
    assert_eq!(
        style_with_declarations(&[("align-items", "center"), ("all", "initial")]).align_items,
        None
    );
}

#[test]
fn align_self_parses_supported_overrides_and_css_wide_values() {
    use crate::style::FlexAlignment::{Center, FlexEnd, FlexStart};

    assert_eq!(style_with_declarations(&[]).align_self, None);
    assert_eq!(
        style_with_declarations(&[("align-self", "auto")]).align_self,
        None
    );
    assert_eq!(
        style_with_declarations(&[("align-self", "flex-start")]).align_self,
        Some(FlexStart)
    );
    assert_eq!(
        style_with_declarations(&[("align-self", "center")]).align_self,
        Some(Center)
    );
    assert_eq!(
        style_with_declarations(&[("align-self", "flex-end")]).align_self,
        Some(FlexEnd)
    );
    assert_eq!(
        style_with_declarations(&[("align-self", "center"), ("align-self", "baseline")]).align_self,
        Some(Center)
    );
    assert_eq!(
        style_with_declarations(&[("align-self", "center"), ("all", "initial")]).align_self,
        None
    );

    let inherited = TextStyle {
        align_self: Some(Center),
        ..TextStyle::default()
    };
    let element = ElementIdentity {
        tag: "p".to_owned(),
        class: None,
        id: None,
    };
    assert_eq!(
        resolve_style(&inherited, &element, &[], &[]).align_self,
        None
    );
    assert_eq!(
        resolve_style(&inherited, &element, &[], &[rule("align-self", "inherit")]).align_self,
        Some(Center)
    );
    assert_eq!(
        resolve_style(&inherited, &element, &[], &[rule("align-self", "initial")]).align_self,
        None
    );
}

#[test]
fn flex_gaps_parse_shorthand_longhands_and_source_order() {
    let single = style_with_declarations(&[("gap", "4px")]);
    assert_eq!(single.row_gap.as_deref(), Some("4px"));
    assert_eq!(single.column_gap.as_deref(), Some("4px"));

    let pair = style_with_declarations(&[("gap", "2px 8px")]);
    assert_eq!(pair.row_gap.as_deref(), Some("2px"));
    assert_eq!(pair.column_gap.as_deref(), Some("8px"));

    let math = style_with_declarations(&[("gap", "calc(1px + 2px)")]);
    assert_eq!(math.row_gap.as_deref(), Some("calc(1px + 2px)"));
    let percentage_math = style_with_declarations(&[("gap", "calc(10% + 2px)")]);
    assert_eq!(percentage_math.row_gap.as_deref(), Some("calc(10% + 2px)"));

    let percentage = style_with_declarations(&[("gap", "10% 5%")]);
    assert_eq!(percentage.row_gap.as_deref(), Some("10%"));
    assert_eq!(percentage.column_gap.as_deref(), Some("5%"));

    let ordered = style_with_declarations(&[
        ("gap", "3px 6px"),
        ("row-gap", "5px"),
        ("column-gap", "9px"),
    ]);
    assert_eq!(ordered.row_gap.as_deref(), Some("5px"));
    assert_eq!(ordered.column_gap.as_deref(), Some("9px"));

    let later_shorthand = style_with_declarations(&[("column-gap", "9px"), ("gap", "3px 6px")]);
    assert_eq!(later_shorthand.column_gap.as_deref(), Some("6px"));

    let invalid = style_with_declarations(&[("gap", "-2px")]);
    assert_eq!(invalid.row_gap, None);
    let negative_percentage = style_with_declarations(&[("gap", "-2%")]);
    assert_eq!(negative_percentage.row_gap, None);
    assert_eq!(
        style_with_declarations(&[("gap", "normal")])
            .column_gap
            .as_deref(),
        Some("0")
    );

    let reset = style_with_declarations(&[("gap", "3px"), ("all", "initial")]);
    assert_eq!(reset.row_gap.as_deref(), Some("0"));
    assert_eq!(reset.column_gap.as_deref(), Some("0"));
}
