use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn first_letter_logical_border_maps_to_the_writing_mode_side() {
    for (direction, writing_mode, expected_side) in [
        ("ltr", "horizontal-tb", 3),
        ("rtl", "horizontal-tb", 1),
        ("ltr", "vertical-rl", 0),
    ] {
        let nodes = [UiNode::Element {
            tag: "p".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("First letter".to_owned())],
        }];
        let stylesheet = [
            StyleRule {
                selectors: vec!["p".to_owned()],
                layer: None,
                declarations: [("direction", direction), ("writing-mode", writing_mode)]
                    .into_iter()
                    .map(|(property, value)| StyleDeclaration {
                        property: property.to_owned(),
                        value: value.to_owned(),
                    })
                    .collect(),
            },
            StyleRule {
                selectors: vec!["p::first-letter".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: "border-inline-start".to_owned(),
                    value: "2px solid red".to_owned(),
                }],
            },
        ];
        let mut page = PageRasterizer::new(&nodes, &stylesheet);
        let initial = page.runs.iter().find(|run| run.text == "F").unwrap();

        assert_eq!(
            initial.style.border_width[expected_side].as_deref(),
            Some("2px"),
            "inline-start border should resolve for {direction} {writing_mode}"
        );
        assert_eq!(
            initial.style.border_style[expected_side],
            crate::style::BorderLineStyle::Solid
        );
        assert_eq!(
            initial.style.border_color[expected_side],
            Some([255, 0, 0, 255])
        );
        assert!(
            page.snapshot_pixels(120, 70)
                .chunks_exact(4)
                .any(|pixel| pixel[..3] == [255, 0, 0]),
            "logical border should paint around the initial glyph for {direction} {writing_mode}"
        );
    }
}
