# CTML CSS

CTML uses regular `.css` files. A stylesheet is declared in the CTML header and parsed during the Rust build. The compiled application receives native style data, so it does not include a CSS parser or browser runtime.

```ctml
<headers>
    <application name="Example" desktop />
    <stylesheet href="main.css" />
</headers>

<body>
    <main>
        <h1 class="title">Hello, CTML</h1>
        <p id="intro">A native page styled with CSS.</p>
    </main>
</body>
```

The stylesheet path is relative to the Rust application's `src/` directory.

```css
body {
    font-family: sans-serif;
    color: #222222;
    background-color: white;
}

main .title {
    color: #2563eb;
    font-size: 32px;
    font-weight: 700;
    line-height: 1.2;
    letter-spacing: 0.02em;
    text-transform: uppercase;
}

#intro {
    font-size: 1em;
    font-style: italic;
    text-decoration: underline;
}
```

## Supported CSS subset

- Selectors: type, class, ID, compound, descendant, direct child (`>`), comma-separated groups, and `*`.
- Text and page properties: `color`, `background-color` on the page, `font-family`, `font-size` in `px` or `em`, `font-weight` (`normal`, `bold`, or a numeric value from 1 to 1000), `font-style` (`normal`, `italic`, or `oblique`), `font-stretch` keywords and percentages, `font-kerning` (`auto`, `normal`, or `none`), `font-variant-ligatures` (common, discretionary, and contextual controls), `font-variant-position` (`normal`, `sub`, and `super`), `font-variant`/`font-variant-caps` (`normal`, `small-caps`, or `all-small-caps`), `line-height` as a multiplier or length, `letter-spacing` in `px` or `em`, `text-align` (`left`, `right`, `center`, `justify`, `start`, and `end`), `text-transform`, `text-decoration` with `solid` or `double` underline style, `text-decoration-color`, and `cursor`.
- `text-transform` supports `none`, `uppercase`, `lowercase`, and `capitalize`. `text-decoration` supports `none`, `underline`, `overline`, and `line-through`.
- Colors support standard named colors, `transparent`, `currentColor` for text and page backgrounds, `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`, comma-separated `rgb()`/`rgba()` and `hsl()`/`hsla()` syntax, modern space-separated `rgb()`/`hsl()` syntax, and `hwb()` with optional slash alpha. RGB channels accept numbers or percentages; hue accepts degrees, gradians, radians, or turns. Alpha is supported for text and the page background; translucent page backgrounds are composited over white.
- Small-cap and subscript/superscript variants use the selected font's OpenType features. A font without the requested feature may render the text without that glyph variant.
- Cursor values include `auto`, `default`, `pointer`, `text`, `crosshair`, `move`, `wait`, `progress`, `help`, `not-allowed`, `no-drop`, `copy`, `alias`, `cell`, `all-scroll`, `grab`, `grabbing`, zoom cursors, and common resize cursors. The OS draws the native cursor icon.
- The cascade follows selector specificity and source order. Text properties inherit through child elements.
- Text wraps to the available page width. Common block elements start a new line.
- Text can be selected by dragging the left mouse button; double-clicking selects a word.

## Not implemented

For a broader implementation checklist covering CSS feature areas, see the [CSS feature checklist](css-roadmap.md).

Unsupported CSS selectors and properties are currently ignored. They do not produce an error or affect the rendered page.

### Selectors and cascade

- Pseudo-classes and pseudo-elements, such as `:hover`, `:focus`, `:active`, and `::before`.
- Attribute selectors, such as `[disabled]` and `[type="text"]`.
- Adjacent-sibling and general-sibling combinators: `+` and `~`.
- Selector namespaces and escaped identifiers.
- `!important` priority and cascade layers.

### Layout and sizing

- `display`, including `none`, `inline`, `block`, and `inline-block`.
- The box model: `width`, `height`, `min-width`, `max-width`, `box-sizing`, `margin`, `padding`, and `gap`.
- Borders, outlines, border radius, and shadows.
- Flexbox, Grid, floats, and positioning (`position`, `top`, `right`, `bottom`, and `left`).
- Overflow, clipping, scrolling, and scrollbars.
- White-space controls, including `white-space`, `word-break`, and `text-overflow`.

### Painting and colors

- Per-element `background-color`; the supported background color applies to the page.
- Background images, gradients, and the `opacity` property.
- Modern color functions such as `lab()`, `lch()`, `oklab()`, `oklch()`, and `color()`.
- Text shadows and dotted or wavy underline styles.

### Interaction and responsive rules

- `:hover` and other interaction states, `pointer-events`, and `user-select`.
- Custom cursor images (`cursor: url(...)`).
- Media queries, container queries, and other at-rules.
- CSS custom properties, `var()`, `calc()`, and other computed-value functions.

This list describes the current native renderer and will change as CTML's styling and layout support expands.

## Testing CSS support

Run the native style resolver tests with:

```sh
cargo test --offline -p ctml
```

The tests cover numeric font weights, oblique font style, font stretch, small-cap variants, decoration colors, named colors, alpha colors, `rgb()`/`rgba()` and `hsl()`/`hsla()`, and malformed value handling. The `examples/native-app` demo also exercises supported color and font values visually.

To open the visual demo, run:

```sh
cargo run --offline -p ctml-native-demo
```

The CTML/Chromium screenshot parity check runs as part of the `ctml` test suite. It uses Chromium from `PATH` or `CHROME_BIN`, then looks for the Chromium Headless Shell in `target/css-parity/chromium-cache/`. It downloads the headless shell only when neither source has a usable executable, and reuses that cached executable on later runs. The downloader is a development-only dependency. The check saves both screenshots and a red pixel-difference image under `target/css-parity/`. A pixel counts as different when any RGB channel differs by more than 16; the check fails when more than 8% of viewport pixels differ. To select a Chromium executable explicitly, set `CTML_CSS_PARITY_BROWSER` to its path.

Run the parity test by itself, with a single test thread, using:

```sh
cargo test -p ctml native_ctml_rendering_matches_browser_html_css_pixels -- --test-threads=1 --nocapture
```

The repository's Cargo configuration runs tests serially and shows test output by default. The parity test prints its progress while it parses fixtures, renders CTML, finds or downloads Chromium, captures HTML, and compares pixels. To run only that test, use:

```sh
cargo test -p ctml native_ctml_rendering_matches_browser_html_css_pixels
```

The first run needs network access only if Chromium is not already installed or cached. The downloaded browser is reused by later runs. The parity test requires Chromium to start and capture a screenshot. If a Linux container's seccomp profile blocks a syscall Chromium needs at startup, run the test on the host or allow `shutdown()` in the container profile; the test reports this restriction and fails rather than skipping pixel comparison.
