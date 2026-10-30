use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};
use base64::{Engine, engine::general_purpose::STANDARD};

fn vertical_stripe_data_url(colors: &[[u8; 4]]) -> String {
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, 1, colors.len() as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        let pixels = colors.iter().flatten().copied().collect::<Vec<_>>();
        writer.write_image_data(&pixels).unwrap();
    }
    format!(
        "url(\"data:image/png;base64,{}\")",
        STANDARD.encode(encoded)
    )
}

#[test]
fn fixed_background_attachment_stays_viewport_anchored_during_nested_scroll() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("scroller".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "div".to_owned(),
            class: Some("image".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: Vec::new(),
        }],
    }];
    let image = vertical_stripe_data_url(&[[255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255]]);
    for attachment in ["scroll", "local", "fixed"] {
        let stylesheet = [
            StyleRule {
                selectors: vec![".scroller".to_owned()],
                layer: None,
                declarations: [
                    ("width", "24px"),
                    ("height", "24px"),
                    ("overflow-y", "auto"),
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
                    ("width", "24px"),
                    ("height", "48px"),
                    ("background-image", image.as_str()),
                    ("background-attachment", attachment),
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
        let before = page.snapshot_pixels(100, 80);
        assert!(page.scroll_by_at(0.0, -1.0, 16.0, 16.0));
        let after = page.snapshot_pixels(100, 80);
        let sample = |pixels: &[u8]| {
            let pixel = &pixels[(16 * 100 + 12) * 4..][..3];
            [pixel[0], pixel[1], pixel[2]]
        };

        if attachment == "fixed" {
            assert_eq!(sample(&before), sample(&after));
        } else {
            assert_ne!(sample(&before), sample(&after));
        }
    }
}
