use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};
use base64::{Engine, engine::general_purpose::STANDARD};

#[test]
fn first_letter_border_image_paints_over_the_solid_border() {
    let pixels = [0, 0, 255, 255].repeat(9);
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, 3, 3);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&pixels)
            .unwrap();
    }
    let source = format!(
        "url(\"data:image/png;base64,{}\")",
        STANDARD.encode(encoded)
    );
    let border_image = format!("{source} 1 / 3 / 0 stretch");
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("First letter".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-letter".to_owned()],
        layer: None,
        declarations: [
            ("border", "3px solid red"),
            ("border-image", border_image.as_str()),
        ]
        .into_iter()
        .map(|(property, value)| StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        })
        .collect(),
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    assert!(
        page.runs
            .iter()
            .find(|run| run.text == "F")
            .is_some_and(|run| {
                run.first_letter_font_style
                    .as_ref()
                    .is_some_and(|style| style.border_image_source.is_some())
            }),
        "the first-letter pseudo style should resolve its border image source"
    );
    let rendered = page.snapshot_pixels(140, 70);

    assert!(
        rendered
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [0, 0, 255]),
        "the border image should paint around the initial letter"
    );
    assert!(
        !rendered
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 0, 0]),
        "the border image should replace the solid border"
    );
}
