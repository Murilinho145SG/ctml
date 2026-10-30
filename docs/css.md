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

All declared and imported stylesheets match against the same CTML selector tree. CTML does not currently define component boundaries, shadow trees, or scoped-style sources, so selectors have document-wide scope and component encapsulation is deferred until such boundaries exist.

Stylesheets can import local files with a quoted path or `url()`, for example `@import "shared/colors.css";`. Imported rules are expanded where the import appears, and nested imports resolve relative to their importing file. Imports cannot escape the application's `src/` directory or form cycles. Network URLs, import media qualifiers, and escaped import paths are currently rejected. Imported files are tracked as build dependencies so Cargo recompiles the application when they change.

The CSS crate also exposes a local resource loader for binary resources. It resolves URL paths relative to their stylesheet, rejects external schemes, queries, fragments, and paths that escape the configured root, and caps reads at 64 MiB. `@font-face` accepts ordered `local()` and local `url()` sources, with `format()` and `tech()` hints; it tries candidates in order and registers the first available font under the declared `font-family`. URL files are embedded in the application build. Static `font-weight`, `font-style`, and `font-stretch` descriptors are supported, along with variable `font-weight` ranges and oblique angle ranges. Variable `font-width` ranges are not supported.

Stylesheet files must be UTF-8. An optional UTF-8 BOM is removed. CSS comments are discarded outside quoted strings, and an unterminated comment is an error. The legacy `@charset` rule does not change decoding and is unsupported.

## CSS value processing

CTML follows these value-processing stages for the properties it supports:

- **Specified values:** the stylesheet parser retains declaration values as CSS token text after removing comments and handling escapes. Property-specific parsers validate the supported grammar; invalid or unsupported declarations do not become style values.
- **Computed styles:** the cascade combines inherited text properties, modeled initial values, stylesheet rules, and inline declarations using importance, layer order, specificity, and source order. Custom properties and supported `var()` and `attr()` substitutions are resolved before the receiving property is parsed. The renderer stores the resulting values in its internal text style; it does not expose a CSSOM.
- **Used values:** relative lengths and percentages resolve when their needed context is available, including font metrics, line height, viewport dimensions, containing text area, or the active page size. Layout then calculates text flow, wrapping, and line metrics. Resizing recompiles styles with the new viewport dimensions.
- **Actual values:** the native shaping and rasterization path selects available font faces, positions glyphs, maps colors to the display's sRGB output, and paints the resulting pixels into the render target. Pixel placement and glyph metrics therefore reflect the native font and raster backends.

