use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn math_functions_apply_to_text_styles_and_effect_lengths() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Calculated values".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: [
            ("font-size", "calc(1em + 4px)"),
            ("line-height", "calc(1.5)"),
            ("letter-spacing", "calc(1px + 0.25em)"),
            ("font-weight", "calc(400 + 200)"),
            ("opacity", "calc(1 / 2)"),
            ("width", "calc(50% - 10px)"),
            ("text-shadow", "calc(0.5em + 2px) 1px black"),
        ]
        .into_iter()
        .map(|(property, value)| StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        })
        .collect(),
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.snapshot_pixels(180, 80);

    let style = &page.runs[0].style;
    assert_eq!(style.font_size, 20.0);
    assert_eq!(style.line_height, 30.0);
    assert_eq!(style.line_height_multiplier, Some(1.5));
    assert_eq!(style.letter_spacing, 6.0);
    assert_eq!(style.weight.0, 600);
    assert_eq!(style.opacity, 0.5);
    assert_eq!(style.width.as_deref(), Some("calc(50% - 10px)"));
    assert_eq!(style.text_shadows[0].offset_x, 12.0);
    assert_eq!(style.text_shadows[0].offset_y, 1.0);
}
