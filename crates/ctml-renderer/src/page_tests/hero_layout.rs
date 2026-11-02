use crate::{StyleDeclaration, StyleRule, page::PageRasterizer};
use ctml_compiler::{parse, parse_stylesheet};

#[test]
fn preview_footer_bottom_padding_stays_inside_its_flex_panel() {
    let page = showcase_page();
    let panel_id = page
        .runs
        .iter()
        .flat_map(|run| &run.style.layout_sizes)
        .find(|size| size.background_layers.contains(&[34, 34, 37, 255]))
        .expect("preview panel style should be present")
        .node_id;
    let footer_id = page
        .runs
        .iter()
        .flat_map(|run| &run.style.layout_end_sizes)
        .find(|size| {
            size.border_style[0] == crate::style::BorderLineStyle::Solid
                && size.padding[0].as_deref() == Some("15px")
        })
        .expect("preview footer style should be present")
        .node_id;
    let panel_box = page
        .overflow_geometry
        .get(&panel_id)
        .expect("preview panel should have border geometry")
        .border_box;
    let footer_box = page
        .overflow_geometry
        .get(&footer_id)
        .expect("preview footer should have border geometry")
        .border_box;
    let bottom_inset = panel_box.bottom - footer_box.bottom;
    assert!(
        (-0.5..=2.5).contains(&bottom_inset),
        "footer should fit flush inside the preview panel, got {bottom_inset}px bottom inset"
    );
}

#[test]
fn empty_hero_rule_starts_after_its_top_margin() {
    let page = showcase_page();
    let description_line = page
        .layout_runs()
        .filter(|line| line.text.contains("Designed to feel effortless."))
        .last()
        .expect("description should have a final line");
    let description_bottom = description_line.line_top + description_line.line_height;
    let rule_id = page
        .runs
        .iter()
        .flat_map(|run| &run.style.layout_sizes)
        .find(|size| size.background_layers.contains(&[54, 54, 58, 255]))
        .expect("hero rule background should be present")
        .node_id;
    let rule_top = page
        .overflow_geometry
        .get(&rule_id)
        .expect("hero rule should have geometry")
        .border_box;
    let technical_top = page
        .layout_runs()
        .find(|line| line.text.contains("01 /"))
        .expect("technical information should follow the hero rule")
        .line_top;
    let gap = rule_top.top - description_bottom;
    assert!(
        (39.0..=42.0).contains(&gap),
        "hero rule should preserve its 40px top margin, got {gap}px"
    );
    let trailing_gap = technical_top - rule_top.bottom;
    assert!(
        (20.0..=30.0).contains(&trailing_gap),
        "technical information should preserve the rule's 24px bottom margin, got {trailing_gap}px"
    );
}

#[test]
fn footer_space_between_respects_both_horizontal_padding_edges() {
    let page = showcase_page();
    let left_text = text_bounds(&page, "CTML © 2026");
    let right_text = text_bounds(&page, "DEVELOPED WITH RUST / EXPERIMENTAL");

    assert!(
        (41.0..=43.0).contains(&left_text.0),
        "footer's first item should start at its 42px inset, got {}px",
        left_text.0
    );
    assert!(
        (40.0..=45.0).contains(&(1920.0 - right_text.1)),
        "footer's last item should end at its 42px inset, got {}px",
        1920.0 - right_text.1
    );
}

#[test]
fn feature_separators_meet_the_strip_bottom_border() {
    let page = showcase_page();
    let strip_id = page
        .runs
        .iter()
        .flat_map(|run| &run.style.layout_sizes)
        .find(|size| {
            size.border_style[0] == crate::style::BorderLineStyle::Solid
                && size.border_style[2] == crate::style::BorderLineStyle::Solid
                && size.border_style[1] == crate::style::BorderLineStyle::None
                && size.border_style[3] == crate::style::BorderLineStyle::None
        })
        .expect("feature strip border style should be present")
        .node_id;
    let feature_id = page
        .runs
        .iter()
        .flat_map(|run| &run.style.layout_sizes)
        .find(|size| {
            size.min_height.as_deref() == Some("143px")
                && size.border_style[1] == crate::style::BorderLineStyle::Solid
        })
        .expect("feature separator border style should be present")
        .node_id;
    let strip_bottom = page
        .overflow_geometry
        .get(&strip_id)
        .expect("feature strip should have border geometry")
        .border_box
        .bottom;
    let separator_bottom = page
        .overflow_geometry
        .get(&feature_id)
        .expect("feature separator should have border geometry")
        .border_box
        .bottom;
    let gap = strip_bottom - separator_bottom;
    assert!(
        (-0.5..=2.5).contains(&gap),
        "feature separators should meet the strip bottom border, got {gap}px gap"
    );
}

