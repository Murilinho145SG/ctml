# CTML CSS

CTML uses regular `.css` files. One or more stylesheets can be declared in the CTML header and are parsed during the Rust build in header order. Their rules enter the cascade in that same order, so later sheets take precedence when layer, importance, and specificity are equal. The compiled application receives native style data, so it does not include a CSS parser or browser runtime.

```ctml
<headers>
    <application name="Example" desktop />
    <stylesheet href="reset.css" />
    <stylesheet href="main.css" />
</headers>

<body>
    <main>
        <h1 class="title">Hello, CTML</h1>
        <p id="intro">A native page styled with CSS.</p>
    </main>
</body>
```

Each stylesheet path is relative to the Rust application's `src/` directory.

Stylesheets can import local files with a quoted path or `url()`, for example `@import "shared/colors.css";`. Imported rules are expanded where the import appears, and nested imports resolve relative to their importing file. Imports cannot escape the application's `src/` directory or form cycles. Network URLs, import media qualifiers, and escaped import paths are currently rejected. Imported files are tracked as build dependencies so Cargo recompiles the application when they change.

Stylesheet files must be UTF-8. An optional UTF-8 BOM is removed. CSS comments are discarded outside quoted strings, and an unterminated comment is an error. The legacy `@charset` rule does not change decoding and is unsupported.

Named `@layer` order statements and blocks are supported, including escaped identifiers, nested named layers, and anonymous blocks. Anonymous layers remain distinct across stylesheet imports. Normal declarations in later layers outrank earlier layers, while `!important` layer order is reversed; unlayered normal rules outrank layered normal rules, and layered important rules outrank unlayered important rules.

Malformed layer declarations and unterminated comments report the source file, line, and column during the build. Incomplete rules at end of file, malformed selector rules, and unsupported at-rules are discarded so later rules can still apply. Invalid declarations are skipped so later declarations in the same rule can still apply.

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

