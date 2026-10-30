use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};
use base64::{Engine, engine::general_purpose::STANDARD};

#[test]
fn first_line_border_image_replaces_its_solid_border() {
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
        children: vec![UiNode::Text("First formatted line".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-line".to_owned()],
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
    let rendered = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(180, 70);

    assert!(
        rendered
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [0, 0, 255]),
        "the first-line border image should paint around the formatted line"
    );
    assert!(
        !rendered
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 0, 0]),
        "the border image should replace the first-line solid border"
    );
}
