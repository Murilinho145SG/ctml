use super::*;

#[test]
fn text_decoration_shorthand_resets_inherited_style_and_color() {
    let rule = StyleRule {
        selectors: vec![".sample".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "text-decoration".to_owned(),
            value: "overline".to_owned(),
        }],
    };
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    let paragraph = ElementIdentity {
        tag: "p".to_owned(),
        class: Some("sample".to_owned()),
        id: None,
    };
    let inherited = TextStyle {
        decoration_style: TextDecorationStyle::Wavy,
        decoration_thickness: Some(3.0),
        text_decoration_color: Some([51, 102, 204, 255]),
        ..TextStyle::default()
    };

    let style = resolve_style(&inherited, &paragraph, &[body], &[rule]);

    assert_eq!(style.decoration_style, TextDecorationStyle::Solid);
    assert_eq!(style.decoration_thickness, None);
    assert_eq!(style.text_decoration_color, None);
    assert!(style.overline);
}
