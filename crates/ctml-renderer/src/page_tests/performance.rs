use super::ui_node;
use crate::{StyleDeclaration, StyleRule, page::PageRasterizer};
use std::{hint::black_box, time::Instant};

#[test]
#[ignore = "manual rendering and interaction benchmark"]
fn native_page_rendering_cost() {
    let document = ctml_compiler::parse(include_str!(
        "../../../../examples/native-app/src/main.ctml"
    ))
    .unwrap();
    let nodes = document.body.iter().map(ui_node).collect::<Vec<_>>();
    let css = concat!(
        include_str!("../../../../examples/native-app/src/base.css"),
        "\n",
        include_str!("../../../../examples/native-app/src/main.css")
    );
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
                .map(|declaration| StyleDeclaration {
                    property: declaration.property,
                    value: declaration.value,
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let mut page = PageRasterizer::new(&nodes, &rules);
    let start = Instant::now();
    black_box(page.snapshot_pixels(1040, 800));
    eprintln!("initial layout + render: {:?}", start.elapsed());
    let start = Instant::now();
    for _ in 0..10 {
        let pixels = page.render_pixels(1040, 800);
        black_box(&pixels);
        page.frame_pixels = pixels;
    }
    eprintln!("redraw mean: {:?}", start.elapsed() / 10);
    let start = Instant::now();
    for index in 0..100 {
        page.update_hover(60.0 + index as f64, 50.0);
        black_box(page.cursor_at(60.0 + index as f64, 50.0));
    }
    eprintln!("pointer event mean: {:?}", start.elapsed() / 100);
    let point = page
        .layout_runs()
        .find_map(|run| {
            let glyph = run.glyphs.first()?;
            Some((glyph.x as f64 + 9.0, run.line_top as f64 + 9.0))
        })
        .unwrap();
    page.begin_selection(point.0, point.1);
    let start = Instant::now();
    for index in 1..11 {
        page.extend_selection(point.0 + index as f64 * 10.0, point.1);
        let pixels = page.render_pixels(1040, 800);
        black_box(&pixels);
        page.frame_pixels = pixels;
    }
    page.end_selection();
    eprintln!("selection + redraw mean: {:?}", start.elapsed() / 10);
    let start = Instant::now();
    for width in (1040..1140).step_by(10) {
        black_box(page.snapshot_pixels(width, 800));
    }
    eprintln!("resize mean: {:?}", start.elapsed() / 10);
    report_memory("after redraws");
    for (label, sizes) in [
        (
            "first growth",
            vec![(1280, 900), (1600, 1000), (1920, 1080)],
        ),
        ("shrink", vec![(1040, 800)]),
        (
            "repeat",
            vec![(1280, 900), (1600, 1000), (1920, 1080), (1040, 800)],
        ),
    ] {
        for (width, height) in sizes {
            black_box(page.snapshot_pixels(width, height));
        }
        report_memory(label);
    }
}

fn report_memory(label: &str) {
    #[cfg(target_os = "linux")]
    eprintln!(
        "{label}: {} | Skia glyphs: {} bytes, resources: {} bytes",
        std::fs::read_to_string("/proc/self/status")
            .unwrap()
            .lines()
            .filter(|line| line.starts_with("VmRSS:") || line.starts_with("VmHWM:"))
            .collect::<Vec<_>>()
            .join(", "),
        skia_safe::graphics::font_cache_used(),
        skia_safe::graphics::resource_cache_total_bytes_used()
    );
}