#[test]
fn feature_separators_use_subpixel_coverage_for_one_pixel_strokes() {
    let mut page = showcase_page();
    let feature_id = page
        .runs
        .iter()
        .flat_map(|run| &run.style.layout_sizes)
        .find(|size| {
            size.min_height.as_deref() == Some("143px")
                && size.border_style[1] == crate::style::BorderLineStyle::Solid
        })
        .expect("feature separator border style should be present")
        .node_id;
    let pixels = page.snapshot_pixels(1920, 1024);
    let feature_box = page
        .overflow_geometry
        .get(&feature_id)
        .expect("feature separator should have border geometry")
        .border_box;
    let x = (feature_box.right - 1.0).floor() as usize;
    let y = (feature_box.top + 70.0).floor() as usize;
    let pixel = |x: usize| {
        let offset = (y * 1920 + x) * 4;
        [pixels[offset], pixels[offset + 1], pixels[offset + 2]]
    };
    let left_coverage = pixel(x);
    let right_coverage = pixel(x + 1);

    assert!(
        left_coverage[0] > 27 && left_coverage[0] < 52,
        "fractional border should partially cover its first pixel, got {left_coverage:?}"
    );
    assert!(
        right_coverage[0] > 27 && right_coverage[0] < 52,
        "fractional border should partially cover its second pixel, got {right_coverage:?}"
    );
    assert_eq!(pixel(x - 1), [27, 27, 29]);
    assert_eq!(pixel(x + 2), [27, 27, 29]);
}

#[test]
fn distributed_row_flow_matches_the_hero_minimum_height() {
    let mut page = showcase_page_at(1904, 1032);
    let pixels = page.snapshot_pixels(1904, 1032);
    let feature_top = (650..760)
        .find(|y| {
            let offset = (y * 1904 + 100) * 4;
            pixels[offset] > 40 && pixels[offset + 1] > 40 && pixels[offset + 2] > 40
        })
        .expect("feature strip top border should be visible");

    assert!(
        (694..=708).contains(&feature_top),
        "feature strip should match the browser's 61vh hero boundary near y=701, got y={feature_top}"
    );
}

fn text_bounds(page: &PageRasterizer, text: &str) -> (f32, f32) {
    let line = page
        .layout_runs()
        .find(|line| line.text.contains(text))
        .expect("footer text should be present");
    let start = line.text.find(text).expect("footer text should be present");
    let end = start + text.len();
    line.glyphs
        .iter()
        .filter(|glyph| glyph.start >= start && glyph.end <= end)
        .fold(
            (f32::INFINITY, f32::NEG_INFINITY),
            |(left, right), glyph| (left.min(glyph.x), right.max(glyph.x + glyph.w)),
        )
}

fn showcase_page() -> PageRasterizer {
    showcase_page_at(1920, 1024)
}

fn showcase_page_at(width: u32, height: u32) -> PageRasterizer {
    let document = parse(include_str!(
        "../../../../examples/native-app/src/main.ctml"
    ))
    .expect("showcase CTML should parse");
    let nodes = document.body.iter().map(super::ui_node).collect::<Vec<_>>();
    let css = format!(
        "{}\n{}",
        include_str!("../../../../examples/native-app/src/base.css"),
        include_str!("../../../../examples/native-app/src/main.css")
    );
    let stylesheet = parse_stylesheet(&css)
        .expect("showcase CSS should parse")
        .rules
        .into_iter()
        .map(|rule| StyleRule {
            selectors: rule.selectors,
            layer: rule.layer,
            declarations: rule
                .declarations
                .into_iter()
                .map(|declaration| StyleDeclaration {
                    property: declaration.property,
                    value: declaration.value,
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(width, height);
    page
}