- Selectors: type, class, ID, compound, descendant, direct child (`>`), adjacent sibling (`+`), general sibling (`~`), comma-separated groups, `*`, CSS escaped identifiers, `:is()`, `:where()`, `:not()`, relational `:has()` with descendant, child, adjacent-sibling, and general-sibling relative selectors, inherited `:lang()`, `:dir(ltr|rtl)`, `:link` and `:any-link` for `<a href>`, `:hover`, `:active`, `:focus`, `:focus-within`, and keyboard-driven `:focus-visible`, and attribute selectors for presence, exact match, token (`~=`), language prefix (`|=`), prefix (`^=`), suffix (`$=`), substring (`*=`), and ASCII-insensitive (`i`) or sensitive (`s`) matching.
- Structural pseudo-classes: `:root`, `:empty`, `:first-child`, `:last-child`, `:only-child`, `:nth-child()`, `:nth-last-child()`, and the corresponding `-of-type` forms. `:nth-*` accepts integers, `odd`, `even`, An+B expressions, and `of S` selector lists on `:nth-child()` and `:nth-last-child()`.
- `width`, `min-width`, and `max-width` on the page `body` and block elements constrain text wrapping; `inline-size`, `min-inline-size`, and `max-inline-size` are aliases in the current horizontal writing mode. Supported values are non-negative lengths and percentages of the containing text area. `width`/`inline-size` also accept `auto`, min-size properties accept `auto`, and max-size properties accept `none`. Nested block percentages resolve against the nearest containing block width.
- Empty blocks reserve vertical flow space through `height`, `min-height`, and `max-height`; content-bearing blocks reserve remaining space for `height` and `min-height`, while `max-height` caps that requested space when the text fits. When text exceeds a content block's height limit, overflow sizing is not implemented yet. `block-size`, `min-block-size`, and `max-block-size` are aliases in the current horizontal writing mode. These sizes accept non-negative lengths and percentages of the available page content height.
- `box-sizing` accepts `content-box` (the initial value) and `border-box`. Both values currently produce the same text area because padding and borders are not part of the text-flow model yet.
- `display: none` suppresses the element, its descendants, and generated content from rendering. `display: block` and `display: flow-root` use block text flow; `display: inline`, `display: inline-block`, and `display: contents` use inline text flow. `display: list-item` adds a marker and block text flow; `<li>` elements use bullets inside unordered lists and ascending decimal numbers inside ordered lists. Inherited `list-style-type` supports `disc`, `circle`, `square`, `none`, `decimal`, `lower-alpha`, and `upper-alpha`; the `list-style` shorthand accepts one of those marker types. `contents` preserves child flow. Elements remain in the selector tree for structural matching; CSS formatting boxes are not implemented.
- Generated content and selection: `::before` and `::after` accept quoted strings through `content`; generated text inherits the originating element's style and can override supported text properties. Global and element-scoped `::selection` rules support `background-color` and `color`, including alpha.
- `::first-line` supports `color`, `opacity` (number or percentage, clamped to 0–1), `background-color` and the color-only `background` shorthand, `text-decoration-line` values `underline`, `overline`, and `line-through`, font family, size, weight, style, stretch, kerning, supported variants/features, letter spacing, inherited and overridden word spacing, text shadow, text emphasis, `text-transform` on the first formatted line of a block, `line-height` on the first line box, and `vertical-align` using `baseline`, `sub`, `super`, `middle`, `text-top`, `text-bottom`, `top`, `bottom`, signed lengths, or percentages of line height. `middle` uses the parent x-height, `text-top` compares cap-height metrics, and `text-bottom` uses a 0.2em descent estimate; `top` and `bottom` align the pseudo line box within the parent line-height. Opacity scales alpha on supported pseudo-element paints; overlapping paints are not yet composited as one isolated group. Text transformation and word spacing affect wrapping, and the pseudo-element line-height affects the first line box; other typography changes glyph rendering while wrapping and line metrics still come from the base text layout. `::first-letter` supports the same color and decoration properties, including opacity and the color-only `background` shorthand, plus font family, size, weight, style, stretch, kerning, variants, feature settings, line-height, letter-spacing, text-shadow, text-emphasis, `text-transform` on the source first-letter character, and `vertical-align` with the same supported values. Pseudo-element decorations honor the supported `text-decoration-style`, `text-decoration-color`, `text-decoration-thickness`, `text-decoration-skip-ink`, and `text-underline-offset` values; other pseudo-element styles are not supported yet.
- Text and page properties: `color`, `background-color` on the page and rendered text fragments, and the `background` shorthand for standalone supported colors or `none`; `font-family`, `font-size` in `px`, `em`, `rem`, `lh`, `ex`, `ch`, `ic`, `cap`, viewport units (`vw`, `vh`, `vi`, `vb`, `vmin`, `vmax` and all `sv*`, `lv*`, and `dv*` variants), `%` relative to the inherited size, and absolute units (`cm`, `mm`, `Q`, `in`, `pt`, `pc`), `font-weight` (`normal`, `bold`, `bolder`, `lighter`, or a numeric value from 1 to 1000), `font-style` (`normal`, `italic`, or `oblique`), `font-stretch` keywords and percentages, `font-kerning` (`auto`, `normal`, and `none`), `font-variant-ligatures` (common, discretionary, historical, and contextual controls), `font-variant-position` (`normal`, `sub`, and `super`), `font-variant-numeric` (lining/oldstyle, proportional/tabular, fraction, ordinal, and slashed-zero features), `font-variant-east-asian` (JIS encodings, simplified/traditional forms, full/proportional width, and ruby), `font-variant` as a shorthand for supported ligature, position, numeric, East Asian, and caps values, `font-variant-caps` (`normal`, `small-caps`, `all-small-caps`, `petite-caps`, `all-petite-caps`, `unicase`, and `titling-caps`), `font-feature-settings` with four-character OpenType tags and numeric, `on`, or `off` values, the `font` shorthand for supported style, small-caps, weight, stretch, size, line-height, and family values, `line-height` as a multiplier, percentage, `em`, `rem`, `lh`, `ex`, `ch`, `ic`, `cap`, viewport units, or absolute length, `letter-spacing` in `px`, `em`, `lh`, `ex`, `ch`, `ic`, `cap`, or viewport units, `text-align` (`left`, `right`, `center`, `justify`, `start`, and `end`), `text-transform`, `text-shadow` with multiple comma-separated shadows, offsets, optional blur lengths, and colors; font-relative lengths include `lh`, `ex`, `ch`, `ic`, and `cap`, `text-decoration` line, style, color, and thickness components, `text-emphasis` as a shorthand for supported style and color values, `text-decoration-color`, `text-decoration-skip-ink` (`auto`, `none`, or `all`), `text-decoration-thickness` (`auto`, `from-font`, or a non-negative length or percentage), `text-underline-offset` (`auto`, signed lengths, or percentages), and `cursor`.
- Text spacing and emphasis: `word-spacing` supports `normal`, signed lengths, and percentages of the font size. `text-emphasis` and `text-emphasis-style` support `none`, single-grapheme strings, and filled/open `dot`, `circle`, `double-circle`, `triangle`, and `sesame` marks. `text-emphasis-color` accepts supported CSS colors and `currentColor`; `text-emphasis-position` supports `over` and `under` for horizontal text.
- `em` font sizes are relative to the inherited size. `rem` currently uses the renderer's 16 CSS pixel root size.
- `calc()`, `min()`, `max()`, and `clamp()` evaluate supported length and percentage expressions in font sizes, line heights, spacing, text effects, and width/height constraints. Number expressions are supported for numeric line heights, opacity, and font weights. Math expressions enforce matching number or length types, finite results, and whitespace around binary `+` and `-`; newer typed arithmetic is not implemented.
- Viewport units apply to supported font sizes, line heights, text effects, letter spacing, and box dimensions. They resolve against the current render target and are recalculated on resize. CTML has no dynamic browser chrome, so unprefixed, `sv*`, `lv*`, and `dv*` variants use the same viewport dimensions.
- `ex`, `ch`, `ic`, and `cap` resolve from the parent typeface for `font-size` and the font-size component of `font`. For `line-height`, `letter-spacing`, text shadows, text decorations, and box sizing, they resolve from the element typeface. `lh` in `line-height` and font shorthand line-height resolves from the parent line height; metric units in font shorthand line-height resolve from the parent typeface.
- `text-transform` supports `none`, `uppercase`, `lowercase`, and `capitalize`. `text-decoration` supports `none`, `underline`, `overline`, and `line-through`; `text-decoration-style` supports `solid`, `double`, `dotted`, `dashed`, and `wavy`.
- Colors support standard named colors, `transparent`, `currentColor` for text and backgrounds, `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`, comma-separated `rgb()`/`rgba()` and `hsl()`/`hsla()` syntax, modern space-separated `rgb()`/`hsl()` syntax, `hwb()`, `lab()`, `lch()`, `oklab()`, `oklch()`, and `color()` with `srgb`, `srgb-linear`, `display-p3`, `display-p3-linear`, `a98-rgb`, `prophoto-rgb`, `rec2020`, `xyz`, `xyz-d50`, or `xyz-d65` spaces. `color-mix()` in `srgb`, `srgb-linear`, `display-p3`, `display-p3-linear`, `a98-rgb`, `prophoto-rgb`, `rec2020`, `xyz`, `xyz-d50`, `xyz-d65`, `lab`, `lch`, `oklab`, `oklch`, `hsl`, or `hwb` interpolates two supported colors with optional percentages, nested color functions, and alpha-aware mixing; polar spaces support `shorter`, `longer`, `increasing`, and `decreasing hue` interpolation, defaulting to `shorter hue`. The renderer clips source colors to sRGB before interpolation. Relative `rgb(from ...)` supports `r`, `g`, `b`, and `alpha` references and numeric or percentage channel values. Number, percentage, and angle expressions in relative color channels support `calc()`, `min()`, `max()`, and `clamp()`; angle calculations accept degrees, gradians, radians, and turns. Relative channel values of `none` are converted as zero; missing-component propagation from source colors is not supported. Relative `hsl(from ...)` supports `h`, `s`, `l`, and `alpha` references plus angle overrides in degrees, gradians, radians, and turns. Relative `hwb(from ...)` supports `h`, `w`, `b`, and `alpha` references plus angle overrides. Relative `lab(from ...)`, `lch(from ...)`, `oklab(from ...)`, and `oklch(from ...)` support channel references and numeric or percentage component values. Relative `color(from ...)` supports `srgb`, `srgb-linear`, `display-p3`, `display-p3-linear`, `a98-rgb`, `prophoto-rgb`, `rec2020`, `xyz`, `xyz-d50`, and `xyz-d65` profiles with channel references and numeric or percentage values. Missing-component behavior and other color profiles are not implemented. RGB channels accept numbers or percentages; hue accepts degrees, gradians, radians, or turns. Alpha is supported for text, page backgrounds, and rendered text-fragment backgrounds; translucent page backgrounds are composited over white. Wide-gamut values are clipped to sRGB for native rendering.
- Small/petite caps are synthesized with uppercase glyphs at a reduced size; unicase, titling caps, subscript, and superscript use the selected font's OpenType features. A font without a requested OpenType feature may render the text without that glyph variant.
- Cursor values include `auto`, `default`, `pointer`, `text`, `crosshair`, `move`, `wait`, `progress`, `help`, `not-allowed`, `no-drop`, `copy`, `alias`, `cell`, `all-scroll`, `grab`, `grabbing`, zoom cursors, and common resize cursors. The OS draws the native cursor icon.
- The cascade follows `!important` priority, selector specificity, and source order. A CTML element may use a `style` attribute for inline declarations; normal inline declarations outrank stylesheet declarations, stylesheet `!important` declarations outrank normal inline declarations, and inline `!important` declarations outrank author stylesheet declarations of equal importance. Text properties inherit through child elements.
- `:focus` and `:focus-within` update when the user clicks text inside a link with `href` or an element with a numeric `tabindex`. Tab and Shift+Tab move focus through links and elements with non-negative numeric `tabindex`; keyboard focus matches `:focus-visible`, while pointer focus does not. CTML has no native form controls, so form-state pseudo-classes are not supported.
- Supported properties accept the CSS-wide keywords `inherit`, `initial`, `unset`, `revert`, and `revert-layer`. `revert` resets a supported declaration to the initial user-agent value modeled by CTML; `revert-layer` rolls back declarations in the current named layer.
- The `all` shorthand accepts those CSS-wide keywords and applies them to every CSS property supported by the renderer.
- Custom properties beginning with `--` inherit with case-sensitive names. `var()` substitutes them in supported property values, including nested fallbacks; cyclic custom properties become invalid and can use a fallback when referenced.
- Text wraps to the available page width. Common block elements start a new line.
- Text can be selected by dragging the left mouse button; double-clicking selects a word.

