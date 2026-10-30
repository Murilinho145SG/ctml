use super::super::{PAGE_PADDING, PageRasterizer};
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn conic_gradient_starts_at_top_and_sweeps_clockwise() {
    let (page, pixels) = render(
        "conic-gradient(red 0deg, red 90deg, blue 90deg, blue 180deg, lime 180deg, lime 270deg, yellow 270deg, yellow 360deg)",
    );
    let rect = page.overflow_geometry[&1].border_box;
    let origin_x = (rect.left - PAGE_PADDING).round() as usize;
    let origin_y = (rect.top - PAGE_PADDING).round() as usize;
    let center_x = origin_x + 40;
    let center_y = origin_y + 20;

    let top = sample(&pixels, center_x + 2, center_y - 10);
    let right = sample(&pixels, center_x + 14, center_y + 6);
    let bottom = sample(&pixels, center_x - 2, center_y + 10);
    let left = sample(&pixels, center_x - 14, center_y - 6);
    assert!(top[0] > top[2], "top={top:?}");
    assert!(right[2] > right[0], "right={right:?}");
    assert!(bottom[1] > bottom[2], "bottom={bottom:?}");
    assert!(left[0] > left[2] && left[1] > left[2], "left={left:?}");
}

#[test]
fn conic_gradient_applies_start_angle_and_position() {
    let (page, pixels) = render(
        "conic-gradient(from 90deg at 25% 50%, red 0deg, red 90deg, blue 90deg, blue 360deg)",
    );
    let rect = page.overflow_geometry[&1].border_box;
    let origin_x = (rect.left - PAGE_PADDING).round() as usize;
    let origin_y = (rect.top - PAGE_PADDING).round() as usize;
    let center_x = origin_x + 20;
    let center_y = origin_y + 20;
    let right_of_center = sample(&pixels, center_x + 10, center_y + 2);
    let below_center = sample(&pixels, center_x - 2, center_y + 10);

    assert!(
        right_of_center[0] > right_of_center[2],
        "{right_of_center:?}"
    );
    assert!(below_center[2] > below_center[0], "{below_center:?}");
}

#[test]
fn repeating_conic_gradient_repeats_color_sectors_around_the_center() {
    let (page, pixels) =
        render("repeating-conic-gradient(red 0deg, red 45deg, blue 45deg, blue 90deg)");
    let rect = page.overflow_geometry[&1].border_box;
    let origin_x = (rect.left - PAGE_PADDING).round() as usize;
    let origin_y = (rect.top - PAGE_PADDING).round() as usize;
    let center_x = origin_x + 40;
    let center_y = origin_y + 20;
    let first_red_sector = sample(&pixels, center_x + 2, center_y - 10);
    let first_blue_sector = sample(&pixels, center_x + 10, center_y - 5);
    let repeated_red_sector = sample(&pixels, center_x - 2, center_y + 10);
    let repeated_blue_sector = sample(&pixels, center_x - 10, center_y + 5);

    assert!(
        first_red_sector[0] > first_red_sector[2],
        "{first_red_sector:?}"
    );
    assert!(
        first_blue_sector[2] > first_blue_sector[0],
        "{first_blue_sector:?}"
    );
    assert!(
        repeated_red_sector[0] > repeated_red_sector[2],
        "{repeated_red_sector:?}"
    );
    assert!(
        repeated_blue_sector[2] > repeated_blue_sector[0],
        "{repeated_blue_sector:?}"
    );
}

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
