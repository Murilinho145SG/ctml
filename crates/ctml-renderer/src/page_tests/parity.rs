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

macro_rules! parity_test_with_tolerance {
    ($name:ident, $case:literal, $tolerance:expr) => {
        #[test]
        fn $name() {
            run_parity_case_with_tolerance(
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
                false,
                $tolerance,
            );
        }
    };
}

macro_rules! exact_parity_test {
    ($name:ident, $case:literal) => {
        #[test]
        fn $name() {
            run_exact_parity_case(
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
            );
        }
    };
}

macro_rules! alpha_text_color_test {
    ($case:literal) => {
        #[test]
        fn alpha_text_color_matches_chromium_at_full_coverage() {
            let native_markup = include_str!(concat!(
                "../../tests/fixtures/parity/",
                $case,
                "/native.ctml"
            ));
            let browser_markup = include_str!(concat!(
                "../../tests/fixtures/parity/",
                $case,
                "/browser.html"
            ));
            let css = include_str!(concat!("../../tests/fixtures/parity/", $case, "/style.css"));
            run_parity_case($case, native_markup, browser_markup, css, false);

            let artifact_dir = parity_artifact_dir().join($case);
            let native = read_rgba_png(&artifact_dir.join("ctml.png"));
            let browser = read_rgba_png(&artifact_dir.join("browser.png"));
            let expected = [153, 178, 204];
            assert!(
                native
                    .pixels
                    .chunks_exact(4)
                    .zip(browser.pixels.chunks_exact(4))
                    .any(|(native, browser)| native[..3] == expected && browser[..3] == expected),
                "no fully covered text pixel matched the expected alpha-blended color in both screenshots"
            );
        }
    };
}

parity_test!(page_colors_match_chromium, "colors", false);
exact_parity_test!(background_color_matches_chromium, "background-color");
exact_parity_test!(
    alpha_background_color_matches_chromium,
    "background-color-alpha"
);
exact_parity_test!(color_current_color_matches_chromium, "color-current-color");
exact_parity_test!(
    alpha_color_current_color_matches_chromium,
    "color-current-color-alpha"
);
alpha_text_color_test!("color-alpha-text");
parity_test!(transparent_colors_match_chromium, "transparent", false);
parity_test!(font_weight_matches_chromium, "font-weight", false);
parity_test!(font_family_matches_chromium, "font-family", false);
parity_test!(
    inter_font_family_matches_chromium,
    "font-family-inter",
    false
);
parity_test!(
    font_family_fallback_matches_chromium,
    "font-family-fallback",
    false
);
parity_test!(font_style_matches_chromium, "font-style", false);
parity_test_with_tolerance!(
    font_stretch_values_match_chromium,
    "font-stretch",
    super::MAX_FONT_STRETCH_DIFFERING_PIXEL_RATIO
);
parity_test!(line_height_values_match_chromium, "line-height", false);
parity_test_with_tolerance!(
    normal_line_height_matches_chromium,
    "line-height-normal",
    super::MAX_NORMAL_LINE_HEIGHT_DIFFERING_PIXEL_RATIO
);
parity_test!(font_size_percentages_match_chromium, "font-size", false);
parity_test!(
    relative_font_size_keywords_match_chromium,
    "font-size-relative-keywords",
    false
);
parity_test!(
    absolute_font_size_keywords_match_chromium,
    "font-size-keywords",
    false
);
parity_test!(absolute_font_sizes_match_chromium, "font-size-units", false);
parity_test!(
    font_relative_sizes_match_chromium,
    "font-relative-sizes",
    false
);
parity_test!(font_features_match_chromium, "font-features", false);
parity_test_with_tolerance!(
    font_variant_shorthand_matches_chromium,
    "font-variant-shorthand",
    super::MAX_FONT_VARIANT_DIFFERING_PIXEL_RATIO
);
parity_test!(additional_font_caps_match_chromium, "font-caps", false);
parity_test!(font_positions_match_chromium, "font-position", false);
parity_test!(text_decoration_matches_chromium, "text-decoration", false);
parity_test_with_tolerance!(
    text_decoration_styles_match_chromium,
    "text-decoration-style",
    super::MAX_TEXT_DECORATION_STYLE_DIFFERING_PIXEL_RATIO
);
parity_test!(
    text_decoration_lines_match_chromium,
    "text-decoration-lines",
    false
);
parity_test!(
    text_decoration_skip_ink_matches_chromium,
    "text-decoration-skip-ink",
    false
);
parity_test!(
    text_decoration_thickness_matches_chromium,
    "text-decoration-thickness",
    false
);
parity_test!(
    text_underline_offset_matches_chromium,
    "text-underline-offset",
    false
);
parity_test!(child_selector_matches_chromium, "child-selector", false);
parity_test!(
    inline_style_attribute_matches_chromium,
    "inline-style",
    false
);
parity_test_with_tolerance!(
    attribute_selectors_match_chromium,
    "attribute-selectors",
    super::MAX_ATTRIBUTE_SELECTOR_DIFFERING_PIXEL_RATIO
);
parity_test!(selector_matrix_matches_chromium, "selector-matrix", false);
parity_test_with_tolerance!(
    sibling_selectors_match_chromium,
    "sibling-selectors",
    0.0007
);
parity_test!(
    structural_selectors_match_chromium,
    "structural-selectors",
    false
);
parity_test_with_tolerance!(has_selectors_match_chromium, "has-selectors", 0.006);
parity_test!(text_alignment_matches_chromium, "text-alignment", false);
parity_test!(
    text_alignment_values_match_chromium,
    "text-alignment-values",
    false
);
parity_test!(letter_spacing_matches_chromium, "letter-spacing", false);
parity_test!(text_transform_matches_chromium, "text-transform", false);
parity_test!(font_kerning_values_match_chromium, "font-kerning", false);
parity_test!(hidden_cursor_matches_chromium, "cursor", true);

