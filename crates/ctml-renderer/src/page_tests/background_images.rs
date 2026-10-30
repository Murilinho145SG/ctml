use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

use base64::{Engine, engine::general_purpose::STANDARD};

fn png_data_url(color: [u8; 4]) -> String {
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, 1, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&color).unwrap();
    }
    format!(
        "url(\"data:image/png;base64,{}\")",
        STANDARD.encode(encoded)
    )
}

#[test]
fn paints_repeating_data_image_backgrounds_in_layer_order() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("image".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: Vec::new(),
    }];
    let images = format!(
        "{}, {}",
        png_data_url([0, 255, 0, 255]),
        png_data_url([0, 0, 255, 255])
    );
    let stylesheet = [StyleRule {
        selectors: vec![".image".to_owned()],
        layer: None,
        declarations: [
            ("width", "30px"),
            ("height", "20px"),
            ("background-image", images.as_str()),
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
    let x = (rect.left + 2.0).round() as usize;
    let y = (rect.top + 2.0).round() as usize;
    let pixel = &pixels[(y * 100 + x) * 4..][..3];

    assert_eq!(
        pixel,
        [0, 255, 0],
        "the first image layer should paint on top"
    );
}

#[test]
fn paints_linear_gradient_backgrounds_in_the_declared_direction() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("gradient".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: Vec::new(),
    }];
    let stylesheet = [StyleRule {
        selectors: vec![".gradient".to_owned()],
        layer: None,
        declarations: [
            ("width", "80px"),
            ("height", "40px"),
            ("background", "linear-gradient(to right, red, blue)"),
        ]
        .into_iter()
        .map(|(property, value)| StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        })
        .collect(),
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(120, 80);
    let rect = page.overflow_geometry[&1].border_box;
    let pixels = page.snapshot_pixels(120, 80);
    let y = (rect.top + (rect.bottom - rect.top) / 2.0).round() as usize;
    let left_x = (rect.left + 2.0).round() as usize;
    let right_x = (rect.right - 10.0).round() as usize;
    let left = &pixels[(y * 120 + left_x) * 4..][..3];
    let right = &pixels[(y * 120 + right_x) * 4..][..3];

    assert!(
        left[0] > 220 && left[2] < 40,
        "gradient start should be red: {left:?}"
    );
    assert!(
        right[2] > 220 && right[0] < 40,
        "gradient end should be blue: {right:?}"
    );
}

#[test]
fn paints_linear_gradients_with_explicit_color_stop_positions() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("gradient".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: Vec::new(),
    }];
    let stylesheet = [StyleRule {
        selectors: vec![".gradient".to_owned()],
        layer: None,
        declarations: [
            ("width", "80px"),
            ("height", "40px"),
            (
                "background-image",
                "linear-gradient(90deg, red 75%, blue 100%)",
            ),
        ]
        .into_iter()
        .map(|(property, value)| StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        })
        .collect(),
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(120, 80);
    let rect = page.overflow_geometry[&1].border_box;
    let pixels = page.snapshot_pixels(120, 80);
    let x = (rect.left + (rect.right - rect.left) / 2.0).round() as usize;
    let y = (rect.top + (rect.bottom - rect.top) / 2.0).round() as usize;
    let center = &pixels[(y * 120 + x) * 4..][..3];

    assert!(
        center[0] > 220 && center[2] < 40,
        "75% stop should keep the center red: {center:?}"
    );
}

#[test]
fn background_shorthand_accepts_a_data_image_and_none_resets_it() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("image".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: Vec::new(),
    }];
    let image = png_data_url([255, 0, 0, 255]);
    let shorthand = [StyleRule {
        selectors: vec![".image".to_owned()],
        layer: None,
        declarations: [
            ("width", "30px"),
            ("height", "20px"),
            ("background", image.as_str()),
        ]
        .into_iter()
        .map(|(property, value)| StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        })
        .collect(),
    }];
    let mut page = PageRasterizer::new(&nodes, &shorthand);
    page.prepare_layout(100, 70);
    let rect = page.overflow_geometry[&1].border_box;
    let pixels = page.snapshot_pixels(100, 70);
    let x = (rect.left + 2.0).round() as usize;
    let y = (rect.top + 2.0).round() as usize;
    let pixel = &pixels[(y * 100 + x) * 4..][..3];
    assert_eq!(
        pixel,
        [255, 0, 0],
        "background shorthand should paint its image"
    );

    let reset = [StyleRule {
        selectors: vec![".image".to_owned()],
        layer: None,
        declarations: [
            ("width", "30px"),
            ("height", "20px"),
            ("background", image.as_str()),
            ("background", "none"),
        ]
        .into_iter()
        .map(|(property, value)| StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        })
        .collect(),
    }];
    let pixels = PageRasterizer::new(&nodes, &reset).snapshot_pixels(100, 70);
    let pixel = &pixels[(20 * 100 + 20) * 4..][..3];
    assert_ne!(pixel, [255, 0, 0], "background:none should clear the image");
}

#[test]
fn paints_inline_data_image_background_through_fragment_padding() {
    let nodes = [UiNode::Element {
        tag: "span".to_owned(),
        class: Some("image".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("X".to_owned())],
    }];
    let image = png_data_url([255, 0, 0, 255]);
    let stylesheet = [StyleRule {
        selectors: vec![".image".to_owned()],
        layer: None,
        declarations: [("padding", "8px"), ("background-image", image.as_str())]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
    }];
    let pixels = PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(100, 70);
    let pixel = &pixels[(20 * 100 + 8) * 4..][..3];

    assert_eq!(pixel, [255, 0, 0]);
}

#[test]
fn image_backgrounds_respect_element_opacity_and_ancestor_clipping() {
    let image = png_data_url([255, 0, 0, 255]);
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("clip".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "section".to_owned(),
            class: Some("image".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("X".to_owned())],
        }],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec![".clip".to_owned()],
            layer: None,
            declarations: [
                ("width", "20px"),
                ("height", "20px"),
                ("overflow", "hidden"),
            ]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
        },
        StyleRule {
            selectors: vec![".image".to_owned()],
            layer: None,
            declarations: [
                ("width", "80px"),
                ("height", "20px"),
                ("opacity", "0.5"),
                ("background-image", image.as_str()),
            ]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(100, 70);
    let clip = page.overflow_clips[&1];
    let pixels = page.snapshot_pixels(100, 70);
    let inside_x = (clip.left + 1.0) as usize;
    let inside_y = (clip.top + 1.0) as usize;
    let inside = &pixels[(inside_y * 100 + inside_x) * 4..][..3];
    let outside_x = (clip.right + 8.0) as usize;
    let outside = &pixels[(inside_y * 100 + outside_x) * 4..][..3];

    assert!(
        inside[0] > 200 && inside[1] > 80 && inside[1] < 200 && inside[2] > 80,
        "opacity should composite the image over the page: {inside:?}"
    );
    assert_eq!(
        outside,
        [255, 255, 255],
        "clipped image escaped its ancestor"
    );
}
