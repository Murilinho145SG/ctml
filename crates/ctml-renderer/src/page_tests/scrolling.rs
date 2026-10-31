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

#[test]
fn nested_scroll_chains_to_parent_scrollers_at_inner_boundaries() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("outer".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "div".to_owned(),
            class: Some("inner".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Element {
                tag: "span".to_owned(),
                class: None,
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text("nested scroll chain ".repeat(48))],
            }],
        }],
    }];
    let overflow_rule = |class: &str, height: &str| StyleRule {
        selectors: vec![format!(".{class}")],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "height".to_owned(),
                value: height.to_owned(),
            },
            StyleDeclaration {
                property: "overflow-y".to_owned(),
                value: "auto".to_owned(),
            },
        ],
    };
    let stylesheet = [
        overflow_rule("outer", "48px"),
        overflow_rule("inner", "24px"),
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.snapshot_pixels(300, 120);

    assert!(page.scroll_by_at(0.0, -10_000.0, 32.0, 24.0));

    let inner = page.nested_scroll.get(&2).copied().unwrap_or_default();
    let outer = page.nested_scroll.get(&1).copied().unwrap_or_default();
    assert!(inner.1 > 0.0);
    assert!(outer.1 > 0.0);
    assert_eq!(page.buffer.scroll(), cosmic_text::Scroll::default());
}

#[test]
fn nested_overflow_scrolls_under_a_vertical_page_root() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("vertical nested overflow ".repeat(16))],
    }];
    for (writing_mode, pointer) in [
        ("vertical-rl", (280.0, 20.0)),
        ("vertical-lr", (20.0, 100.0)),
    ] {
        for axis in ["x", "y"] {
            let mut block_declarations = vec![
                StyleDeclaration {
                    property: "block-size".to_owned(),
                    value: "40px".to_owned(),
                },
                StyleDeclaration {
                    property: "inline-size".to_owned(),
                    value: "60px".to_owned(),
                },
                StyleDeclaration {
                    property: format!("overflow-{axis}"),
                    value: "auto".to_owned(),
                },
            ];
            if axis == "y" {
                block_declarations.push(StyleDeclaration {
                    property: "white-space".to_owned(),
                    value: "nowrap".to_owned(),
                });
            }
            let stylesheet = [
                StyleRule {
                    selectors: vec!["body".to_owned()],
                    layer: None,
                    declarations: vec![StyleDeclaration {
                        property: "writing-mode".to_owned(),
                        value: writing_mode.to_owned(),
                    }],
                },
                StyleRule {
                    selectors: vec!["div".to_owned()],
                    layer: None,
                    declarations: block_declarations,
                },
            ];
            let mut page = PageRasterizer::new(&nodes, &stylesheet);
            let before = page.snapshot_pixels(300, 120);
            assert_eq!(page.overflow_geometry.len(), 1);

            let delta = if axis == "x" {
                (-24.0, 0.0)
            } else {
                (0.0, -24.0)
            };
            let scroll_pointer = match (writing_mode, axis) {
                ("vertical-rl", "y") => (280.0, 50.0),
                ("vertical-lr", "x") => (40.0, 100.0),
                _ => pointer,
            };
            assert!(page.scroll_by_at(delta.0, delta.1, scroll_pointer.0, scroll_pointer.1));

            let offset = page
                .nested_scroll
                .values()
                .next()
                .copied()
                .unwrap_or_default();
            assert!(if axis == "x" {
                offset.0 > 0.0
            } else {
                offset.1 > 0.0
            });
            assert_eq!(page.buffer.scroll(), cosmic_text::Scroll::default());
            assert!(
                page.begin_selection(scroll_pointer.0, scroll_pointer.1),
                "{writing_mode} overflow-{axis}"
            );
            let after = page.snapshot_pixels(300, 120);
            assert_ne!(before, after, "{writing_mode} overflow-{axis}");
            let scrollbar_region = match (writing_mode, axis) {
                ("vertical-rl", "x") => (252..292, 60..68),
                ("vertical-lr", "x") => (8..48, 104..112),
                ("vertical-rl", "y") => (283..291, 8..68),
                ("vertical-lr", "y") => (40..48, 52..112),
                _ => unreachable!(),
            };
            assert!(
                after.chunks_exact(4).enumerate().any(|(index, pixel)| {
                    let x = index % 300;
                    let y = index / 300;
                    scrollbar_region.0.contains(&x)
                        && scrollbar_region.1.contains(&y)
                        && pixel[..3] == [142, 142, 142]
                }),
                "{writing_mode} overflow-{axis} scrollbar should be visible"
            );
        }
    }
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
