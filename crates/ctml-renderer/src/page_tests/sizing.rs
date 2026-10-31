use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn body_width_constrains_text_wrapping() {
    let text = "a sequence of words that takes several lines when the body is narrow ".repeat(8);
    let narrow_line_count = line_count_with_rule("width", "80px", text.clone(), 400);
    let wide_line_count = line_count_without_rule(text, 400);

    assert!(
        narrow_line_count > wide_line_count,
        "narrow body should wrap to more lines ({narrow_line_count} vs {wide_line_count})"
    );
}

#[test]
fn lh_unit_resolves_against_inherited_line_height() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Line height units".to_owned())],
    }];
    let make_page = |paragraph_declaration: (&str, &str)| {
        PageRasterizer::new(
            &nodes,
            &[
                StyleRule {
                    selectors: vec!["body".to_owned()],
                    layer: None,
                    declarations: vec![StyleDeclaration {
                        property: "line-height".to_owned(),
                        value: "24px".to_owned(),
                    }],
                },
                StyleRule {
                    selectors: vec!["p".to_owned()],
                    layer: None,
                    declarations: vec![StyleDeclaration {
                        property: paragraph_declaration.0.to_owned(),
                        value: paragraph_declaration.1.to_owned(),
                    }],
                },
            ],
        )
    };

    let font_size_page = make_page(("font-size", "2lh"));
    assert_eq!(font_size_page.runs[0].style.font_size, 48.0);

    let line_height_page = make_page(("line-height", "2lh"));
    assert_eq!(line_height_page.runs[0].style.line_height, 48.0);

    let font_shorthand_page = make_page(("font", "12px/2lh serif"));
    assert_eq!(font_shorthand_page.runs[0].style.font_size, 12.0);
    assert_eq!(font_shorthand_page.runs[0].style.line_height, 48.0);
}

#[test]
fn font_size_metric_units_use_the_parent_typeface() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Metric sized text".to_owned())],
    }];
    for unit in ["ex", "ch", "ic", "cap"] {
        let stylesheet = [
            StyleRule {
                selectors: vec!["body".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: "font-size".to_owned(),
                    value: "20px".to_owned(),
                }],
            },
            StyleRule {
                selectors: vec!["p".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: "font-size".to_owned(),
                    value: format!("2{unit}"),
                }],
            },
        ];
        let page = PageRasterizer::new(&nodes, &stylesheet);
        let metrics = page.root_style.font_relative_units;
        let factor = match unit {
            "ex" => metrics.ex,
            "ch" => metrics.ch,
            "ic" => metrics.ic,
            "cap" => metrics.cap,
            _ => unreachable!(),
        };
        let expected_size = 40.0 * factor;
        assert!(
            (page.runs[0].style.font_size - expected_size).abs() < 0.01,
            "{unit} should resolve using parent font metrics"
        );
    }
}

#[test]
fn font_shorthand_resolves_metric_sized_font_from_parent_typeface() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Metric shorthand".to_owned())],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font-size".to_owned(),
                value: "20px".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "font".to_owned(),
                value: "2ch serif".to_owned(),
            }],
        },
    ];
    let page = PageRasterizer::new(&nodes, &stylesheet);
    let expected_size = 40.0 * page.root_style.font_relative_units.ch;
    assert!((page.runs[0].style.font_size - expected_size).abs() < 0.01);
}

#[test]
fn first_letter_font_size_metric_units_use_the_originating_font() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Metric first letter".to_owned())],
    }];
    let stylesheet = [StyleRule {
        selectors: vec!["p::first-letter".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "font-size".to_owned(),
            value: "2ex".to_owned(),
        }],
    }];
    let page = PageRasterizer::new(&nodes, &stylesheet);
    let expected_size = 32.0 * page.runs[0].style.font_relative_units.ex;
    assert!((page.runs[0].style.font_size - expected_size).abs() < 0.01);
}

#[test]
fn body_min_and_max_width_constrain_text_wrapping() {
    let text = "words that wrap when the body width is constrained ".repeat(8);
    let min_width_lines = line_count_with_rule("min-width", "800px", text.clone(), 160);
    let min_width_reference = line_count_without_rule(text.clone(), 160);
    let max_width_lines = line_count_with_rule("max-width", "80px", text.clone(), 400);
    let wide_lines = line_count_without_rule(text, 400);

    assert!(min_width_lines < min_width_reference);
    assert!(max_width_lines > wide_lines);
}

