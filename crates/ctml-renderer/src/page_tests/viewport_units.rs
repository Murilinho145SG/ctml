use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn viewport_units_recompute_styles_when_the_page_is_resized() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Viewport-relative text".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "font-size".to_owned(),
                    value: "10vw".to_owned(),
                },
                StyleDeclaration {
                    property: "line-height".to_owned(),
                    value: "2svh".to_owned(),
                },
            ],
        },
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "font".to_owned(),
                    value: "10vw/2svh serif".to_owned(),
                },
                StyleDeclaration {
                    property: "text-shadow".to_owned(),
                    value: "1dvw 1lvh 0.5dvmin blue".to_owned(),
                },
                StyleDeclaration {
                    property: "letter-spacing".to_owned(),
                    value: "-0.5svmin".to_owned(),
                },
                StyleDeclaration {
                    property: "text-decoration-thickness".to_owned(),
                    value: "0.5lvh".to_owned(),
                },
            ],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);

    page.snapshot_pixels(800, 600);
    assert_eq!(page.root_style.font_size, 80.0);
    assert_eq!(page.root_style.line_height, 12.0);
    assert_eq!(page.runs[0].style.font_size, 80.0);
    assert_eq!(page.runs[0].style.line_height, 12.0);
    assert_eq!(page.runs[0].style.text_shadows[0].offset_x, 8.0);
    assert_eq!(page.runs[0].style.text_shadows[0].offset_y, 6.0);
    assert_eq!(page.runs[0].style.text_shadows[0].blur, 3.0);
    assert_eq!(page.runs[0].style.letter_spacing, -3.0);
    assert_eq!(page.runs[0].style.decoration_thickness, Some(3.0));

    page.snapshot_pixels(400, 300);
    assert_eq!(page.root_style.font_size, 40.0);
    assert_eq!(page.root_style.line_height, 6.0);
    assert_eq!(page.runs[0].style.font_size, 40.0);
    assert_eq!(page.runs[0].style.line_height, 6.0);
    assert_eq!(page.runs[0].style.text_shadows[0].offset_x, 4.0);
    assert_eq!(page.runs[0].style.text_shadows[0].offset_y, 3.0);
    assert_eq!(page.runs[0].style.text_shadows[0].blur, 1.5);
    assert_eq!(page.runs[0].style.letter_spacing, -1.5);
    assert_eq!(page.runs[0].style.decoration_thickness, Some(1.5));
}
