use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn root_overflow_auto_scrolls_with_wheel_offsets_but_hidden_does_not() {
    let nodes = (0..40)
        .map(|index| text_paragraph(&format!("scroll line {index}")))
        .collect::<Vec<_>>();
    let body_rule = |overflow: &str| StyleRule {
        selectors: vec!["body".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "overflow-y".to_owned(),
            value: overflow.to_owned(),
        }],
    };
    let mut scrollable = PageRasterizer::new(&nodes, &[body_rule("auto")]);
    let before = scrollable.snapshot_pixels(220, 100);
    assert!(scrollable.scroll_by(0.0, -36.0));
    let after = scrollable.snapshot_pixels(220, 100);
    assert_ne!(
        before, after,
        "root scrolling should move the viewport content"
    );
    let offset = scrollable.buffer.scroll();
    assert!(offset.line > 0 || offset.vertical > 0.0);

    let mut clipped = PageRasterizer::new(&nodes, &[body_rule("hidden")]);
    clipped.snapshot_pixels(220, 100);
    assert!(!clipped.scroll_by(0.0, -36.0));
}

#[test]
fn root_overflow_scroll_draws_a_scrollbar_and_auto_hides_it_without_overflow() {
    let rule = |overflow: &str| StyleRule {
        selectors: vec!["body".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "overflow-y".to_owned(),
            value: overflow.to_owned(),
        }],
    };
    let short = [text_paragraph("short")];
    let mut scroll = PageRasterizer::new(&short, &[rule("scroll")]);
    let forced = scroll.snapshot_pixels(80, 60);
    assert!(forced.chunks_exact(4).enumerate().any(|(index, pixel)| {
        let x = (index % 80) as u32;
        x >= 72 && pixel[..3] == [142, 142, 142]
    }));

    let mut automatic = PageRasterizer::new(&short, &[rule("auto")]);
    let automatic = automatic.snapshot_pixels(80, 60);
    assert!(
        !automatic
            .chunks_exact(4)
            .enumerate()
            .any(|(index, pixel)| (index % 80) as u32 >= 72 && pixel[..3] == [142, 142, 142])
    );
}

#[test]
fn nested_overflow_auto_scrolls_under_the_pointer_without_scrolling_the_page() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "span".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("nested scroll text ".repeat(40))],
        }],
    }];
    let rule = StyleRule {
        selectors: vec!["div".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "height".to_owned(),
                value: "32px".to_owned(),
            },
            StyleDeclaration {
                property: "overflow-y".to_owned(),
                value: "auto".to_owned(),
            },
        ],
    };
    let stylesheet = [
        rule,
        StyleRule {
            selectors: vec!["span".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "background-color".to_owned(),
                value: "#ff0000".to_owned(),
            }],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let before = page.snapshot_pixels(300, 120);
    assert_eq!(page.overflow_geometry.len(), 1);
    assert!(page.scroll_by_at(0.0, -24.0, 32.0, 24.0));
    assert!(
        page.nested_scroll
            .values()
            .any(|(_, offset_y)| *offset_y > 0.0)
    );
    assert_eq!(page.buffer.scroll(), cosmic_text::Scroll::default());
    let after = page.snapshot_pixels(300, 120);
    assert_ne!(before, after);
    let clip = page.overflow_geometry[&1].clip;
    assert!(
        before
            .chunks_exact(4)
            .zip(after.chunks_exact(4))
            .enumerate()
            .any(|(index, (before, after))| {
                let x = (index % 300) as f32;
                let y = (index / 300) as f32;
                x < clip.right - 8.0 && y >= clip.top && y < clip.bottom && before != after
            })
    );
    assert!(
        before
            .chunks_exact(4)
            .zip(after.chunks_exact(4))
            .enumerate()
            .any(|(index, (before, after))| {
                let x = (index % 300) as f32;
                let y = (index / 300) as f32;
                x < clip.right - 8.0
                    && y >= clip.top
                    && y < clip.bottom
                    && ((before[..3] == [255, 0, 0]) != (after[..3] == [255, 0, 0]))
            })
    );
    assert!(after.chunks_exact(4).enumerate().any(|(index, pixel)| {
        let x = (index % 300) as f32;
        let y = (index / 300) as f32;
        x >= clip.right - 8.0
            && x < clip.right
            && y >= clip.top
            && y < clip.bottom
            && pixel[..3] == [142, 142, 142]
    }));
    assert!(page.begin_selection(32.0, 24.0));
    assert!(page.selection.is_some());
}

#[test]
fn nested_overflow_x_scrolls_unwrapped_text_horizontally() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "span".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("wide inline content ".repeat(24))],
        }],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["div".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "width".to_owned(),
                    value: "80px".to_owned(),
                },
                StyleDeclaration {
                    property: "height".to_owned(),
                    value: "50px".to_owned(),
                },
                StyleDeclaration {
                    property: "overflow-x".to_owned(),
                    value: "auto".to_owned(),
                },
            ],
        },
        StyleRule {
            selectors: vec!["span".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "white-space".to_owned(),
                value: "nowrap".to_owned(),
            }],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let before = page.snapshot_pixels(300, 120);
    assert!(page.scroll_by_at(-24.0, 0.0, 32.0, 24.0));
    let after = page.snapshot_pixels(300, 120);
    let (offset_x, offset_y) = page.nested_scroll[&1];
    assert!(offset_x > 0.0);
    assert_eq!(offset_y, 0.0);
    assert_eq!(page.buffer.scroll(), cosmic_text::Scroll::default());
    assert_ne!(before, after);
    let clip = page.overflow_geometry[&1].clip;
    assert!(after.chunks_exact(4).enumerate().any(|(index, pixel)| {
        let x = (index % 300) as f32;
        let y = (index / 300) as f32;
        x >= clip.left
            && x < clip.right
            && y >= clip.bottom - 8.0
            && y < clip.bottom
            && pixel[..3] == [142, 142, 142]
    }));
}

fn text_paragraph(text: &str) -> UiNode {
    UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text(text.to_owned())],
    }
}
