use super::super::{PAGE_PADDING, PageRasterizer};
use crate::{StyleDeclaration, StyleRule, UiNode};
use base64::{Engine, engine::general_purpose::STANDARD};

fn green_pixel_data_url() -> String {
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, 1, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&[0, 255, 0, 255]).unwrap();
    }
    format!(
        "url(\"data:image/png;base64,{}\")",
        STANDARD.encode(encoded)
    )
}

fn node() -> [UiNode; 1] {
    [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("box".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: Vec::new(),
    }]
}

fn rule(image: &str, extra: &[(&str, &str)]) -> StyleRule {
    let mut declarations = vec![
        ("width", "20px"),
        ("height", "20px"),
        ("padding", "4px"),
        ("border", "2px solid black"),
        ("background-image", image),
    ];
    declarations.extend_from_slice(extra);
    StyleRule {
        selectors: vec![".box".to_owned()],
        layer: None,
        declarations: declarations
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }
}

fn green_pixels(pixels: &[u8], width: usize) -> Vec<(usize, usize)> {
    pixels
        .chunks_exact(4)
        .enumerate()
        .filter_map(|(index, pixel)| {
            (pixel[0] == 0 && pixel[1] == 255 && pixel[2] == 0)
                .then_some((index % width, index / width))
        })
        .collect()
}

#[test]
fn background_origin_positions_an_image_from_the_content_box() {
    let nodes = node();
    let image = green_pixel_data_url();
    let mut page = PageRasterizer::new(
        &nodes,
        &[rule(
            &image,
            &[
                ("background-size", "4px 4px"),
                ("background-repeat", "no-repeat"),
                ("background-position", "right bottom"),
                ("background-origin", "content-box"),
            ],
        )],
    );
    page.prepare_layout(100, 80);
    let geometry = page.overflow_geometry[&1];
    let pixels = page.snapshot_pixels(100, 80);
    let green = green_pixels(&pixels, 100);
    let max_x = green.iter().map(|(x, _)| *x).max().unwrap();
    let max_y = green.iter().map(|(_, y)| *y).max().unwrap();
    let expected_x = (geometry.content_box.right - PAGE_PADDING).ceil() as usize - 1;
    let expected_y = (geometry.content_box.bottom - PAGE_PADDING).ceil() as usize - 1;

    assert_eq!(green.len(), 16);
    assert!(
        max_x.abs_diff(expected_x) <= 1,
        "x: {max_x} vs {expected_x}"
    );
    assert!(
        max_y.abs_diff(expected_y) <= 1,
        "y: {max_y} vs {expected_y}"
    );
}

#[test]
fn background_clip_limits_image_painting_to_the_content_box() {
    let nodes = node();
    let image = green_pixel_data_url();
    let mut page = PageRasterizer::new(
        &nodes,
        &[rule(&image, &[("background-clip", "content-box")])],
    );
    page.prepare_layout(100, 80);
    let geometry = page.overflow_geometry[&1];
    let pixels = page.snapshot_pixels(100, 80);
    let green = green_pixels(&pixels, 100);
    let left = (geometry.content_box.left - PAGE_PADDING).floor() as usize;
    let top = (geometry.content_box.top - PAGE_PADDING).floor() as usize;
    let right = (geometry.content_box.right - PAGE_PADDING).ceil() as usize;
    let bottom = (geometry.content_box.bottom - PAGE_PADDING).ceil() as usize;
    assert!(green.len() > (right - left) * (bottom - top) * 3 / 4);
    assert!(
        green
            .iter()
            .all(|(x, y)| *x >= left && *x < right && *y >= top && *y < bottom)
    );
}

#[test]
fn background_clip_limits_the_background_color_to_the_content_box() {
    let nodes = node();
    let mut page = PageRasterizer::new(
        &nodes,
        &[rule(
            "none",
            &[
                ("background-color", "#00ff00"),
                ("background-clip", "border-box, content-box"),
            ],
        )],
    );
    page.prepare_layout(100, 80);
    let geometry = page.overflow_geometry[&1];
    let pixels = page.snapshot_pixels(100, 80);
    let green = green_pixels(&pixels, 100);
    let left = geometry.content_box.left.floor() as usize;
    let top = geometry.content_box.top.floor() as usize;
    let right = geometry.content_box.right.ceil() as usize;
    let bottom = geometry.content_box.bottom.ceil() as usize;
    assert_eq!(green.len(), (right - left) * (bottom - top));
    assert!(
        green
            .iter()
            .all(|(x, y)| *x >= left && *x < right && *y >= top && *y < bottom)
    );
}
