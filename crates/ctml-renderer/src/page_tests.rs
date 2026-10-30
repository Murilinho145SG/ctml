use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use chromiumoxide_fetcher::{BrowserFetcher, BrowserFetcherOptions, BrowserKind, BrowserVersion};
use ctml_compiler::{AttributeValue, Node};
use url::Url;

use crate::UiNode;
use progress::{ProgressBar, log_notice, log_step, log_success};

#[path = "page_tests/progress.rs"]
mod progress;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;
const CHANNEL_TOLERANCE: u8 = 16;
const MAX_DIFFERING_PIXEL_RATIO: f32 = 0.0005;
const MAX_FONT_STRETCH_DIFFERING_PIXEL_RATIO: f32 = 0.00055;
const MAX_FONT_VARIANT_DIFFERING_PIXEL_RATIO: f32 = 0.0027;
const MAX_NORMAL_LINE_HEIGHT_DIFFERING_PIXEL_RATIO: f32 = 0.0048;
const MAX_TEXT_DECORATION_STYLE_DIFFERING_PIXEL_RATIO: f32 = 0.0011;
const MAX_ATTRIBUTE_SELECTOR_DIFFERING_PIXEL_RATIO: f32 = 0.0125;
const CHROMIUM_VERSION: &str = "156.0.8078.47";
const CHROMIUM_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);
const CHROMIUM_CAPTURE_TIMEOUT: Duration = Duration::from_secs(30);

#[path = "page_tests/aspect_ratio.rs"]
mod aspect_ratio;
#[path = "page_tests/background_attachment.rs"]
mod background_attachment;
#[path = "page_tests/background_box.rs"]
mod background_box;
#[path = "page_tests/background_images.rs"]
mod background_images;
#[path = "page_tests/backgrounds.rs"]
mod backgrounds;
#[path = "page_tests/blend_modes.rs"]
mod blend_modes;
#[path = "page_tests/border_images.rs"]
mod border_images;
#[path = "page_tests/border_radius.rs"]
mod border_radius;
#[path = "page_tests/borders.rs"]
mod borders;
#[path = "page_tests/box_model.rs"]
mod box_model;
#[path = "page_tests/box_shadows.rs"]
mod box_shadows;
#[path = "page_tests/conic_gradients.rs"]
mod conic_gradients;
#[path = "page_tests/containment.rs"]
mod containment;
#[path = "page_tests/css_math.rs"]
mod css_math;
#[path = "page_tests/element_opacity.rs"]
mod element_opacity;
#[path = "page_tests/first_letter_border_image.rs"]
mod first_letter_border_image;
#[path = "page_tests/first_letter_logical_box.rs"]
mod first_letter_logical_box;
#[path = "page_tests/first_letter_transform.rs"]
mod first_letter_transform;
#[path = "page_tests/first_line_border_image.rs"]
mod first_line_border_image;
#[path = "page_tests/first_line_box_model.rs"]
mod first_line_box_model;
#[path = "page_tests/first_line_spacing.rs"]
mod first_line_spacing;
#[path = "page_tests/first_line_transform.rs"]
mod first_line_transform;
#[path = "page_tests/font_relative_units.rs"]
mod font_relative_units;
#[path = "page_tests/hyphens.rs"]
mod hyphens;
#[path = "page_tests/inline_boxes.rs"]
mod inline_boxes;
#[path = "page_tests/interactions.rs"]
mod interactions;
#[path = "page_tests/linear_gradients.rs"]
mod linear_gradients;
#[path = "page_tests/outline.rs"]
mod outline;
#[path = "page_tests/parity.rs"]
mod parity;
#[path = "page_tests/pseudo.rs"]
mod pseudo;
#[path = "page_tests/pseudo_first_letter.rs"]
mod pseudo_first_letter;
#[path = "page_tests/pseudo_opacity.rs"]
mod pseudo_opacity;
#[path = "page_tests/pseudo_vertical_align.rs"]
mod pseudo_vertical_align;
#[path = "page_tests/radial_gradients.rs"]
mod radial_gradients;
#[path = "page_tests/root_border.rs"]
mod root_border;
#[path = "page_tests/scrolling.rs"]
mod scrolling;
#[path = "page_tests/selection.rs"]
mod selection;
#[path = "page_tests/sizing.rs"]
mod sizing;
#[path = "page_tests/sizing_blocks.rs"]
mod sizing_blocks;
#[path = "page_tests/sizing_intrinsic.rs"]
mod sizing_intrinsic;
#[path = "page_tests/sizing_overflow.rs"]
mod sizing_overflow;
#[path = "page_tests/sizing_percentages.rs"]
mod sizing_percentages;
#[path = "page_tests/sizing_vertical.rs"]
mod sizing_vertical;
#[path = "page_tests/text_transform.rs"]
mod text_transform;
#[path = "page_tests/viewport_units.rs"]
mod viewport_units;

