use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};
use base64::{Engine, engine::general_purpose::STANDARD};

fn border_image_url() -> String {
    let colors = [
        [255, 0, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        [255, 255, 0, 255],
        [255, 255, 255, 255],
        [255, 0, 255, 255],
        [0, 255, 255, 255],
        [128, 128, 128, 255],
        [0, 0, 0, 255],
    ]
    .concat();
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, 3, 3);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&colors)
            .unwrap();
    }
    format!(
        "url(\"data:image/png;base64,{}\")",
        STANDARD.encode(encoded)
    )
}

#[test]
fn border_image_slice_paints_nine_source_regions_and_optional_center() {
    let (rect, pixels) = render("1 fill");
    let sample = |x: f32, y: f32| sample_at(&pixels, rect.left + x, rect.top + y);

    assert_eq!(sample(1.0, 1.0), [255, 0, 0]);
    assert_eq!(sample(20.0, 1.0), [0, 255, 0]);
    assert_eq!(sample(46.0, 1.0), [0, 0, 255]);
    assert_eq!(sample(1.0, 16.0), [255, 255, 0]);
    assert_eq!(sample(20.0, 16.0), [255, 255, 255]);
    assert_eq!(sample(46.0, 20.0), [255, 0, 255]);
    assert_eq!(sample(24.0, 38.0), [128, 128, 128]);
}

#[test]
fn border_image_slice_accepts_percentages() {
    let (rect, pixels) = render("33.333% fill");
    let top_left = sample_at(&pixels, rect.left + 1.0, rect.top + 1.0);
    let center = sample_at(&pixels, rect.left + 20.0, rect.top + 16.0);
    assert_eq!(top_left, [255, 0, 0]);
    assert_eq!(center, [255, 255, 255]);
}

#[test]
fn border_image_slice_leaves_center_unpainted_without_fill() {
    let (rect, pixels) = render("1");
    let center = sample_at(&pixels, rect.left + 20.0, rect.top + 16.0);
    assert_eq!(center, [0, 255, 255]);
}

fn render(slice: &str) -> (super::super::overflow_clip::ClipRect, Vec<u8>) {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("image-border".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: Vec::new(),
    }];
    let source = border_image_url();
    let stylesheet = [StyleRule {
        selectors: vec![".image-border".to_owned()],
        layer: None,
        declarations: [
            ("width", "40px"),
            ("height", "32px"),
            ("border-width", "4px"),
            ("background-color", "cyan"),
            ("border-image-source", source.as_str()),
            ("border-image-slice", slice),
        ]
        .into_iter()
        .map(|(property, value)| StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        })
        .collect(),
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(100, 70);
    let rect = page.overflow_geometry[&1].border_box;
    let pixels = page.snapshot_pixels(100, 70);
    (rect, pixels)
}

fn sample_at(pixels: &[u8], x: f32, y: f32) -> [u8; 3] {
    let x = x.round() as usize;
    let y = y.round() as usize;
    let pixel = &pixels[(y * 100 + x) * 4..][..3];
    [pixel[0], pixel[1], pixel[2]]
}
