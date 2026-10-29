use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

#[test]
fn selection_pseudo_element_sets_highlight_background_color() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Select me".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["::selection".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "background-color".to_owned(),
            value: "red".to_owned(),
        }],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let _baseline = page.snapshot_pixels(120, 50);
    assert!(page.begin_selection(10.0, 14.0));
    assert!(page.extend_selection(60.0, 14.0));

    let pixels = page.snapshot_pixels(120, 50);

    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 0, 0])
    );
}

#[test]
fn selection_highlight_composites_alpha_over_existing_background() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Select me".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["::selection".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "background-color".to_owned(),
            value: "rgb(255 0 0 / 50%)".to_owned(),
        }],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let _baseline = page.snapshot_pixels(120, 50);
    assert!(page.begin_selection(10.0, 14.0));
    assert!(page.extend_selection(60.0, 14.0));

    let pixels = page.snapshot_pixels(120, 50);

    assert!(pixels.chunks_exact(4).any(|pixel| {
        pixel[0] == 255 && (120..=135).contains(&pixel[1]) && pixel[1] == pixel[2]
    }));
}