## Not implemented

For a broader implementation checklist covering CSS feature areas, see the [CSS feature checklist](css-roadmap.md).

Malformed declarations and selector rules are skipped without a warning, and parsing continues with later declarations and rules. Unknown at-rules are skipped. Unknown property names are retained by the stylesheet parser; properties the renderer does not support are ignored without a warning. Unsupported selectors are also ignored without a warning. Malformed layer declarations and unterminated comments fail the build with a file, line, and column diagnostic.

### Selectors and cascade

- Focus and form-state pseudo-classes such as `:focus`, `:disabled`, and `:checked`, plus pseudo-elements such as `::before`.
- Selector namespaces and escaped identifiers. CTML tag names do not currently accept namespace prefixes and CTML does not model namespace URIs.
- Stateful pseudo-classes beyond `:hover` and `:active`.
- `:dir(auto)` first-strong character detection; `:dir()` currently uses explicit inherited `dir="ltr|rtl"` values and defaults to `ltr`.
- `:visited`, form and range state, and `:target`; CTML currently has no link history, form controls, or document navigation target.
- Additional `::first-line` and `::first-letter` properties beyond the supported subset described above.

### Layout and sizing

- `display` values other than `none`, `block`, `inline`, `contents`, `flow-root`, `inline-block`, and `list-item`; CTML does not yet implement CSS formatting boxes.
- Other `list-style-type` values, marker position and styling, reversed ordered lists, and explicit list-item numbering.
- The box model: overflow behavior for fixed `height` and `max-height` on content-bearing blocks, applying `border-box` to padding and borders, sizing of inline and replaced elements, intrinsic sizing keywords, `margin`, `padding`, and `gap`. Block widths currently affect text wrapping without generating positioned CSS boxes.
- Borders, outlines, border radius, and shadows.
- Flexbox, Grid, floats, and positioning (`position`, `top`, `right`, `bottom`, and `left`).
- Overflow, clipping, scrolling, and scrollbars.
- White-space controls, including `white-space`, `word-break`, and `text-overflow`.

