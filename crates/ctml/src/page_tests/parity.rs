use std::fs;

use ctml_compiler::{AttributeValue, parse, parse_stylesheet};
use winit::window::CursorIcon;

use super::super::PageRasterizer;
use super::progress::{colorize, log_step, log_success};
use super::{
    HEIGHT, MAX_DIFFERING_PIXEL_RATIO, WIDTH, capture_browser, compare_pixels, configured_browser,
    parity_artifact_dir, read_rgba_png, ui_node, write_rgba_png,
};
use crate::{StyleDeclaration, StyleRule};

macro_rules! parity_test {
    ($name:ident, $case:literal, $expect_hidden_cursor:literal) => {
        #[test]
        fn $name() {
            run_parity_case(
                $case,
                include_str!(concat!(
                    "../../tests/fixtures/parity/",
                    $case,
                    "/native.ctml"
                )),
                include_str!(concat!(
                    "../../tests/fixtures/parity/",
                    $case,
                    "/browser.html"
                )),
                include_str!(concat!("../../tests/fixtures/parity/", $case, "/style.css")),
                $expect_hidden_cursor,
            );
        }
    };
}

parity_test!(page_colors_match_chromium, "colors", false);
parity_test!(transparent_colors_match_chromium, "transparent", false);
parity_test!(font_weight_matches_chromium, "font-weight", false);
parity_test!(font_size_percentages_match_chromium, "font-size", false);
parity_test!(absolute_font_sizes_match_chromium, "font-size-units", false);
parity_test!(
    font_relative_sizes_match_chromium,
    "font-relative-sizes",
    false
);
parity_test!(font_features_match_chromium, "font-features", false);
parity_test!(font_positions_match_chromium, "font-position", false);
parity_test!(text_decoration_matches_chromium, "text-decoration", false);
parity_test!(child_selector_matches_chromium, "child-selector", false);
parity_test!(text_alignment_matches_chromium, "text-alignment", false);
parity_test!(hidden_cursor_matches_chromium, "cursor", true);

fn run_parity_case(
    name: &str,
    native_markup: &str,
    browser_markup: &str,
    css: &str,
    expect_hidden_cursor: bool,
) {
    log_step(&format!("[{name}] Parsing CTML, HTML, and CSS fixtures"));
    let document = parse(native_markup).expect("CTML parity fixture should parse");
    let stylesheet_link = document
        .stylesheet()
        .and_then(|element| element.attribute("href"))
        .and_then(AttributeValue::as_str);
    assert_eq!(stylesheet_link, Some("style.css"), "parity case {name}");

    let body = document.body.iter().map(ui_node).collect::<Vec<_>>();
    let stylesheet = parse_stylesheet(css)
        .expect("CSS parity fixture should parse")
        .rules
        .into_iter()
        .map(|rule| StyleRule {
            selectors: rule.selectors,
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

    log_step(&format!("[{name}] Rendering CTML pixels"));
    let mut rasterizer = PageRasterizer::new(&body, &stylesheet);
    let native_pixels = rasterizer.snapshot_pixels(WIDTH, HEIGHT);
    if expect_hidden_cursor {
        assert_eq!(
            rasterizer.cursor_at(0.0, 0.0),
            (CursorIcon::Default, false),
            "parity case {name}: cursor: none should hide the native cursor"
        );
    }

    let artifact_dir = parity_artifact_dir().join(name);
    fs::create_dir_all(&artifact_dir).expect("create parity case artifact directory");
    let native_image = artifact_dir.join("ctml.png");
    let browser_image = artifact_dir.join("browser.png");
    let diff_image = artifact_dir.join("diff.png");
    write_rgba_png(&native_image, WIDTH, HEIGHT, &native_pixels);
    log_success(&format!(
        "[{name}] Saved native screenshot to {}",
        native_image.display()
    ));

    let browser = configured_browser();
    let browser_dir = artifact_dir.join(format!("browser-input-{}", std::process::id()));
    fs::create_dir_all(&browser_dir).expect("create browser fixture directory");
    fs::write(browser_dir.join("index.html"), browser_markup).expect("write HTML fixture");
    fs::write(browser_dir.join("style.css"), css).expect("write CSS fixture");
    log_step(&format!("[{name}] Capturing HTML in Chromium"));
    capture_browser(&browser, &browser_dir, &browser_image);

    let native = read_rgba_png(&native_image);
    let browser = read_rgba_png(&browser_image);
    assert_eq!(
        native.dimensions,
        (WIDTH, HEIGHT),
        "CTML image size: {name}"
    );
    assert_eq!(
        browser.dimensions,
        (WIDTH, HEIGHT),
        "Chromium image size: {name}"
    );

    let (diff_pixels, diff_image_pixels, max_channel_delta) =
        compare_pixels(&native.pixels, &browser.pixels);
    write_rgba_png(&diff_image, WIDTH, HEIGHT, &diff_image_pixels);
    let pixel_count = (WIDTH * HEIGHT) as usize;
    let diff_ratio = diff_pixels as f32 / pixel_count as f32;
    eprintln!(
        "{}",
        colorize(
            &format!(
                "CSS parity [{name}]: {diff_pixels}/{pixel_count} pixels differ ({:.3}%), max channel delta {max_channel_delta}. Artifacts: {}",
                diff_ratio * 100.0,
                artifact_dir.display()
            ),
            "\x1b[36m"
        )
    );
    assert!(
        diff_ratio <= MAX_DIFFERING_PIXEL_RATIO,
        "CSS parity case {name} differs in {:.3}% of pixels (limit {:.3}%); inspect {} and {}",
        diff_ratio * 100.0,
        MAX_DIFFERING_PIXEL_RATIO * 100.0,
        native_image.display(),
        browser_image.display()
    );
    log_success(&format!(
        "[{name}] Pixel comparison passed (limit {:.2}%)",
        MAX_DIFFERING_PIXEL_RATIO * 100.0
    ));
}