#[test]
fn logical_inline_size_properties_use_the_horizontal_width_axis() {
    let text = "logical inline size aliases constrain text wrapping ".repeat(8);
    let inline_size_lines = line_count_with_rule("inline-size", "80px", text.clone(), 400);
    let min_inline_size_lines = line_count_with_rule("min-inline-size", "800px", text.clone(), 160);
    let min_reference_lines = line_count_without_rule(text.clone(), 160);
    let max_inline_size_lines = line_count_with_rule("max-inline-size", "80px", text.clone(), 400);
    let wide_lines = line_count_without_rule(text, 400);

    assert!(inline_size_lines > wide_lines);
    assert!(min_inline_size_lines < min_reference_lines);
    assert!(max_inline_size_lines > wide_lines);
}

#[test]
fn vertical_writing_uses_inline_size_on_the_column_flow_axis() {
    let text = "logical vertical size constrains column text flow ".repeat(10);
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text(text)],
    }];
    let vertical_root = StyleRule {
        selectors: vec!["body".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "writing-mode".to_owned(),
            value: "vertical-rl".to_owned(),
        }],
    };
    let mut unconstrained = PageRasterizer::new(&nodes, std::slice::from_ref(&vertical_root));
    unconstrained.prepare_layout(1000, 500);
    let unconstrained_lines = unconstrained.buffer.layout_runs().count();

    let root_with_inline_size = [
        vertical_root.clone(),
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "inline-size".to_owned(),
                value: "40px".to_owned(),
            }],
        },
    ];
    let mut root_sized = PageRasterizer::new(&nodes, &root_with_inline_size);
    root_sized.prepare_layout(1000, 500);
    let root_sized_lines = root_sized.buffer.layout_runs().count();
    assert!(
        root_sized_lines > unconstrained_lines,
        "root inline-size should constrain columns ({root_sized_lines} vs {unconstrained_lines}, {:?})",
        root_sized.root_style.inline_size
    );

    let nested_with_inline_size = [
        vertical_root,
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "inline-size".to_owned(),
                value: "40px".to_owned(),
            }],
        },
    ];
    let mut nested_sized = PageRasterizer::new(&nodes, &nested_with_inline_size);
    nested_sized.prepare_layout(1000, 500);
    assert!(nested_sized.buffer.layout_runs().count() > unconstrained_lines);
}

#[test]
fn nested_orthogonal_writing_mode_wraps_on_its_inline_axis() {
    let text = "nested vertical writing uses the page height as its inline axis ".repeat(10);
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text(text.clone())],
    }];
    let horizontal_page = PageRasterizer::new(&nodes, &[]);
    let mut horizontal_page = horizontal_page;
    horizontal_page.prepare_layout(160, 400);
    let horizontal_lines = horizontal_page.buffer.layout_runs().count();

    for writing_mode in ["vertical-rl", "vertical-lr"] {
        let vertical_child = [StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "writing-mode".to_owned(),
                value: writing_mode.to_owned(),
            }],
        }];
        let mut nested_page = PageRasterizer::new(&nodes, &vertical_child);
        nested_page.prepare_layout(160, 400);
        let nested_lines = nested_page.buffer.layout_runs().count();

        assert!(
            nested_lines < horizontal_lines,
            "{writing_mode} child should wrap against the taller inline axis ({nested_lines} vs {horizontal_lines})"
        );
    }
}

