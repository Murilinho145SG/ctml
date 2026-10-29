use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn font_metric_units_resolve_in_line_height_and_text_effect_lengths() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Metric lengths".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "line-height".to_owned(),
                    value: "2ex".to_owned(),
                },
                StyleDeclaration {
                    property: "font-family".to_owned(),
                    value: "serif".to_owned(),
                },
            ],
        },
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "line-height".to_owned(),
                    value: "2ex".to_owned(),
                },
                StyleDeclaration {
                    property: "font-family".to_owned(),
                    value: "serif".to_owned(),
                },
                StyleDeclaration {
                    property: "text-shadow".to_owned(),
                    value: "1ch 0.5ic 0.25cap black".to_owned(),
                },
                StyleDeclaration {
                    property: "text-decoration-thickness".to_owned(),
                    value: "0.5cap".to_owned(),
                },
                StyleDeclaration {
                    property: "text-underline-offset".to_owned(),
                    value: "-0.25ic".to_owned(),
                },
                StyleDeclaration {
                    property: "letter-spacing".to_owned(),
                    value: "-0.5ex".to_owned(),
                },
            ],
        },
    ];
    let page = PageRasterizer::new(&nodes, &stylesheet);
    let style = &page.runs[0].style;
    let font_size = style.font_size;
    let units = style.font_relative_units;
    assert!((style.line_height - font_size * units.ex * 2.0).abs() < 0.01);
    assert!((style.text_shadows[0].offset_x - font_size * units.ch).abs() < 0.01);
    assert!((style.text_shadows[0].offset_y - font_size * units.ic * 0.5).abs() < 0.01);
    assert!((style.text_shadows[0].blur - font_size * units.cap * 0.25).abs() < 0.01);
    assert!((style.decoration_thickness.unwrap() - font_size * units.cap * 0.5).abs() < 0.01);
    assert!((style.underline_offset.unwrap() + font_size * units.ic * 0.25).abs() < 0.01);
    assert!((style.letter_spacing + font_size * units.ex * 0.5).abs() < 0.01);
    assert!(style.letter_spacing_px);
    assert!(
        (page.root_style.line_height
            - page.root_style.font_size * page.root_style.font_relative_units.ex * 2.0)
            .abs()
            < 0.01
    );
}
