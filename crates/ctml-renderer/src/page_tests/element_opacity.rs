use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

fn declaration(property: &str, value: &str) -> StyleDeclaration {
    StyleDeclaration {
        property: property.to_owned(),
        value: value.to_owned(),
    }
}

#[test]
fn element_opacity_scales_fragment_background_and_stays_on_text_runs() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Opacity".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![
            declaration("background-color", "blue"),
            declaration("color", "red"),
            declaration("opacity", "0.5"),
        ],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(160, 70);

    assert!(
        page.runs
            .iter()
            .any(|run| { run.style.opacity == 0.5 && run.style.color == [255, 0, 0, 255] })
    );
    assert!(pixels.chunks_exact(4).any(|pixel| {
        (pixel[0] == 127 || pixel[0] == 128)
            && (pixel[1] == 127 || pixel[1] == 128)
            && pixel[2] == 255
    }));
}

#[test]
fn nested_element_opacities_multiply_for_descendant_paints() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "span".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("Nested".to_owned())],
        }],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![
                declaration("color", "transparent"),
                declaration("opacity", "0.5"),
            ],
        },
        StyleRule {
            selectors: vec!["span".to_owned()],
            layer: None,
            declarations: vec![
                declaration("background-color", "lime"),
                declaration("color", "transparent"),
                declaration("opacity", "0.5"),
            ],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(160, 70);

    assert!(page.runs.iter().any(|run| run.style.opacity == 0.25));
    assert!(pixels.chunks_exact(4).any(|pixel| {
        (pixel[0] == 191 || pixel[0] == 192)
            && pixel[1] == 255
            && (pixel[2] == 191 || pixel[2] == 192)
    }));
}

#[test]
fn page_root_opacity_composites_overlapping_page_paints_as_one_group() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Overlap".to_owned())],
    }];
    let declarations = |opacity: &str| {
        vec![
            declaration("background-color", "blue"),
            declaration("opacity", opacity),
        ]
    };
    let opaque_stylesheet = [
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: declarations("1"),
        },
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![declaration("color", "red")],
        },
    ];
    let translucent_stylesheet = [
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: declarations("0.5"),
        },
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![declaration("color", "red")],
        },
    ];
    let opaque = PageRasterizer::new(&nodes, &opaque_stylesheet).snapshot_pixels(160, 70);
    let translucent = PageRasterizer::new(&nodes, &translucent_stylesheet).snapshot_pixels(160, 70);

    assert!(
        opaque
            .chunks_exact(4)
            .any(|pixel| pixel[0] > pixel[1] && pixel[0] > 180)
    );
    for (opaque, translucent) in opaque.chunks_exact(4).zip(translucent.chunks_exact(4)) {
        for (actual, source) in translucent[..3].iter().zip(&opaque[..3]) {
            let expected = (255.0 + (f32::from(*source) - 255.0) * 0.5).round() as u8;
            assert!(actual.abs_diff(expected) <= 1);
        }
    }
}

#[test]
fn element_opacity_scales_text_decorations() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Underline".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![
            declaration("color", "transparent"),
            declaration("text-decoration-line", "underline"),
            declaration("text-decoration-color", "red"),
            declaration("opacity", "0.5"),
        ],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(160, 70);

    assert!(pixels.chunks_exact(4).any(|pixel| {
        pixel[0] == 255
            && (pixel[1] == 127 || pixel[1] == 128)
            && (pixel[2] == 127 || pixel[2] == 128)
    }));
}