### Painting and colors

- Backgrounds on empty elements and complete CSS boxes; `background-color` currently paints rendered text fragments and the page.
- Background images, gradients, and the `opacity` property.
- Other predefined/custom color spaces and perceptual wide-gamut handling.
- Vertical text-emphasis side placement.

### Interaction and responsive rules

- `:hover` and other interaction states, `pointer-events`, and `user-select`.
- Custom cursor images (`cursor: url(...)`).
- Media queries, container queries, and other at-rules.
- CSS custom properties, `var()`, `calc()`, and other computed-value functions.

This list describes the current native renderer and will change as CTML's styling and layout support expands.

## Testing CSS support

The [CSS test matrix](css-test-matrix.md) maps each currently supported property, value family, selector, and cascade behavior to its unit and Chromium parity tests.

Run the renderer style resolver tests with:

```sh
cargo test --offline -p ctml-renderer --lib css_style
```

The tests cover numeric font weights, oblique font style, font stretch, small-cap variants, decoration colors, and style cascade behavior. The `ctml-css` tests cover named and alpha colors, `rgb()`/`rgba()`, `hsl()`/`hsla()`, selectors, and malformed color values. The `examples/native-app` demo also exercises supported color and font values visually.

To open the visual demo, run:

```sh
cargo run --offline -p ctml-native-demo
```