fn ui_node(node: &Node) -> UiNode {
    match node {
        Node::Text(text) => UiNode::Text(text.clone()),
        Node::Element(element) => UiNode::Element {
            tag: element.name.clone(),
            class: element
                .attribute("class")
                .and_then(AttributeValue::as_str)
                .map(str::to_owned),
            id: element
                .attribute("id")
                .and_then(AttributeValue::as_str)
                .map(str::to_owned),
            attributes: element
                .attributes
                .iter()
                .map(|attribute| {
                    (
                        attribute.name.clone(),
                        attribute.value.as_str().map(str::to_owned),
                    )
                })
                .collect(),
            children: element.children.iter().map(ui_node).collect(),
        },
    }
}

fn configured_browser() -> PathBuf {
    if let Some(browser) = std::env::var_os("CTML_CSS_PARITY_BROWSER") {
        let browser = PathBuf::from(browser);
        assert!(
            browser.exists(),
            "CTML_CSS_PARITY_BROWSER does not exist: {}",
            browser.display()
        );
        assert!(
            chromium_is_available(&browser),
            "CTML_CSS_PARITY_BROWSER must be Chromium {CHROMIUM_VERSION}: {}",
            browser.display()
        );
        return browser;
    }
    if std::env::var_os("CTML_CSS_PARITY_PREFER_DOWNLOADED").is_some() {
        return download_chromium();
    }
    if let Some(browser) = std::env::var_os("CHROME_BIN") {
        let browser = PathBuf::from(browser);
        if browser.exists() && chromium_is_available(&browser) {
            return browser;
        }
    }

    let executable_suffix = std::env::consts::EXE_SUFFIX;
    let browser_names = [
        "chrome-headless-shell",
        "chromium",
        "chromium-browser",
        "google-chrome",
        "chrome",
    ];
    std::env::var_os("PATH")
        .into_iter()
        .flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
        .flat_map(|directory| {
            browser_names
                .iter()
                .map(move |name| directory.join(format!("{name}{executable_suffix}")))
        })
        .find(|browser| browser.exists() && chromium_is_available(browser))
        .unwrap_or_else(download_chromium)
}

fn chromium_is_available(browser: &Path) -> bool {
    Command::new(browser)
        .arg("--version")
        .output()
        .is_ok_and(|output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout).contains(CHROMIUM_VERSION)
        })
}

fn download_chromium() -> PathBuf {
    log_notice("No system Chromium found; checking the test browser cache.");
    let cache_path = parity_artifact_dir().join("chromium-cache");
    fs::create_dir_all(&cache_path).expect("create Chromium browser cache directory");
    if let Some(browser) = find_cached_chromium(&cache_path) {
        log_success(&format!("Reusing cached Chromium at {}", browser.display()));
        return browser;
    }

    log_step("No cached Chromium found; downloading it once");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("create Tokio runtime for Chromium download");
    let options = BrowserFetcherOptions::builder()
        .with_path(cache_path)
        .with_kind(BrowserKind::ChromeHeadlessShell)
        .with_version(
            CHROMIUM_VERSION
                .parse::<BrowserVersion>()
                .expect("pinned Chromium version should be valid"),
        )
        .build()
        .expect("configure Chromium test browser");
    let progress = ProgressBar::start("Preparing Chromium Headless Shell (downloading if needed)");
    let result = runtime.block_on(async {
        tokio::time::timeout(
            CHROMIUM_DOWNLOAD_TIMEOUT,
            BrowserFetcher::new(options).fetch(),
        )
        .await
    });
    progress.finish();
    let installation = result
        .unwrap_or_else(|_| panic!("timed out downloading Chromium after 300 seconds"))
        .unwrap_or_else(|error| {
            panic!(
                "could not download Chromium Headless Shell for CSS parity. Set CTML_CSS_PARITY_BROWSER to a Chromium executable; first-time downloads require network access: {error}"
            )
        });
    log_success("Chromium is ready");
    installation.executable_path
}

fn find_cached_chromium(cache_path: &Path) -> Option<PathBuf> {
    let executable_names = ["chrome-headless-shell", "chrome-headless-shell.exe"];
    let mut pending = vec![(cache_path.to_path_buf(), 0)];

    while let Some((directory, depth)) = pending.pop() {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| executable_names.contains(&name))
                && chromium_is_available(&path)
            {
                return Some(path);
            }
            if depth < 5 && path.is_dir() {
                pending.push((path, depth + 1));
            }
        }
    }
    None
}

