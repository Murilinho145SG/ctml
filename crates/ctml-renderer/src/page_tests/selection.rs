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
        declarations: vec![
            StyleDeclaration {
                property: "background-color".to_owned(),
                value: "red".to_owned(),
            },
            StyleDeclaration {
                property: "color".to_owned(),
                value: "lime".to_owned(),
            },
        ],
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
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [0, 255, 0])
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

#[test]
fn element_scoped_selection_styles_apply_only_to_matching_text() {
    let nodes = [
        UiNode::Element {
            tag: "p".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("Select this".to_owned())],
        },
        UiNode::Element {
            tag: "h1".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("Other text".to_owned())],
        },
    ];
    let stylesheet = [StyleRule {
        selectors: vec!["p::selection".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "background-color".to_owned(),
                value: "red".to_owned(),
            },
            StyleDeclaration {
                property: "color".to_owned(),
                value: "lime".to_owned(),
            },
        ],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let _baseline = page.snapshot_pixels(140, 70);
    assert!(page.begin_selection(10.0, 14.0));
    assert!(page.extend_selection(65.0, 14.0));

    let pixels = page.snapshot_pixels(140, 70);

    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [255, 0, 0])
    );
    assert!(
        !pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [173, 216, 230])
    );
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] == [0, 255, 0])
    );
}

#[test]
fn dragging_past_line_edges_selects_the_first_and_last_characters() {
    let text = "Select the whole line";
    let mut page = PageRasterizer::new(&[UiNode::Text(text.to_owned())], &[]);
    page.snapshot_pixels(320, 80);
    assert!(page.begin_selection(10.0, 14.0));
    assert!(page.extend_selection(500.0, 14.0));
    let (_, end) = page.selection.unwrap();
    assert_eq!(end.index, text.len());
    assert!(page.extend_selection(-50.0, 14.0));
    let (_, end) = page.selection.unwrap();
    assert_eq!(end.index, 0);
}

#[test]
fn dragging_below_wrapped_text_reaches_the_end() {
    let text = "Select this sentence across several wrapped lines.";
    let mut page = PageRasterizer::new(&[UiNode::Text(text.to_owned())], &[]);
    page.snapshot_pixels(120, 180);
    assert!(page.begin_selection(10.0, 14.0));
    assert!(page.extend_selection(500.0, 500.0));
    let (_, end) = page.selection.unwrap();
    assert_eq!(end.index, text.len());
}

#[test]
fn undecorated_text_does_not_allocate_glyph_masks() {
    let mut page = PageRasterizer::new(&[UiNode::Text("Plain text".to_owned())], &[]);
    page.snapshot_pixels(200, 80);
    assert!(page.cache.image_cache.is_empty());
}

#[test]
fn selection_background_uses_the_same_page_origin_as_text() {
    let nodes = [UiNode::Text("Select me".to_owned())];
    let stylesheet = [StyleRule {
        selectors: vec!["::selection".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "background-color".to_owned(),
            value: "red".to_owned(),
        }],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.snapshot_pixels(200, 80);
    let (left, top, end) = {
        let run = page.layout_runs().next().unwrap();
        (
            run.glyphs.first().unwrap().x + super::super::PAGE_PADDING,
            run.line_top + super::super::PAGE_PADDING,
            run.glyphs.last().unwrap().end,
        )
    };
    page.selection = Some((
        cosmic_text::Cursor::new(0, 0),
        cosmic_text::Cursor::new(0, end),
    ));
    let pixels = page.render_pixels(200, 80);
    let red_pixels: Vec<_> = pixels
        .chunks_exact(4)
        .enumerate()
        .filter(|(_, pixel)| pixel[..3] == [255, 0, 0])
        .map(|(index, _)| (index % 200, index / 200))
        .collect();
    assert!(!red_pixels.is_empty());
    assert_eq!(
        red_pixels.iter().map(|point| point.0).min().unwrap(),
        left.floor() as usize
    );
    assert_eq!(
        red_pixels.iter().map(|point| point.1).min().unwrap(),
        top.floor() as usize
    );
}