The CTML/Chromium screenshot parity suite runs as separate tests for the cases under `crates/ctml-renderer/tests/fixtures/parity/`. Each case has its own CTML, HTML, and CSS files and compares its own screenshots. It uses Chromium from `PATH` or `CHROME_BIN`, then looks for the Chromium Headless Shell in `target/css-parity/chromium-cache/`. It downloads the headless shell only when neither source has a usable executable, and reuses that cached executable on later runs. The downloader is a development-only dependency. Screenshots and red pixel-difference images are saved under `target/css-parity/<case>/`. Exact parity cases fail on any RGB pixel difference. Other cases allow up to 0.05% of pixels above channel tolerance 16, with documented per-case limits for attribute selectors, font-stretch, font-variant-shorthand, line-height-normal, and text-decoration-style in the [test matrix](css-test-matrix.md). To select a Chromium executable explicitly, set `CTML_CSS_PARITY_BROWSER` to its path.

Run the parity test by itself, with a single test thread, using:

```sh
cargo test -p ctml-renderer --lib page::tests::parity -- --test-threads=1 --nocapture
```

The repository's Cargo configuration runs tests serially and shows test output by default. The parity test prints its progress while it parses fixtures, renders CTML, finds or downloads Chromium, captures HTML, and compares pixels. To run only that test, use:

```sh
cargo test -p ctml-renderer --lib page::tests::parity
```

The first run needs network access only if Chromium is not already installed or cached. The downloaded browser is reused by later runs. The parity test requires Chromium to start and capture a screenshot. If a Linux container's seccomp profile blocks a syscall Chromium needs at startup, run the test on the host or allow `shutdown()` in the container profile; the test reports this restriction and fails rather than skipping pixel comparison.