fn capture_browser(browser: &Path, input_dir: &Path, output: &Path) {
    if output.exists() {
        fs::remove_file(output).expect("remove stale browser screenshot");
    }
    let profile = input_dir.join("profile");
    fs::create_dir_all(&profile).expect("create temporary browser profile");
    let url = Url::from_file_path(input_dir.join("index.html"))
        .expect("convert HTML fixture path to a file URL")
        .to_string();
    let mut command = Command::new(browser);
    command.args([
        "--headless",
        "--disable-gpu",
        "--disable-dev-shm-usage",
        "--no-sandbox",
        "--no-zygote",
        "--disable-crashpad-for-testing",
        "--disable-lcd-text",
        "--font-render-hinting=none",
        "--hide-scrollbars",
        "--force-color-profile=srgb",
        "--force-device-scale-factor=1",
        &format!("--window-size={WIDTH},{HEIGHT}"),
        &format!("--screenshot={}", output.display()),
        &format!("--user-data-dir={}", profile.display()),
        &url,
    ]);
    let stdout_path = input_dir.join("chromium.stdout.log");
    let stderr_path = input_dir.join("chromium.stderr.log");
    command
        .stdout(Stdio::from(
            File::create(&stdout_path).expect("create Chromium stdout log"),
        ))
        .stderr(Stdio::from(
            File::create(&stderr_path).expect("create Chromium stderr log"),
        ));
    let mut child = command.spawn().expect("launch browser for screenshot");
    let progress = ProgressBar::start("Capturing browser screenshot");
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().expect("wait for Chromium process") {
            progress.finish();
            break status;
        }
        if started.elapsed() >= CHROMIUM_CAPTURE_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            drop(progress);
            panic!(
                "Chromium screenshot timed out after 30 seconds. stderr:\n{}",
                read_log(&stderr_path)
            );
        }
        thread::sleep(Duration::from_millis(100));
    };
    if status.success() && output.exists() {
        return;
    }

    let stdout = read_log(&stdout_path);
    let stderr = read_log(&stderr_path);
    let sandbox_hint = if stderr.contains("sandbox_host_linux.cc")
        && stderr.contains("shutdown: Operation not permitted")
    {
        " This Linux environment blocks a syscall Chromium requires at startup. Run the test on the host or allow shutdown() for Chromium in the container seccomp profile."
    } else {
        ""
    };

    panic!(
        "Chromium exited with {status} and did not produce a valid screenshot at {}. Browser: {}. Logs: {}.{sandbox_hint}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.display(),
        browser.display(),
        input_dir.display()
    );
}

fn read_log(path: &Path) -> String {
    let Ok(log) = fs::read_to_string(path) else {
        return format!("could not read log at {}", path.display());
    };
    let lines = log.lines().collect::<Vec<_>>();
    let preview = lines
        .iter()
        .take(40)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    if lines.len() > 40 {
        format!("{preview}\n... log truncated; full log: {}", path.display())
    } else {
        preview
    }
}

struct RgbaImage {
    dimensions: (u32, u32),
    pixels: Vec<u8>,
}

fn read_rgba_png(path: &Path) -> RgbaImage {
    let decoder = png::Decoder::new(BufReader::new(
        File::open(path).expect("open PNG screenshot"),
    ));
    let mut reader = decoder.read_info().expect("read PNG metadata");
    let mut encoded = vec![0; reader.output_buffer_size().expect("PNG output buffer size")];
    let frame = reader
        .next_frame(&mut encoded)
        .expect("decode PNG screenshot");
    let source = &encoded[..frame.buffer_size()];
    let mut pixels = Vec::with_capacity((frame.width * frame.height * 4) as usize);
    match frame.color_type {
        png::ColorType::Rgba => pixels.extend_from_slice(source),
        png::ColorType::Rgb => {
            for pixel in source.chunks_exact(3) {
                pixels.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for pixel in source.chunks_exact(2) {
                pixels.extend_from_slice(&[pixel[0], pixel[0], pixel[0], pixel[1]]);
            }
        }
        png::ColorType::Grayscale => {
            for gray in source {
                pixels.extend_from_slice(&[*gray, *gray, *gray, 255]);
            }
        }
        png::ColorType::Indexed => panic!("indexed PNG should be expanded by the decoder"),
    }
    RgbaImage {
        dimensions: (frame.width, frame.height),
        pixels,
    }
}

fn compare_pixels(first: &[u8], second: &[u8]) -> (usize, usize, Vec<u8>, u8) {
    assert_eq!(first.len(), second.len(), "screenshot byte lengths");
    let mut differing_pixels = 0;
    let mut exact_differences = 0;
    let mut diff_image = Vec::with_capacity(first.len());
    let mut max_delta = 0;
    for (first_pixel, second_pixel) in first.chunks_exact(4).zip(second.chunks_exact(4)) {
        let delta = (0..3)
            .map(|channel| first_pixel[channel].abs_diff(second_pixel[channel]))
            .max()
            .unwrap_or_default();
        max_delta = max_delta.max(delta);
        if delta > 0 {
            exact_differences += 1;
            diff_image.extend_from_slice(&[delta, 0, 0, 255]);
        } else {
            diff_image.extend_from_slice(&[0, 0, 0, 255]);
        }
        if delta > CHANNEL_TOLERANCE {
            differing_pixels += 1;
        }
    }
    (differing_pixels, exact_differences, diff_image, max_delta)
}

fn write_rgba_png(path: &Path, width: u32, height: u32, pixels: &[u8]) {
    let file = File::create(path).expect("create PNG screenshot");
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .expect("write PNG header")
        .write_image_data(pixels)
        .expect("write PNG image");
}

fn parity_artifact_dir() -> PathBuf {
    std::env::var_os("CTML_CSS_PARITY_ARTIFACTS")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/css-parity")
        })
}