These stages describe CTML's implementation boundary rather than a promise that every CSS property or every CSS value grammar is supported.

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
- `width`, `min-width`, and `max-width` on the page `body` and block elements constrain text flow. `inline-size`, `min-inline-size`, and `max-inline-size` size the inline axis in horizontal and page-root vertical writing modes. Supported values are non-negative lengths and percentages of the containing text area. `width`/`inline-size` also accept `auto`, min-size properties accept `auto`, and max-size properties accept `none`. Nested block percentages resolve against the nearest containing text area.
- Empty blocks reserve flow space through `height`, `min-height`, and `max-height`. Content-bearing blocks use fixed `height`/`block-size` and min/max constraints for following flow; auto height starts from content height, then maximums cap the used block extent. Text that exceeds a fixed or maximum block size remains visible under the default `overflow: visible`, and following blocks start after the sized block extent. The one- and two-value `overflow: visible | hidden | clip | auto | scroll` forms and the `overflow-x`/`overflow-y` longhands clip descendant text, fragment backgrounds, selections, and decorations to the used inline and block extents of content-bearing blocks in horizontal and page-root vertical writing modes. Clipped text does not receive pointer or selection hits. On the page root, `auto` and `scroll` accept mouse-wheel input along scrollable axes. `scroll` draws a scrollbar on the selected root axis; `auto` draws one when formatted content overflows. Thumb size and position follow the formatted root content and scroll offset. `hidden` and `clip` do not respond to user scrolling. Under a horizontal page root, content-bearing blocks with `overflow-y: auto | scroll` scroll under the pointer and draw a vertical scrollbar; their text, backgrounds, selections, decorations, and hit testing follow the nested offset. Nested horizontal-axis scrolling, nested scrollers under a vertical page root, and scroll-chain composition are not implemented. Overflow clipping also supports nested orthogonal blocks under a horizontal page root, using their local writing transforms; nested orthogonal blocks under a vertical page root are not composed independently. `block-size`, `min-block-size`, and `max-block-size` size the block axis in horizontal and page-root vertical writing modes. These sizes accept non-negative lengths and percentages of the available page content area.
- Under a horizontal page root, nested vertical blocks wrap against their own inline extent, compose glyph columns, fragment backgrounds, selection highlights, and decorations locally, apply overflow clipping in their transformed bounds, and map pointer hits back to the buffer. Cross-axis block sizing still constrains page flow. Nested orthogonal blocks under a vertical page root are not composed independently yet.
- `box-sizing` accepts `content-box` (the initial value) and `border-box`. Both values currently produce the same text area because padding and borders are not part of the text-flow model yet.
- `visibility: visible|hidden|collapse` inherits. Hidden text keeps its layout space but does not paint text, fragment backgrounds, selection highlights, or decorations; `visibility: visible` can restore rendering on a descendant. `collapse` behaves like `hidden` because table formatting is not implemented.
- `pointer-events: auto|none` controls pointer hit testing on text. `none` skips an element's text as a target while allowing descendants to restore targeting with `auto`.
- `user-select: auto|text|none|all` controls text selection. `none` blocks selection without disabling pointer interaction, `text` restores selection inside a disabled ancestor, and `all` selects the owning element's complete text atomically.
- `display: none` suppresses the element, its descendants, and generated content from rendering. `display: block` and `display: flow-root` use block text flow; `display: inline`, `display: inline-block`, and `display: contents` use inline text flow. Multi-keyword flow forms accept order-independent block/inline outer and flow/flow-root inner values. `display: list-item` adds a marker and block text flow; forms such as `inline list-item` place the marker in inline text flow. `<li>` elements use bullets inside unordered lists and ascending decimal numbers inside ordered lists. The predefined CSS Counter Styles Level 3 names are supported for `list-style-type`, `counter()`, and `counters()`, including numeric, alphabetic, fixed, disclosure, localized decimal, kana, Ethiopic, and Japanese, Chinese, and Korean longhand systems. Anonymous `symbols()` styles support cyclic, fixed, symbolic, alphabetic, and numeric systems with string symbols. The CJK longhand styles fall back to `cjk-decimal` outside their supported ranges; disclosure markers use downward and right-pointing triangles. The `list-style` shorthand accepts one marker type. `contents` preserves child flow. Elements remain in the selector tree for structural matching; CSS formatting boxes are not implemented.
- Generated content and selection: `::before` and `::after` accept quoted strings and `attr()` expressions through `content`; multiple values can be concatenated. `attr(name, fallback)` emits its token-sequence fallback when the attribute is missing or fails conversion. `type(<string>)`, `type(<number>)`, `type(<integer>)`, `type(<custom-ident>)`, and simple finite `<percentage>`, `<length>`, `<length-percentage>`, `<angle>`, `<time>`, `<frequency>`, and `<resolution>` tokens convert attribute text to those values. Supported style declarations also accept string, integer, number, percentage, length, length-percentage, angle, color, and custom-ident `attr()` values, including typed fallbacks. `calc()`, `min()`, `max()`, and `clamp()` expressions are evaluated for typed integer, number, percentage, length, length-percentage, and angle attribute values. String attributes are substituted as CSS string tokens. `env(safe-area-inset-*)` and `env(safe-area-max-inset-*)` resolve to `0px` on the desktop host; unknown environment names use their fallback if present. Generated text inherits the originating element's style and can override supported text properties. Global and element-scoped `::selection` rules support `background-color` and `color`, including alpha.
- `::first-line` supports `color`, `opacity` (number or percentage, clamped to 0–1), `background-color` and the color-only `background` shorthand, `text-decoration-line` values `underline`, `overline`, and `line-through`, font family, size, weight, style, stretch, kerning, supported variants/features, `text-rendering` for glyph shaping, letter spacing, inherited and overridden word spacing, text shadow, text emphasis, and the optional `left|right` side in `text-emphasis-position` in vertical writing, `text-orientation` for glyph orientation, `text-transform` on the first formatted line of a block, `line-height` on the first line box, and `vertical-align` using `baseline`, `sub`, `super`, `middle`, `text-top`, `text-bottom`, `top`, `bottom`, signed lengths, or percentages of line height. `middle` uses the parent x-height, `text-top` compares cap-height metrics, and `text-bottom` uses a 0.2em descent estimate; `top` and `bottom` align the pseudo line box within the parent line-height. Opacity scales alpha on supported pseudo-element paints; overlapping paints are not yet composited as one isolated group. Text transformation and word spacing affect wrapping, and the pseudo-element line-height affects the first line box; other typography changes glyph rendering while wrapping and line metrics still come from the base text layout. `::first-letter` inherits supported inherited styles from `::first-line` and selects the first Unicode letter, number, or symbol with its associated preceding punctuation and following punctuation; intervening Unicode space separators are included, except ideographic space. This selected text can span adjacent text runs. `text-transform` changes the selected letter while preserving the punctuation. It supports the same color and decoration properties, including opacity and the color-only `background` shorthand, plus font family, size, weight, style, stretch, kerning, variants, feature settings, `text-rendering` for glyph shaping, line-height, letter-spacing, text-shadow, text-emphasis, and the optional `left|right` side in `text-emphasis-position` in vertical writing, `text-orientation` for glyph orientation, and `vertical-align` with the same supported values. Pseudo-element decorations honor the supported `text-decoration-style`, `text-decoration-color`, `text-decoration-thickness`, `text-decoration-skip-ink`, and `text-underline-offset` values; other pseudo-element styles are not supported yet.
- `hyphens: manual` displays a hyphen when a line wraps at U+00AD SOFT HYPHEN; the character stays invisible when the word does not break there. `hyphens: none` disables this discretionary break. `hyphens: auto` adds dictionary break points for recognized `lang` values using embedded Knuth–Liang dictionaries; explicit soft hyphens take priority. A language must be specified on the element or an ancestor for automatic hyphenation. Unsupported or missing languages do not add dictionary break points.
- Text and page properties: `color`, `background-color` on the page and rendered text fragments, and the `background` shorthand for standalone supported colors or `none`; `font-family`, `font-size` in `px`, `em`, `rem`, `lh`, `ex`, `ch`, `ic`, `cap`, viewport units (`vw`, `vh`, `vi`, `vb`, `vmin`, `vmax` and all `sv*`, `lv*`, and `dv*` variants), `%` relative to the inherited size, and absolute units (`cm`, `mm`, `Q`, `in`, `pt`, `pc`), `font-weight` (`normal`, `bold`, `bolder`, `lighter`, or a numeric value from 1 to 1000), `font-style` (`normal`, `italic`, or `oblique` with an optional angle from -90deg to 90deg, including angle calculations), `font-stretch` keywords and percentages, `font-kerning` (`auto`, `normal`, and `none`), `font-variant-ligatures` (common, discretionary, historical, and contextual controls), `font-variant-position` (`normal`, `sub`, and `super`), `font-variant-numeric` (lining/oldstyle, proportional/tabular, fraction, ordinal, and slashed-zero features), `font-variant-east-asian` (JIS encodings, simplified/traditional forms, full/proportional width, and ruby), `font-variant` as a shorthand for supported ligature, position, numeric, East Asian, and caps values, `font-variant-caps` (`normal`, `small-caps`, `all-small-caps`, `petite-caps`, `all-petite-caps`, `unicase`, and `titling-caps`), `font-feature-settings` with four-character OpenType tags and numeric, `on`, or `off` values, the `font` shorthand for supported style (including an oblique angle), small-caps, weight, stretch, size, line-height, and family values, `line-height` as a multiplier, percentage, `em`, `rem`, `lh`, `ex`, `ch`, `ic`, `cap`, viewport units, or absolute length, `letter-spacing` in `px`, `em`, `lh`, `ex`, `ch`, `ic`, `cap`, or viewport units, `direction: ltr|rtl` for inherited paragraph base direction, `unicode-bidi: isolate` for isolated inline ranges, page-root `writing-mode: horizontal-tb|vertical-rl|vertical-lr` for horizontal or vertical column flow, `text-align` (`left`, `right`, `center`, `justify`, `start`, and `end`), `text-transform`, `text-shadow` with multiple comma-separated shadows, offsets, optional blur lengths, and colors; font-relative lengths include `lh`, `ex`, `ch`, `ic`, and `cap`, `text-decoration` line, style, color, and thickness components, `text-emphasis` as a shorthand for supported style and color values, `text-decoration-color`, `text-decoration-skip-ink` (`auto`, `none`, or `all`), `text-decoration-thickness` (`auto`, `from-font`, or a non-negative length or percentage), `text-underline-offset` (`auto`, signed lengths, or percentages), and `cursor`.
- Text spacing and emphasis: `word-spacing` supports `normal`, signed lengths, and percentages of the font size. `text-emphasis` and `text-emphasis-style` support `none`, single-grapheme strings, and filled/open `dot`, `circle`, `double-circle`, `triangle`, and `sesame` marks. `text-emphasis-color` accepts supported CSS colors and `currentColor`; `text-emphasis-position` supports `over` and `under` for horizontal text and positions marks on the requested `left`/`right` side in vertical flow. Page-root vertical flow rotates the composed layout; `text-orientation: mixed|upright|sideways` controls per-glyph orientation using Unicode vertical-orientation data.
- `em` font sizes are relative to the inherited size. `rem` currently uses the renderer's 16 CSS pixel root size.
- `calc()`, `min()`, `max()`, and `clamp()` evaluate supported length and percentage expressions in font sizes, line heights, spacing, text effects, and width/height constraints. Number expressions are supported for numeric line heights, opacity, and font weights. Math expressions enforce matching number or length types, finite results, and whitespace around binary `+` and `-`; newer typed arithmetic is not implemented.
- Viewport units apply to supported font sizes, line heights, text effects, letter spacing, and box dimensions. They resolve against the current render target and are recalculated on resize. CTML has no dynamic browser chrome, so unprefixed, `sv*`, `lv*`, and `dv*` variants use the same viewport dimensions.
- `ex`, `ch`, `ic`, and `cap` resolve from the parent typeface for `font-size` and the font-size component of `font`. For `line-height`, `letter-spacing`, text shadows, text decorations, and box sizing, they resolve from the element typeface. `lh` in `line-height` and font shorthand line-height resolves from the parent line height; metric units in font shorthand line-height resolve from the parent typeface.
- `text-transform` supports `none`, `uppercase`, `lowercase`, and `capitalize`. `text-decoration` supports `none`, `underline`, `overline`, and `line-through`; `text-decoration-style` supports `solid`, `double`, `dotted`, `dashed`, and `wavy`.
- Colors support standard named colors, `transparent`, `currentColor` for text and backgrounds, `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`, comma-separated `rgb()`/`rgba()` and `hsl()`/`hsla()` syntax, modern space-separated `rgb()`/`hsl()` syntax, `hwb()`, `lab()`, `lch()`, `oklab()`, `oklch()`, and `color()` with `srgb`, `srgb-linear`, `display-p3`, `display-p3-linear`, `a98-rgb`, `prophoto-rgb`, `rec2020`, `rec2100-pq`, `rec2100-hlg`, `rec2100-linear`, `xyz`, `xyz-d50`, or `xyz-d65` spaces. Absolute modern color functions accept `none` components and render each as zero, including hue and alpha. For `color-mix(in srgb, ...)`, matching missing components and alpha are carried from direct `color(srgb ...)` inputs before interpolation. Missing alpha from direct `color(srgb ...)` inputs is also carried through other supported interpolation spaces. Matching missing components from direct predefined RGB-profile colors are carried through supported RGB interpolation spaces, including nested mixes. Matching `x`, `y`, and `z` components from direct XYZ-profile colors are carried through XYZ interpolation spaces. Matching missing components from direct Lab, LCH, Oklab, and Oklch colors are carried through corresponding interpolation spaces, including nested mixes. Powerless hues are ignored in HSL, HWB, LCH, and Oklch interpolation. `color-mix()` in `srgb`, `srgb-linear`, `display-p3`, `display-p3-linear`, `a98-rgb`, `prophoto-rgb`, `rec2020`, `rec2100-pq`, `rec2100-hlg`, `rec2100-linear`, `xyz`, `xyz-d50`, `xyz-d65`, `lab`, `lch`, `oklab`, `oklch`, `hsl`, or `hwb` interpolates two supported colors with optional percentages, nested color functions, and alpha-aware mixing; polar spaces support `shorter`, `longer`, `increasing`, and `decreasing hue` interpolation, defaulting to `shorter hue`. Source colors are mapped to sRGB before interpolation. Relative `rgb(from ...)` supports `r`, `g`, `b`, and `alpha` references and numeric or percentage channel values. Number, percentage, and angle expressions in relative color channels support `calc()`, `min()`, `max()`, and `clamp()`; angle calculations accept degrees, gradians, radians, and turns. Relative channel values of `none` are converted as zero; missing-component propagation from source colors is not supported. Relative `hsl(from ...)` supports `h`, `s`, `l`, and `alpha` references plus angle overrides in degrees, gradians, radians, and turns. Relative `hwb(from ...)` supports `h`, `w`, `b`, and `alpha` references plus angle overrides. Relative `lab(from ...)`, `lch(from ...)`, `oklab(from ...)`, and `oklch(from ...)` support channel references and numeric or percentage component values. Relative `color(from ...)` supports `srgb`, `srgb-linear`, `display-p3`, `display-p3-linear`, `a98-rgb`, `prophoto-rgb`, `rec2020`, `rec2100-pq`, `rec2100-hlg`, `rec2100-linear`, `xyz`, `xyz-d50`, and `xyz-d65` profiles with channel references and numeric or percentage values. Missing-component behavior and other color profiles are not implemented. `opacity` scales glyphs, text decorations, text-fragment backgrounds, selection highlights, pseudo-element paints, shadows, and emphasis marks, and combines along ancestors. Overlapping paints are not composited as an isolated element group. Alpha is supported for text, page backgrounds, and rendered text-fragment backgrounds; translucent page backgrounds are composited over white. Wide-gamut values are perceptually gamut-mapped to sRGB for native rendering.
- `@color-profile` parses local ICC profile declarations and loads valid profiles relative to the declaring stylesheet within the configured resource root. The native renderer accepts loaded profiles through `WindowConfig.color_profiles` and resolves absolute `color()`, relative `color(from ...)`, and custom `color-mix()` spaces in color-valued styles. Little CMS converts ICC grayscale, RGB, CMYK, and multichannel device profiles with up to 15 components; non-device Lab/XYZ and other non-MCH signatures are not currently supported as custom profiles.
- System color keywords include the CSS Color system palette and deprecated aliases. CTML currently resolves them through fixed light-theme fallback values; it does not read OS colors, forced-colors mode, or element color schemes.
- Small/petite caps are synthesized with uppercase glyphs at a reduced size; unicase, titling caps, subscript, and superscript use the selected font's OpenType features. A font without a requested OpenType feature may render the text without that glyph variant.
- Cursor values include the standard `auto`, `default`, `context-menu`, `help`, `pointer`, `progress`, `wait`, `cell`, `crosshair`, `text`, `vertical-text`, `alias`, `copy`, `move`, `no-drop`, `not-allowed`, `grab`, `grabbing`, resize, scroll, and zoom keywords, plus the backend extensions `all-resize` and `dnd-ask`. The OS draws the native cursor icon.
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
- Hebrew, Armenian, CJK, and other list marker systems; marker position and styling; reversed ordered lists; and explicit list-item numbering.
- The box model: overflow behavior for fixed `height` and `max-height` on content-bearing blocks, applying `border-box` to padding and borders, sizing of inline and replaced elements, intrinsic sizing keywords, `margin`, `padding`, and `gap`. Block widths currently affect text wrapping without generating positioned CSS boxes.
- Borders, outlines, border radius, and shadows.
- Flexbox, Grid, floats, and positioning (`position`, `top`, `right`, `bottom`, and `left`).
- Overflow, clipping, scrolling, and scrollbars.
- White-space controls, including `white-space`, `text-wrap`/`text-wrap-mode` (`wrap` and `nowrap`), `word-break`, and `text-overflow`.

### Painting and colors

- Backgrounds on empty elements and complete CSS boxes; `background-color` currently paints rendered text fragments and the page.
- Background images, gradients, and the `opacity` property.
- Other predefined/custom color spaces.
- Vertical text-emphasis side placement.

### Interaction and responsive rules

- `:hover` and other interaction states, `caret-color`, and `accent-color`.
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
