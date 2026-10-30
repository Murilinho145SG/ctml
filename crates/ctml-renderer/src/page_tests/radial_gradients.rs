use super::super::{PAGE_PADDING, PageRasterizer};
use crate::{StyleDeclaration, StyleRule, UiNode};

fn render(gradient: &str) -> (PageRasterizer, Vec<u8>) {
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
            ("background-image", gradient),
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
    let pixels = page.snapshot_pixels(120, 80);
    (page, pixels)
}

fn sample(pixels: &[u8], x: usize, y: usize) -> [u8; 3] {
    let pixel = &pixels[(y * 120 + x) * 4..][..3];
    [pixel[0], pixel[1], pixel[2]]
}

#[test]
fn radial_gradient_supports_ellipse_circle_and_position_descriptors() {
    let (ellipse, ellipse_pixels) = render("radial-gradient(ellipse at 25% 50%, red, blue)");
    let rect = ellipse.overflow_geometry[&1].border_box;
    let origin_x = (rect.left - PAGE_PADDING).round() as usize;
    let origin_y = (rect.top - PAGE_PADDING).round() as usize;
    let ellipse_center = sample(&ellipse_pixels, origin_x + 20, origin_y + 20);
    let ellipse_edge = sample(&ellipse_pixels, origin_x + 76, origin_y + 20);
    assert!(ellipse_center[0] > ellipse_center[2], "{ellipse_center:?}");
    assert!(ellipse_edge[2] > ellipse_edge[0], "{ellipse_edge:?}");

    let (circle, circle_pixels) =
        render("radial-gradient(circle closest-side at center, red, blue)");
    let rect = circle.overflow_geometry[&1].border_box;
    let origin_x = (rect.left - PAGE_PADDING).round() as usize;
    let origin_y = (rect.top - PAGE_PADDING).round() as usize;
    let circle_center = sample(&circle_pixels, origin_x + 40, origin_y + 20);
    let circle_edge = sample(&circle_pixels, origin_x + 40, origin_y + 38);
    assert!(circle_center[0] > circle_center[2], "{circle_center:?}");
    assert!(circle_edge[2] > circle_edge[0], "{circle_edge:?}");
}

#[test]
fn repeating_radial_gradient_repeats_percentage_stop_patterns() {
    let (page, pixels) =
        render("repeating-radial-gradient(circle, red 0%, red 25%, blue 25%, blue 50%)");
    let rect = page.overflow_geometry[&1].border_box;
    let origin_x = (rect.left - PAGE_PADDING).round() as usize;
    let origin_y = (rect.top - PAGE_PADDING).round() as usize;
    let center_x = origin_x + 40;
    let center_y = origin_y + 20;
    let center = sample(&pixels, center_x, center_y);
    let first_blue_ring = sample(&pixels, center_x + 14, center_y);
    let repeated_red_ring = sample(&pixels, center_x + 25, center_y);
    let repeated_blue_ring = sample(&pixels, center_x + 35, center_y);

    assert!(center[0] > center[2], "center={center:?}");
    assert!(
        first_blue_ring[2] > first_blue_ring[0],
        "{first_blue_ring:?}"
    );
    assert!(
        repeated_red_ring[0] > repeated_red_ring[2],
        "{repeated_red_ring:?}"
    );
    assert!(
        repeated_blue_ring[2] > repeated_blue_ring[0],
        "{repeated_blue_ring:?}"
    );
}