#[test]
fn nested_vertical_text_is_composed_in_its_local_writing_mode() {
    let nodes = [UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Text("Vertical text column ".repeat(16))],
    }];
    for writing_mode in ["vertical-rl", "vertical-lr"] {
        let stylesheet = [
            StyleRule {
                selectors: vec!["p".to_owned()],
                layer: None,
                declarations: vec![
                    StyleDeclaration {
                        property: "writing-mode".to_owned(),
                        value: writing_mode.to_owned(),
                    },
                    StyleDeclaration {
                        property: "cursor".to_owned(),
                        value: "pointer".to_owned(),
                    },
                    StyleDeclaration {
                        property: "background-color".to_owned(),
                        value: "#00ff00".to_owned(),
                    },
                    StyleDeclaration {
                        property: "color".to_owned(),
                        value: "#ff00ff".to_owned(),
                    },
                    StyleDeclaration {
                        property: "text-decoration-line".to_owned(),
                        value: "underline".to_owned(),
                    },
                    StyleDeclaration {
                        property: "text-decoration-color".to_owned(),
                        value: "#ff0000".to_owned(),
                    },
                    StyleDeclaration {
                        property: "text-decoration-skip-ink".to_owned(),
                        value: "all".to_owned(),
                    },
                ],
            },
            StyleRule {
                selectors: vec!["::selection".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: "background-color".to_owned(),
                    value: "#0000ff".to_owned(),
                }],
            },
        ];
        let mut page = PageRasterizer::new(&nodes, &stylesheet);
        let pixels = page.snapshot_pixels(160, 400);
        let origins = super::super::local_writing::vertical_block_origins(
            &page.runs,
            &page.buffer,
            crate::style::WritingMode::HorizontalTb,
            &page.flex_wrap_line_offsets,
        );
        let first_line = page.buffer.layout_runs().next().unwrap();
        let first_glyph = first_line.glyphs.first().unwrap();
        let style_index = super::super::local_writing::style_index_at(
            &super::super::local_writing::text_style_ranges(&page.runs),
            first_glyph.start,
        )
        .unwrap();
        let block = page.runs[style_index].first_line_block.unwrap();
        let transform = super::super::local_writing::LocalVerticalTransform::new(
            page.runs[style_index].style.writing_mode,
            origins[&block],
            160,
            400,
        );
        let (left, top, glyph_width, glyph_height) = transform.map_rect(
            first_glyph.x + 8.0,
            first_line.line_top + 8.0,
            first_glyph.w,
            first_line.line_height,
        );
        let first_point = (
            f64::from(left + glyph_width / 2.0),
            f64::from(top + glyph_height / 2.0),
        );
        assert_eq!(
            page.cursor_at(first_point.0, first_point.1).0,
            winit::window::CursorIcon::Pointer
        );
        let second_glyph = &first_line.glyphs[5];
        let (second_left, second_top, second_width, second_height) = transform.map_rect(
            second_glyph.x + 8.0,
            first_line.line_top + 8.0,
            second_glyph.w,
            first_line.line_height,
        );
        let second_point = (
            f64::from(second_left + second_width / 2.0),
            f64::from(second_top + second_height / 2.0),
        );
        let mut bounds = (160_u32, 400_u32, 0_u32, 0_u32);
        let mut decoration_bounds = (160_u32, 400_u32, 0_u32, 0_u32);
        for y in 0..400 {
            for x in 0..160 {
                let index = ((y * 160 + x) * 4) as usize;
                if pixels[index] > 200 && pixels[index + 1] < 80 && pixels[index + 2] > 200 {
                    bounds.0 = bounds.0.min(x);
                    bounds.1 = bounds.1.min(y);
                    bounds.2 = bounds.2.max(x);
                    bounds.3 = bounds.3.max(y);
                }
                if pixels[index] > 200 && pixels[index + 1] < 80 && pixels[index + 2] < 80 {
                    decoration_bounds.0 = decoration_bounds.0.min(x);
                    decoration_bounds.1 = decoration_bounds.1.min(y);
                    decoration_bounds.2 = decoration_bounds.2.max(x);
                    decoration_bounds.3 = decoration_bounds.3.max(y);
                }
            }
        }
        let painted_width = bounds.2.saturating_sub(bounds.0);
        let painted_height = bounds.3.saturating_sub(bounds.1);
        assert!(
            painted_height > painted_width * 2,
            "{writing_mode} glyphs should retain vertical composition: {bounds:?}"
        );
        let decoration_width = decoration_bounds.2.saturating_sub(decoration_bounds.0);
        let decoration_height = decoration_bounds.3.saturating_sub(decoration_bounds.1);
        assert!(
            decoration_height > decoration_width * 2,
            "{writing_mode} underline: {decoration_bounds:?}"
        );

        assert!(page.begin_selection(first_point.0, first_point.1));
        assert!(page.extend_selection(second_point.0, second_point.1));
        let selected_pixels = page.snapshot_pixels(160, 400);
        let mut selection_bounds = (160_u32, 400_u32, 0_u32, 0_u32);
        for y in 0..400 {
            for x in 0..160 {
                let index = ((y * 160 + x) * 4) as usize;
                if selected_pixels[index..index + 3] == [0, 0, 255] {
                    selection_bounds.0 = selection_bounds.0.min(x);
                    selection_bounds.1 = selection_bounds.1.min(y);
                    selection_bounds.2 = selection_bounds.2.max(x);
                    selection_bounds.3 = selection_bounds.3.max(y);
                }
            }
        }
        assert!(
            selection_bounds.3 - selection_bounds.1 > selection_bounds.2 - selection_bounds.0,
            "{writing_mode} selection: {selection_bounds:?}"
        );
    }
}

pub(super) fn line_count_with_rule(
    property: &str,
    value: &str,
    text: String,
    viewport_width: u32,
) -> usize {
    let nodes = [UiNode::Text(text)];
    let stylesheet = [StyleRule {
        selectors: vec!["body".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: property.to_owned(),
            value: value.to_owned(),
        }],
    }];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(viewport_width, 300);
    let line_count = page.buffer.layout_runs().count();
    line_count
}

pub(super) fn line_count_without_rule(text: String, viewport_width: u32) -> usize {
    let nodes = [UiNode::Text(text)];
    let mut page = PageRasterizer::new(&nodes, &[]);
    page.prepare_layout(viewport_width, 300);
    page.buffer.layout_runs().count()
}
