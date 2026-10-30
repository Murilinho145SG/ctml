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

fn repeating_border_image_url() -> String {
    let colors = [
        [255, 0, 0, 255],
        [0, 255, 0, 255],
        [255, 255, 0, 255],
        [255, 0, 255, 255],
        [0, 0, 255, 255],
        [255, 255, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        [255, 0, 0, 255],
        [0, 255, 255, 255],
        [128, 128, 128, 255],
        [0, 0, 0, 255],
        [128, 128, 128, 255],
        [0, 255, 255, 255],
        [255, 255, 255, 255],
    ]
    .concat();
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, 5, 3);
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

#[test]
fn border_image_width_number_scales_the_painted_border_area() {
    let (rect, pixels) = render_with_width("1 fill", Some("2"));
    let extended_corner = sample_at(&pixels, rect.left + 6.0, rect.top + 6.0);
    assert_eq!(extended_corner, [255, 0, 0]);
}

#[test]
fn border_image_width_axis_pairs_and_percentages_resolve_per_side() {
    let (rect, pixels) = render_with_width("1 fill", Some("2 1"));
    let top_strip = sample_at(&pixels, rect.left + 6.0, rect.top + 6.0);
    let corner = sample_at(&pixels, rect.left + 2.0, rect.top + 6.0);
    assert_eq!(top_strip, [0, 255, 0]);
    assert_eq!(corner, [255, 0, 0]);

    let (rect, pixels) = render_with_width("1 fill", Some("25%"));
    let extended_corner = sample_at(&pixels, rect.left + 8.0, rect.top + 8.0);
    assert_eq!(extended_corner, [255, 0, 0]);
}

#[test]
fn border_image_outset_extends_paint_bounds_without_changing_slice_content() {
    let (rect, pixels) = render_with_outset("2");
    assert_eq!(
        sample_at(&pixels, rect.right + 6.0, rect.top + 20.0),
        [255, 0, 255]
    );

    let (rect, pixels) = render_with_outset("3px");
    assert_eq!(
        sample_at(&pixels, rect.right + 1.0, rect.top + 20.0),
        [255, 0, 255]
    );
}

#[test]
fn border_image_shorthand_expands_values_and_respects_longhand_source_order() {
    let (rect, pixels) = render_shorthand(false);
    assert_eq!(
        sample_at(&pixels, rect.left + 6.0, rect.top + 6.0),
        [255, 0, 0]
    );
    assert_eq!(
        sample_at(&pixels, rect.left + 20.0, rect.top + 16.0),
        [0, 255, 255]
    );

    let (rect, pixels) = render_shorthand(true);
    assert_eq!(
        sample_at(&pixels, rect.left + 20.0, rect.top + 16.0),
        [255, 255, 255]
    );
}

#[test]
fn border_image_repeat_tiles_the_middle_source_strip() {
    let (rect, pixels) = render_repeated_border("repeat stretch", "1");
    let sample = |x: f32| sample_at(&pixels, rect.left + x, rect.top + 1.0);

    assert_eq!(sample(6.0), [0, 255, 0]);
    assert_eq!(sample(10.0), [255, 255, 0]);
    assert_eq!(sample(14.0), [255, 0, 255]);
    assert_eq!(sample(18.0), [0, 255, 0]);
}

#[test]
fn border_image_round_resizes_tiles_to_fill_the_edge_without_gaps() {
    let (rect, pixels) = render_repeated_border("round stretch", "1");
    for x in 4..44 {
        assert_ne!(
            sample_at(&pixels, rect.left + x as f32, rect.top + 1.0),
            [0, 255, 255],
            "unexpected gap at x={x}"
        );
    }
}

#[test]
fn border_image_space_distributes_gaps_between_tiles() {
    let (rect, pixels) = render_repeated_border("space stretch", "1");
    let gap = sample_at(&pixels, rect.left + 17.0, rect.top + 1.0);
    let next_tile = sample_at(&pixels, rect.left + 19.0, rect.top + 1.0);
    assert_eq!(gap, [0, 255, 255]);
    assert_eq!(next_tile, [0, 255, 0]);
}

#[test]
fn border_image_repeat_tiles_a_filled_center_both_ways() {
    let (rect, pixels) = render_repeated_border("repeat repeat", "1 fill");
    let sample = |x: f32| sample_at(&pixels, rect.left + x, rect.top + 5.0);
    assert_eq!(sample(5.0), [0, 255, 0]);
    assert_eq!(sample(9.0), [0, 0, 255]);
    assert_eq!(sample(13.0), [255, 0, 0]);
    assert_eq!(sample(17.0), [0, 255, 0]);
}

fn render_repeated_border(
    repeat: &str,
    slice: &str,
) -> (super::super::overflow_clip::ClipRect, Vec<u8>) {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("repeating-border".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: Vec::new(),
    }];
    let source = repeating_border_image_url();
    let stylesheet = [StyleRule {
        selectors: vec![".repeating-border".to_owned()],
        layer: None,
        declarations: [
            ("width", "40px"),
            ("height", "32px"),
            ("display", "block"),
            ("border-width", "4px"),
            ("background-color", "cyan"),
            ("border-image-source", source.as_str()),
            ("border-image-slice", slice),
            ("border-image-repeat", repeat),
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

fn render(slice: &str) -> (super::super::overflow_clip::ClipRect, Vec<u8>) {
    render_with_width(slice, None)
}

fn render_with_width(
    slice: &str,
    image_width: Option<&str>,
) -> (super::super::overflow_clip::ClipRect, Vec<u8>) {
    render_with_border_geometry(slice, image_width, None)
}

fn render_with_outset(outset: &str) -> (super::super::overflow_clip::ClipRect, Vec<u8>) {
    render_with_border_geometry("1", None, Some(outset))
}

fn render_shorthand(slice_first: bool) -> (super::super::overflow_clip::ClipRect, Vec<u8>) {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("image-border".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: Vec::new(),
    }];
    let source = border_image_url();
    let shorthand = format!("{} 1 fill / 2", source);
    let mut declarations = vec![
        ("width", "40px"),
        ("height", "32px"),
        ("border-width", "4px"),
        ("background-color", "cyan"),
    ];
    if slice_first {
        declarations.push(("border-image-slice", "1"));
    }
    declarations.push(("border-image", shorthand.as_str()));
    if !slice_first {
        declarations.push(("border-image-slice", "1"));
    }
    let stylesheet = [StyleRule {
        selectors: vec![".image-border".to_owned()],
        layer: None,
        declarations: declarations
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

fn render_with_border_geometry(
    slice: &str,
    image_width: Option<&str>,
    image_outset: Option<&str>,
) -> (super::super::overflow_clip::ClipRect, Vec<u8>) {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("image-border".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: Vec::new(),
    }];
    let source = border_image_url();
    let mut declarations = vec![
        ("width", "40px"),
        ("height", "32px"),
        ("border-width", "4px"),
        ("background-color", "cyan"),
        ("border-image-source", source.as_str()),
        ("border-image-slice", slice),
    ];
    if let Some(image_width) = image_width {
        declarations.push(("border-image-width", image_width));
    }
    if let Some(image_outset) = image_outset {
        declarations.push(("border-image-outset", image_outset));
    }
    let stylesheet = [StyleRule {
        selectors: vec![".image-border".to_owned()],
        layer: None,
        declarations: declarations
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