#[test]
fn transparent_nested_text_resolves_to_zero_alpha() {
    let document = parse(include_str!(
        "../../tests/fixtures/parity/transparent/native.ctml"
    ))
    .expect("transparent CTML fixture should parse");
    let body = document.body.iter().map(ui_node).collect::<Vec<_>>();
    let stylesheet = parse_stylesheet(include_str!(
        "../../tests/fixtures/parity/transparent/style.css"
    ))
    .expect("transparent CSS fixture should parse")
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
    let rasterizer = PageRasterizer::new(&body, &stylesheet);
    let clear_run = rasterizer
        .runs
        .iter()
        .find(|run| run.text.contains('B'))
        .expect("transparent span should produce a text run");

    assert_eq!(clear_run.style.color, [0, 0, 0, 0]);

    let clear_document = parse(
        "<headers><application name=\"Clear text\" desktop /></headers><body><p class=\"clear\">B</p></body>",
    )
    .expect("transparent text CTML should parse");
    let clear_body = clear_document.body.iter().map(ui_node).collect::<Vec<_>>();
    let mut clear_rasterizer = PageRasterizer::new(&clear_body, &stylesheet);
    let pixels = clear_rasterizer.snapshot_pixels(WIDTH, HEIGHT);
    let visible_pixel_count = pixels
        .chunks_exact(4)
        .filter(|pixel| pixel[..3] != [255, 255, 255])
        .count();
    assert_eq!(
        visible_pixel_count, 0,
        "transparent text must not paint any pixels"
    );
}

fn run_parity_case(
    name: &str,
    native_markup: &str,
    browser_markup: &str,
    css: &str,
    expect_hidden_cursor: bool,
) {
    run_parity_case_with_tolerance(
        name,
        native_markup,
        browser_markup,
        css,
        expect_hidden_cursor,
        MAX_DIFFERING_PIXEL_RATIO,
    );
}

fn run_parity_case_with_tolerance(
    name: &str,
    native_markup: &str,
    browser_markup: &str,
    css: &str,
    expect_hidden_cursor: bool,
    max_differing_pixel_ratio: f32,
) {
    run_parity_case_with_mode(
        name,
        native_markup,
        browser_markup,
        css,
        expect_hidden_cursor,
        false,
        max_differing_pixel_ratio,
    );
}

fn run_exact_parity_case(name: &str, native_markup: &str, browser_markup: &str, css: &str) {
    run_parity_case_with_mode(name, native_markup, browser_markup, css, false, true, 0.0);
}

fn run_parity_case_with_mode(
    name: &str,
    native_markup: &str,
    browser_markup: &str,
    css: &str,
    expect_hidden_cursor: bool,
    require_exact_match: bool,
    max_differing_pixel_ratio: f32,
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

    let (diff_pixels, exact_differences, diff_image_pixels, max_channel_delta) =
        compare_pixels(&native.pixels, &browser.pixels);
    write_rgba_png(&diff_image, WIDTH, HEIGHT, &diff_image_pixels);
    let pixel_count = (WIDTH * HEIGHT) as usize;
    let diff_ratio = diff_pixels as f32 / pixel_count as f32;
    eprintln!(
        "{}",
        colorize(
            &format!(
                "CSS parity [{name}]: {exact_differences}/{pixel_count} pixels differ exactly; {diff_pixels} exceed channel tolerance ({:.3}%), max channel delta {max_channel_delta}. Artifacts: {}",
                diff_ratio * 100.0,
                artifact_dir.display()
            ),
            "\x1b[36m"
        )
    );
    if require_exact_match {
        assert_eq!(
            exact_differences,
            0,
            "CSS parity case {name} differs in {exact_differences} pixels; inspect {} and {}",
            native_image.display(),
            browser_image.display()
        );
        log_success(&format!("[{name}] Pixels match Chromium exactly"));
        return;
    }
    assert!(
        diff_ratio <= max_differing_pixel_ratio,
        "CSS parity case {name} differs in {:.3}% of pixels (limit {:.3}%); inspect {} and {}",
        diff_ratio * 100.0,
        max_differing_pixel_ratio * 100.0,
        native_image.display(),
        browser_image.display()
    );
    log_success(&format!(
        "[{name}] Pixel comparison passed (limit {:.2}%)",
        max_differing_pixel_ratio * 100.0
    ));
}
