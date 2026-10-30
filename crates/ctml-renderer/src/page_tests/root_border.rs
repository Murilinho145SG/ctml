use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};
use base64::{Engine, engine::general_purpose::STANDARD};

fn blue_png_data_url() -> String {
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, 3, 3);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&[0, 0, 255, 255].repeat(9))
            .unwrap();
    }
    format!(
        "url(\"data:image/png;base64,{}\")",
        STANDARD.encode(encoded)
    )
}

#[test]
fn body_border_paints_on_the_page_root() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Root border".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["body".to_owned()],
        layer: None,
        declarations: [("border", "4px solid red"), ("border-radius", "18px")]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }];
    let rendered = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(120, 80);

    assert!(
        rendered
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 0, 0]),
        "the body border should paint on the root box"
    );
    assert_eq!(
        &rendered[..3],
        [255, 255, 255],
        "root border radius should leave the outer corner unpainted"
    );
}

#[test]
fn body_border_image_paints_on_the_page_root() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Root border image".to_owned())],
    }];
    let source = blue_png_data_url();
    let border_image = format!("{source} 1 / 3 / 0 stretch");
    let stylesheet = [StyleRule {
        selectors: vec!["body".to_owned()],
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
    let rendered = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(120, 80);

    assert!(
        rendered
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [0, 0, 255]),
        "the root border image should paint around the page viewport"
    );
    assert!(
        !rendered
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 0, 0]),
        "the root border image should replace the solid border"
    );
}
