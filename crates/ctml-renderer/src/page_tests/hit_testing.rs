use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};
use winit::window::CursorIcon;

fn page(text: &str, css: &str) -> PageRasterizer {
    let nodes = [UiNode::Element {
        tag: "p".into(),
        class: None,
        id: None,
        attributes: vec![],
        children: vec![UiNode::Text(text.into())],
    }];
    styled_page(&nodes, css)
}

fn styled_page(nodes: &[UiNode], css: &str) -> PageRasterizer {
    let stylesheet = ctml_css::parse_stylesheet(css).unwrap();
    let rules = stylesheet
        .rules
        .into_iter()
        .map(|rule| StyleRule {
            selectors: rule.selectors,
            layer: rule.layer,
            declarations: rule
                .declarations
                .into_iter()
                .map(|d| StyleDeclaration {
                    property: d.property,
                    value: d.value,
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    PageRasterizer::new(nodes, &rules)
}

#[test]
fn painted_text_is_inside_the_pointer_target() {
    for css in [
        "p { font-size: 44px; line-height: 1.2; padding: 30px; cursor: crosshair; }",
        "p { font-size: 32px; line-height: 1.5; text-align: center; letter-spacing: 3px; cursor: crosshair; }",
        "p { font-size: 28px; line-height: 2; text-align: right; cursor: crosshair; }",
    ] {
        let mut page = page("Hit the actual text", css);
        let pixels = page.snapshot_pixels(500, 180);
        let mut ink = 0;
        let mut missed = Vec::new();
        for (index, pixel) in pixels.chunks_exact(4).enumerate() {
            if pixel[..3].iter().all(|channel| *channel < 100) {
                ink += 1;
                let (x, y) = (index % 500, index / 500);
                if page.cursor_at(x as f64 + 0.5, y as f64 + 0.5).0 != CursorIcon::Crosshair {
                    missed.push((x, y));
                }
            }
        }
        assert!(ink > 0, "the fixture must paint text");
        assert!(
            missed.is_empty(),
            "painted text must remain interactive: {css}: {missed:?}"
        );
    }
}

#[test]
fn css_spacing_and_alignment_do_not_make_empty_space_selectable() {
    for css in [
        "p { padding: 30px; cursor: crosshair; }",
        "p { margin-left: 60px; padding-left: 20px; text-indent: 25px; cursor: crosshair; }",
        "p { margin-left: 10%; padding-left: 5%; cursor: crosshair; }",
        "p { text-align: center; cursor: crosshair; }",
        "p { text-align: right; cursor: crosshair; }",
    ] {
        let mut page = page("Selectable text", css);
        for width in [400, 640, 320] {
            page.snapshot_pixels(width, 160);
            let (left, right, y, start, end) = page
                .layout_runs()
                .find_map(|run| {
                    let first = run.glyphs.iter().find(|glyph| glyph.w > 0.0)?;
                    let last = run.glyphs.iter().rev().find(|glyph| glyph.w > 0.0)?;
                    Some((
                        first.x + super::super::PAGE_PADDING,
                        last.x + last.w + super::super::PAGE_PADDING,
                        run.line_top + run.line_height * 0.5 + super::super::PAGE_PADDING,
                        first.start,
                        last.end,
                    ))
                })
                .unwrap();
            assert_eq!(
                page.cursor_at((left - 2.0) as f64, y as f64).0,
                CursorIcon::Default,
                "{css}"
            );
            page.begin_selection((left - 2.0) as f64, y as f64);
            assert!(
                page.selection_anchor.is_none(),
                "padding and alignment gaps must not start a selection: {css}"
            );
            page.begin_selection((left + 0.5) as f64, y as f64);
            assert_eq!(page.selection_anchor.unwrap().index, start, "{css}");
            page.extend_selection((right + 30.0) as f64, y as f64);
            assert_eq!(page.selection.unwrap().1.index, end, "{css}");
            page.end_selection();
        }
    }
}

#[test]
fn cursor_style_uses_the_glyph_under_the_pointer_at_inline_boundaries() {
    let nodes = vec![UiNode::Element {
        tag: "p".into(),
        class: None,
        id: None,
        attributes: vec![],
        children: [("first", "A"), ("second", "B")]
            .into_iter()
            .map(|(class, text)| UiNode::Element {
                tag: "span".into(),
                class: Some(class.into()),
                id: None,
                attributes: vec![],
                children: vec![UiNode::Text(text.into())],
            })
            .collect(),
    }];
    let mut page = styled_page(
        &nodes,
        "p { font-size: 32px; } .first {cursor: crosshair;} .second {cursor: pointer;}",
    );
    page.snapshot_pixels(200, 80);
    let (first_x, second_x, y) = page
        .layout_runs()
        .find_map(|run| {
            let first = run.glyphs.first()?;
            let second = run.glyphs.get(1)?;
            Some((
                first.x + first.w * 0.8 + super::super::PAGE_PADDING,
                second.x + second.w * 0.2 + super::super::PAGE_PADDING,
                run.line_top + run.line_height * 0.5 + super::super::PAGE_PADDING,
            ))
        })
        .unwrap();
    assert_eq!(
        page.cursor_at(first_x as f64, y as f64).0,
        CursorIcon::Crosshair
    );
    assert_eq!(
        page.cursor_at(second_x as f64, y as f64).0,
        CursorIcon::Pointer
    );
}

#[test]
fn selecting_part_of_a_word_does_not_move_the_painted_characters() {
    for (text, start, end) in [("AVATAR office affinity", 1, 4), ("العربية العربية", 2, 8)]
    {
        let mut page = page(
            text,
            "p { font-family: serif; font-size: 44px; letter-spacing: 1px; word-spacing: 9px; } p::selection {color: red; background-color: transparent;}",
        );
        let baseline = page.snapshot_pixels(700, 100);
        page.selection = Some((
            cosmic_text::Cursor::new(0, start),
            cosmic_text::Cursor::new(0, end),
        ));
        let selected = page.render_pixels(700, 100);
        // Skia applies color-dependent antialiasing contrast. Compare solid ink
        // against empty space, allowing edge coverage to change with text color.
        let moved = baseline
            .chunks_exact(4)
            .zip(selected.chunks_exact(4))
            .filter(|(left, right)| left[1].abs_diff(right[1]) > 96)
            .count();
        assert_eq!(
            moved, 0,
            "selection color must not change glyph placement: {text}"
        );
    }
}

#[test]
fn letter_spacing_does_not_offset_the_painted_glyph_from_its_layout() {
    for spacing in [0, 2, 8, -2] {
        let mut page = page(
            "H",
            &format!(
                "p {{ font-size: 40px; font-family: monospace; letter-spacing: {spacing}px; }}"
            ),
        );
        let pixels = page.snapshot_pixels(200, 100);
        let glyph = page.layout_runs().next().unwrap().glyphs[0].clone();
        let typeface = crate::typeface_cache::load(&page.font_system, glyph.font_id).unwrap();
        let mut font = skia_safe::Font::from_typeface(typeface, glyph.font_size);
        font.set_hinting(skia_safe::FontHinting::None);
        font.set_linear_metrics(true);
        let mut bounds = [skia_safe::Rect::default()];
        font.get_bounds(
            &[skia_safe::GlyphId::from(glyph.glyph_id)],
            &mut bounds,
            None,
        );
        let expected_left = glyph.x
            + glyph.x_offset * glyph.font_size
            + super::super::PAGE_PADDING
            + bounds[0].left;
        let painted_left = pixels
            .chunks_exact(4)
            .enumerate()
            .filter(|(_, pixel)| pixel[0] < 250)
            .map(|(index, _)| index % 200)
            .min()
            .unwrap() as f32;
        assert!(
            (painted_left - expected_left).abs() <= 1.0,
            "{spacing}px tracking: layout ink starts at {expected_left}, painting starts at {painted_left}"
        );
    }
}
