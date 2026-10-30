# CSS Feature Checklist

This document tracks CSS features that CTML may support. It is a broad implementation checklist based on the CSS standards, not a promise to reproduce every browser feature. CSS is developed as many independent modules, so this list should evolve as the project defines its native rendering scope.

Checkboxes describe CTML support today. A checked item means the feature is available in the current implementation; an unchecked item is not implemented or has not yet been verified. See [CSS documentation](css.md) for the exact currently supported syntax and values.

## Stylesheet processing and core language

- [x] Load a linked `.css` stylesheet during the Rust build.
- [x] Parse style rules and declarations at build time.
- [x] Parse multiple linked stylesheets in document header order.
- [x] Support local `@import` resolution at build time, preserving import order and rejecting cycles or paths outside the stylesheet root.
- [x] Require UTF-8 stylesheet files, accept and remove an optional UTF-8 BOM, discard comments outside strings, and report unterminated comments; `@charset` does not select another encoding.
- [x] Tokenize supported stylesheet and declaration syntax with CSS escapes, strings, comments, invalid-token handling, and bad-declaration recovery.
- [x] Recover at end of file, discard malformed selector rules and unsupported at-rules, and continue parsing later rules.
- [x] Report stylesheet parse errors with a source file path, line, and column.
- [x] Skip malformed declarations silently; retain unknown properties during parsing and ignore unsupported properties during rendering.
- [x] Support inline `style` attributes on CTML elements.
- [ ] Support CSSOM or runtime stylesheet mutation (debug/development mode only, if needed).

Runtime stylesheet mutation is not needed by the current native renderer: it compiles stylesheets into the application at build time and has no runtime DOM or development refresh loop.

## Selectors

- [x] Type, class, ID, universal (`*`), compound, descendant, direct child (`>`), and comma-separated selectors.
- [ ] Namespace selectors.

Namespace selectors do not apply to current CTML documents: tag names do not accept namespace prefixes, and compiled elements carry no namespace URI.
- [x] Attribute selectors: presence, exact match, token, language, prefix, suffix, substring, and case flags.
- [x] Child (`>`) combinator.
- [x] Adjacent sibling (`+`) and general sibling (`~`) combinators.
- [x] Structural pseudo-classes: `:root`, `:empty`, `:first-child`, `:last-child`, `:only-child`, `:nth-child()`, `:nth-last-child()`, and type-based equivalents.
- [x] Pointer-state pseudo-classes `:hover` and `:active` for text-bearing CTML elements.
- [x] `:focus` and `:focus-within` for clicked links with `href` and elements with numeric `tabindex`.
- [x] `:focus-visible` based on keyboard focus, with Tab and Shift+Tab navigation through links and sequentially focusable elements.
- [ ] Form-state pseudo-classes; CTML does not yet have native form controls.
- [x] Selector logic: `:is()`, `:where()`, and `:not()`.
- [x] Relational selector `:has()` with descendant, child, adjacent-sibling, and general-sibling relative selectors.
- [x] Language and direction selectors: inherited `:lang()` ranges and explicit/inherited `:dir(ltr|rtl)`.
- [x] Link pseudo-classes `:link` and `:any-link` for `<a href>` elements. Form, range, visited-history, and target pseudo-classes do not currently apply because CTML has no corresponding controls, history state, or navigation target.
- [x] `::before` and `::after` generated text from quoted `content` strings and string-valued `attr(name)`, with supported text properties.
- [x] String-valued `attr(name, fallback)` uses a serialized token-sequence fallback when the attribute is missing.
- [x] Generated-content `attr()` converts `type(<string>)`, `type(<number>)`, and `type(<integer>)`, using the fallback when conversion fails.
- [x] Global `::selection` background color, including alpha compositing.
- [x] Element-scoped `::selection` background colors.
- [x] Global and element-scoped `::selection` foreground colors.
- [x] `::first-line` foreground and background colors on the first formatted line of a block.
- [x] `::first-letter` foreground and background colors on the first typographic character and its associated punctuation.
- [x] `text-decoration-line` values `underline`, `overline`, and `line-through` on `::first-line` and `::first-letter`, with supported decoration style, color, and thickness.
- [x] `::first-letter` applies `font-family`, `font-size`, `font-weight`, `font-style`, and `font-stretch` to the first typographic character and its associated punctuation.
- [x] `::first-letter` also applies supported font kerning, variant, feature, line-height, letter-spacing, text-shadow, and text-emphasis properties to that character.
- [x] `::first-letter` applies `font-optical-sizing` and `font-variation-settings` to the initial glyph.
- [x] `text-orientation` applies to `::first-line` and `::first-letter` glyphs.
- [x] `::first-letter` includes the first typographic character and associated Unicode punctuation and intervening spaces across adjacent text runs; `text-transform` changes the character while preserving the punctuation.
- [x] `::first-line` font family, size, weight, style, stretch, kerning, supported variants/features, and letter spacing in glyph rendering; wrapping and line metrics still come from the base text layout.
- [x] `::first-line` applies `font-optical-sizing` and `font-variation-settings` during glyph rendering.
- [x] Apply `text-rendering` to `::first-line` and `::first-letter` glyph shaping.
- [x] `::first-line` text shadow and emphasis marks in glyph rendering.
- [x] Apply the optional left/right side in `text-emphasis-position` on `::first-line` and `::first-letter`.
- [x] Pseudo-element text decorations honor `text-decoration-skip-ink` values `auto`, `none`, and `all`.
- [x] `text-underline-offset` moves underlines on `::first-line` and `::first-letter`.
- [x] Apply the supported color-only `background` shorthand to `::first-line` and `::first-letter`.
- [x] Apply inherited and overridden `text-transform` to the `::first-letter` source character, including case expansion.
- [x] Apply `text-transform` to `::first-line` text during line formation and reshape the resulting lines.
- [x] Apply `line-height` from `::first-line` to the first line box.
- [x] Apply inherited and overridden `word-spacing` to `::first-line` spaces during wrapping.
- [x] Scale alpha for supported first-line and first-letter text, backgrounds, shadows, and decorations with pseudo-element `opacity`.
- [x] Apply `vertical-align` values `baseline`, `sub`, `super`, `middle`, `text-top`, `text-bottom`, `top`, `bottom`, signed lengths, and percentages of line height to `::first-line` and `::first-letter` glyphs.
- [x] `::first-letter` physical border shorthands, side shorthands, and side longhands for width, style, and color paint around the initial glyph.
- [x] `::first-letter` outer and inset `box-shadow` paint around the initial letter's inline box.
- [x] `::first-letter` border-radius values round the initial glyph's painted border.
- [x] `::first-letter` supports `outline` and its width, style, color, and offset properties.
- [x] `::first-letter` physical and logical `margin` and `padding` properties adjust inline spacing; block padding expands its painted background and border area.
- [ ] Other `::first-line` and `::first-letter` styles, including remaining pseudo-element properties not listed above.
- [ ] `::placeholder`; CTML does not currently define form controls.
- [ ] Shadow-tree and part selectors if CTML introduces component encapsulation.
- [x] CSS escapes in type, class, and ID identifiers, attribute names and values, plus specificity for the currently supported selector constructs.
- [x] Filtered `:nth-child(... of S)` and `:nth-last-child(... of S)` matching.
- [x] Remaining selector specificity edge cases: nested functional arguments, filtered nth selectors, escaped identifiers, and pseudo-element weight.

## Cascade, inheritance, and computed values

- [x] Selector specificity and source order.
- [x] Inheritance for supported text properties.
- [x] Cascade ordering for author stylesheets and inline author styles, including `!important`; `revert` uses modeled user-agent initial values.
- [ ] User-origin stylesheets; CTML does not currently expose a user-origin stylesheet source.
- [x] `!important` declarations.
- [x] Named cascade layers with `@layer` statements and blocks, including nested order and reversed `!important` precedence.
- [x] Anonymous cascade layers.
- [x] Escaped layer names.
- [x] `inherit`, `initial`, and `unset` for supported properties.
- [x] `revert` to reset supported declarations to CTML's modeled initial user-agent values.
- [x] `revert-layer` for supported declarations in named layers.
- [x] `text-decoration` shorthand expansion with source-order conflicts against its supported longhands.
- [x] `font` shorthand for supported style, small-caps variant, weight, stretch, size, line-height, and family values.
- [x] `all` shorthand for CSS-wide values across the properties supported by the renderer.
- [x] `background` shorthand for standalone supported colors and `none`, including source-order conflicts with `background-color`.
- [x] `list-style` shorthand for standalone supported marker types, including source-order conflicts with `list-style-type`.
- [x] `text-emphasis` shorthand expansion to supported style and color longhands, including source-order conflicts.
- [x] `font-variant` shorthand expansion to supported variants, including source-order conflicts with longhands.
- [ ] Remaining shorthands for features not yet supported by the renderer, including their longhand conflict resolution.
- [x] Inheritance and CSS-wide keyword resolution for every property currently supported by the renderer.
- [x] Custom properties (`--name`) and `var()` substitution, including inherited values, fallback handling, and cycle invalidation.
- [x] Define specified, computed, used, and actual value processing for supported properties in the CSS implementation documentation.
- [x] Define document-wide scope for current selectors and defer component encapsulation until CTML has component boundaries.

## Values, units, and calculations

- [ ] All standard CSS number, integer, percentage, dimension, string, URL, and identifier value forms.
- [x] Parse finite CSS numeric tokens for supported lengths and percentages, including signs, decimals, and exponents, while rejecting separated units and trailing tokens.
- [x] Parse `normal`, `bold`, and finite numeric `@font-face font-weight` range endpoints from 1 to 1000, including fractional and exponent notation; preserve fractional values for variable-axis clamping.
- [x] Parse finite CSS number and percentage tokens for supported scalar properties such as `opacity`, `line-height`, `font-stretch`, `vertical-align`, and spacing.
- [x] Absolute lengths: `px`, `cm`, `mm`, `Q`, `in`, `pt`, and `pc` for font sizes and line heights.
- [x] Font-relative `em` and `rem` lengths for font sizes and line heights.
- [x] `lh` font-relative unit for `font-size`, `line-height`, `font` shorthand line height, text-shadow lengths, and text-decoration lengths.
- [x] Font metric-relative units `ex`, `ch`, `ic`, and `cap` in `font-size` and the font-size component of `font`.
- [x] Font metric-relative units in line-height, letter spacing, text effects, and box sizing.
- [x] Viewport-relative units, including small, large, and dynamic variants, for supported font, text-effect, and box-size lengths.
- [x] Evaluate `calc()`, `min()`, `max()`, and `clamp()` for supported length/percentage values, plus number expressions for opacity, line-height, and font weight.
- [ ] Container query units, if container queries are implemented.
- [x] Angles, times, frequencies, and resolutions where relevant to implemented properties and generated content.
- [x] Parse finite `deg`, `grad`, `rad`, and `turn` angle tokens for CSS colors and oblique `@font-face` ranges.
- [x] Apply `font-style: oblique <angle>` to the OpenType `slnt` axis, including angle calculations and explicit-angle validation.
- [x] Evaluate supported angle calculations for `font-style: oblique` and normalize angle units to degrees.
- [x] `calc()`, `min()`, `max()`, and `clamp()` for supported length, percentage, number, angle, and relative-color values.
- [x] Typed arithmetic and calculation functions from newer CSS Values modules for the number and dimension types consumed by supported properties.
- [x] Support CSS `round()` for numbers and supported length dimensions, including all rounding strategies and number-only default intervals.
- [x] Support typed `mod()` and `rem()` calculations for numbers and supported length dimensions.
- [x] Support CSS `abs()` for supported dimensions and `abs()`/`sign()` for numbers.
- [x] Support numeric `sqrt()`, `pow()`, `log()`, and `exp()`, plus typed `hypot()` for numbers and supported dimensions.
- [x] Support CSS trigonometric functions for numbers and angle tokens, including inverse functions and `atan2()`.
- [x] Remaining applicable `attr()` forms and `env()` values for the native desktop host.
- [x] Generated-content `attr()` converts simple finite `<percentage>`, `<length>`, `<length-percentage>`, `<angle>`, `<time>`, `<frequency>`, and `<resolution>` attribute tokens; invalid values use the fallback.
- [x] Apply string, integer, number, percentage, length, length-percentage, color, and custom-ident `attr()` substitution to supported style declarations, including typed fallback values.
- [x] Direct typed integer `attr()` in generated content and style declarations accepts signed ASCII integer syntax without a machine-sized range limit and normalizes leading zeros.
- [x] Apply typed `<angle>` `attr()` values and fallbacks in supported style declarations, including CSS color functions.
- [x] Evaluate supported `calc()`, `min()`, `max()`, and `clamp()` expressions in typed integer, number, percentage, length, length-percentage, and angle `attr()` values.
- [x] Resolve `safe-area-inset-*` and `safe-area-max-inset-*` `env()` values as zero on the desktop host and use fallbacks for unknown environment variables.
- [x] Preserve default and `type(<string>)` attribute values as CSS string tokens during declaration substitution.
- [x] Parse CSS `url()` token and string forms and load local resources relative to a stylesheet; canonicalize paths within a configured root, bound reads, and reject external schemes and root escapes.
- [x] Connect local `url()` resources to `@font-face` font loading.
- [x] Locale-independent numeric parsing and well-defined rounding behavior.
- [x] Parse CSS numeric tokens with locale-independent decimal syntax, accepting `.` and rejecting comma or non-ASCII decimal forms.
- [x] Define `round()` ties toward positive infinity and test the `up`, `down`, and `to-zero` strategies for positive and negative values.

## Colors and color effects

- [x] Standard named CSS color set.
- [x] `transparent` color keyword.
- [x] `currentColor` color keyword for text color and page background.
- [x] Hex colors with alpha: `#RGBA` and `#RRGGBBAA`.
- [x] Legacy comma-separated `rgb()`/`rgba()` with numeric or percentage channels and number or percentage alpha.
- [x] Legacy comma-separated `hsl()`/`hsla()` with degree, gradian, radian, or turn hue and alpha.
- [x] Modern space-separated `rgb()`/`hsl()` syntax and slash alpha.
- [x] `hwb()` colors with optional alpha.
- [x] `lab()`, `lch()`, `oklab()`, `oklch()`, and `color()` for `srgb`, `srgb-linear`, `display-p3`, `display-p3-linear`, `a98-rgb`, `prophoto-rgb`, `rec2020`, `rec2100-pq`, `rec2100-hlg`, `rec2100-linear`, `xyz`, `xyz-d50`, and `xyz-d65` spaces.
- [x] Perceptual gamut mapping to sRGB with OklCh chroma reduction and local MINDE.
- [x] Other predefined and custom color profiles, including ICC grayscale, RGB, CMYK, and multichannel device spaces.
- [x] Parse `@color-profile` descriptors and load valid local ICC profiles relative to the declaring stylesheet, bounded to the configured resource root.
- [x] Expose profile-aware parsing for absolute and relative custom `color()` spaces and custom `color-mix()` interpolation through loaded ICC profiles.
- [x] Pass registered custom color profiles through native renderer style resolution.
- [x] CSS HDR `rec2100-linear` profile in absolute `color()`, relative colors, and interpolation; other HDR color spaces are implemented below.
- [x] CSS HDR `rec2100-pq` and `rec2100-hlg` profiles in absolute `color()`, relative colors, and interpolation.
- [x] Render absolute modern color functions with missing (`none`) components treated as zero, including hue and alpha.
- [x] `color-mix()` in `srgb`, `srgb-linear`, `display-p3`, `display-p3-linear`, `a98-rgb`, `prophoto-rgb`, `rec2020`, `rec2100-pq`, `rec2100-hlg`, `rec2100-linear`, `xyz`, `xyz-d50`, `xyz-d65`, `lab`, `lch`, `oklab`, `oklch`, `hsl`, and `hwb` with two supported colors, optional percentages, nested color functions, alpha-aware interpolation, and shorter, longer, increasing, or decreasing hue interpolation for polar spaces.
- [x] Custom `color-mix()` interpolation spaces.
  Requires `@color-profile` registration, ICC profile loading, and profile-aware conversion; custom color spaces are currently marked at-risk in CSS Color 5.
- [x] Basic `rgb(from ...)`, `hsl(from ...)`, `hwb(from ...)`, `lab(from ...)`, `lch(from ...)`, `oklab(from ...)`, `oklch(from ...)`, and `color(from ...)` relative colors in supported profiles with channel references, alpha preservation, supported numeric, percentage, and angle overrides, number, percentage, and angle calculations, and `none` components mapped to zero during conversion.
- [x] Carry matching missing components and alpha from direct `color(srgb ...)` inputs through `color-mix(in srgb, ...)`.
- [x] Carry missing alpha components from direct `color(srgb ...)` inputs through other supported `color-mix()` spaces.
- [x] Carry matching missing components from direct predefined RGB-profile colors through supported RGB interpolation spaces.
- [x] Propagate missing components through nested mixes in supported RGB interpolation spaces.
- [x] Carry matching missing `x`, `y`, and `z` components from direct XYZ-profile colors through XYZ interpolation spaces.
- [x] Carry missing components through matching Lab, LCH, Oklab, and Oklch interpolation spaces, including nested mixes.
- [x] Ignore powerless hues during interpolation in HSL, HWB, LCH, and Oklch spaces.
- [x] Standard and deprecated system color keywords using CTML's fixed light fallback palette.
- [ ] Respond to OS forced-color palettes and element color schemes when the host exposes them.
- [x] Alpha colors for text and page background, including page-background compositing over white.
- [x] Element `opacity` scales text glyphs, decorations, shadows and emphasis marks, fragment backgrounds, selection highlights, pseudo-element paints, and nested descendant paints; ancestor opacity multiplies with descendant opacity.
- [ ] Isolate overlapping element paints through group compositing.
- [ ] `accent-color`, `color-scheme`, and color adjustment properties.
- [ ] Blend modes and isolation.

## Box model and sizing

- [x] `display: none` removes an element's rendered text, descendants, and generated content while retaining selector-tree matching.
- [x] `display: block` and `display: inline` override tag-based line separation for rendered text.
- [x] `display: contents` removes tag-based line separation while preserving child text flow.
- [x] `display: flow-root` and `display: inline-block` use block and inline text flow, respectively.
- [x] Supported multi-keyword `display` forms accept order-independent block/inline outer and flow/flow-root inner values, including `list-item`; `inline list-item` participates in inline text flow.
- [x] `display: list-item` and inherited `list-style-type` values `disc`, `circle`, `square`, `none`, `decimal`, `decimal-leading-zero`, `lower-alpha`, `upper-alpha`, `lower-greek`, `lower-roman`, and `upper-roman`; `<li>` defaults to bullets in unordered lists and decimal numbers in ordered lists.
- [ ] Remaining `display` values and CSS formatting box generation.
- [x] `body` and block element `width`, `min-width`, and `max-width` constrain text wrapping for non-negative lengths and percentages; `inline-size`, `min-inline-size`, and `max-inline-size` size the inline axis for horizontal and page-root vertical writing modes.
- [x] Empty block `height`, `min-height`, and `max-height` reserve and cap vertical flow space; nested `height` and `block-size` percentages resolve against an explicitly sized containing block. `block-size` and its min/max forms size the block axis for horizontal and page-root vertical writing modes.
- [x] Content-bearing fixed heights and `max-height`/`max-block-size` use the requested used block extent for following flow, keep overflowing text visible under the default `overflow: visible`, and size auto-height blocks from content before applying maximums.
- [x] One- and two-value `overflow: visible | hidden | clip` and `overflow-x`/`overflow-y` clip descendant painting and pointer/selection hits for content-bearing blocks in horizontal and page-root vertical writing modes.
- [x] Overflow clipping for nested orthogonal blocks under a horizontal page root.
- [x] Page-root `overflow: auto | scroll` responds to mouse-wheel input along scrollable axes.
- [x] Basic scrollbar visuals for page-root `overflow: scroll` and overflowing `auto` axes; thumb size and position follow formatted content and scroll offset.
- [x] Nested `overflow-y: auto | scroll` scrolling and scrollbar visuals for content-bearing blocks under a horizontal page root.
- [x] Nested `overflow-x: auto | scroll` scrolling and scrollbar visuals for content-bearing blocks under a horizontal page root.
- [x] Nested `overflow-x` and `overflow-y` scrollers under vertical page roots, with scrollbar visuals and writing-mode-aware paint and hit-test offsets.
- [x] Nested scroll-chain composition under a horizontal page root, passing unconsumed wheel deltas from inner scrollers to their ancestors and then to the page root.
- [x] Overflow clipping for nested orthogonal blocks under a vertical page root, including descendant paints and pointer/selection hits.
- [x] Nested content-bearing block `height`, `block-size`, `min-height`, `min-block-size`, `max-height`, and `max-block-size` percentages resolve against an explicitly sized containing block's used block extent.
- [ ] Additional sizing behavior beyond text flow for content-bearing blocks.
- [x] `box-sizing: content-box | border-box`; `content-box` is the initial value, and `border-box` includes supported physical padding in width and block-size calculations.
- [x] Physical `padding` shorthand and `padding-top/right/bottom/left` accept non-negative font-relative lengths and percentages of the containing block's inline size, inset text, and add to block flow.
- [x] `min-content`, `max-content`, and `fit-content()` sizing for text-bearing widths, including logical inline-size and min/max constraints.
- [x] Include supported physical and logical border widths in `border-box` dimensions and block flow.
- [x] Percentage physical padding resolves against the containing block's inline size.
- [x] Logical `padding-inline` and `padding-block` shorthand, including start/end longhands, map through `writing-mode` and `direction` and follow declaration order with physical padding.
- [x] Physical and logical margin shorthands and sides accept signed font-relative lengths and percentages, follow writing mode, direction, and source order, and adjust text flow. `auto` currently resolves to zero.
- [x] Border shorthand and physical/logical side properties for widths, styles, and colors; paint solid, dotted, dashed, double, and shaded groove/ridge/inset/outset edges on nested content-bearing blocks.
- [x] `border-radius` with one-to-four values, elliptical radii, corner longhands, and rounded border painting on nested content-bearing blocks.
- [x] `outline`, `outline-width`, `outline-style`, `outline-color`, and signed `outline-offset`; paint outlines around nested content-bearing blocks, including rounded corners.
- [x] Positive numeric `aspect-ratio` values derive an automatic dimension from the available or specified counterpart for horizontal content-bearing and empty blocks.
- [x] `box-shadow` with multiple outer and inset shadows, offsets, blur, spread, colors, and rounded block silhouettes.
- [ ] `border-image` and image slicing/repetition. The renderer has no decoded-image resource path yet; this depends on native image loading and decoding.
- [x] `box-decoration-break: slice | clone` repeats inline text fragment side borders for `clone`; `slice` draws only the outer inline start/end edges.
- [x] `visibility: visible|hidden|collapse`; `collapse` is treated as hidden because CTML has no table formatting context.
- [ ] `content-visibility` and containment behavior. `hidden` skips descendant painting and hit testing, and applies size, style, layout, and paint containment; auto-sized content reserves zero block flow height unless `contain-intrinsic-size` provides a fallback. `auto` applies style, layout, and paint containment, skips content outside a viewport margin of 50%, and applies size containment with intrinsic-size fallback while skipped. Focused and selected content remains available. Style containment scopes counter and quote changes to the element subtree. `contain: paint`, `content`, and `strict` clip descendant painting. `contain: size` and `strict` remove intrinsic block flow height and horizontal intrinsic width contributions while allowing visible overflow. Vertical page writing maps physical intrinsic width and height fallbacks to block and inline axes. Orthogonal nested inline-size containment maps its contained inline contribution to the page block axis and uses the matching physical intrinsic fallback. Layout containment state is parsed, but its containing-block, formatting-context, and stacking effects await positioned and float layout.
- [x] Parse and preserve layout-containment state for `contain: layout`, `content`, and `strict`, including layout containment implied by `content-visibility: hidden|auto`. Its containing-block, formatting-context, and stacking behavior awaits positioned and float layout.

## Layout

- [ ] Normal flow and inline formatting behavior.
- [ ] Block, inline, inline-block, and anonymous box behavior.
- [ ] Flexbox: container/child properties, wrapping, ordering, and sizing.
- [ ] Grid: explicit/implicit tracks, placement, auto sizing, named areas, and subgrid.
- [ ] Box alignment: alignment, justification, gaps, and baseline alignment.
- [ ] Positioned layout: static, relative, absolute, fixed, and sticky positioning.
- [ ] Stacking contexts and `z-index`.
- [ ] Floats, clear, and shape-based text wrapping.
- [ ] Multi-column layout.
- [ ] Tables and table layout, if CTML supports table elements.
- [ ] Overflow, clipping, scrolling, scrollbars, and scroll snapping.
- [ ] Fragmentation, page breaks, and paged media where relevant.
- [ ] Logical properties beyond `inline-size`/`block-size` and writing-mode-aware layout on nested blocks.
- [ ] Anchor positioning and advanced layout modules if they become stable and useful.

## Text, fonts, and writing systems

- [x] `color`, `font-family`, `font-size` in supported lengths and percentages relative to the inherited size, numeric `font-weight` values from 1 to 1000, `normal`/`bold`, and relative `bolder`/`lighter`, `font-style` (`normal`, `italic`, `oblique`), `font-stretch` keywords and percentages mapped to available widths, `line-height`, and `letter-spacing`.
- [x] `text-transform` and basic `text-decoration` line values.
- [x] `font-kerning` (`auto`, `normal`, and `none`).
- [x] `font-style: oblique <angle>` from -90deg to 90deg, including conversion to the opposite-sign OpenType `slnt` axis.
- [x] Common, discretionary, and contextual ligature controls through `font-variant-ligatures`.
- [x] Subscript and superscript glyph variants through `font-variant-position`.
- [x] Numeric glyph variants through `font-variant-numeric` for lining/oldstyle, proportional/tabular, fraction, ordinal, and slashed-zero features.
- [x] East Asian glyph variants through `font-variant-east-asian` for JIS encodings, simplified/traditional forms, full/proportional width, and ruby.
- [x] Low-level `font-feature-settings` with four-character OpenType tags, numeric settings, `on`, and `off`.
- [x] Optical sizing and font variation settings.
- [x] Local `@font-face` loading with a CSS family alias and static weight, style, and stretch descriptors.
- [x] Variable font weight ranges and oblique style angle ranges in `@font-face`, including range-aware matching and weight clamping.
- [x] `@font-face` tries `local()` and local URL candidates in source order, aliases the first available font to the declared family, and matches static weight/style/stretch descriptors through the native font database.
- [x] `font-variant`/`font-variant-caps`: `normal`, `small-caps`, `all-small-caps`, `petite-caps`, `all-petite-caps`, `unicase`, and `titling-caps` through OpenType features.
- [x] Text alignment: left, right, center, justify, start, and end.
- [x] `text-align-last` for automatic, logical and physical edge, centered, and justified final lines.
- [x] `text-indent` lengths and percentages, with hanging and each-line behavior.
- [x] `text-justify`: `auto`, `none`, `inter-word`, `inter-character`, and legacy `distribute`.
- [x] `white-space` collapsing and preservation (`normal`, `nowrap`, `pre`, `pre-wrap`, `pre-line`, `break-spaces`), `text-wrap`/`text-wrap-mode` (`wrap`, `nowrap`), `overflow-wrap` (`normal`, `break-word`, `anywhere`), and `word-break` (`normal`, `break-all`, `keep-all`, `break-word`). Intrinsic min-content sizing differences for `anywhere` are not modeled.
- [x] `line-break`: parses `auto`, `loose`, `normal`, `strict`, and `anywhere`; `anywhere` allows glyph-level breaks. `strict`/`loose` CJK tailoring is not modeled yet.
- [x] `hyphens: none|manual|auto` values; `none` disables line breaks at soft hyphens. `auto` adds dictionary break points for recognized `lang` values.
- [x] Render a visible hyphen when a line breaks at a manual U+00AD soft-hyphen opportunity; keep the character invisible when no break occurs or `hyphens: none` disables it.
- [x] Dictionary-driven automatic hyphenation for embedded language dictionaries; explicit soft hyphens take priority.
- [x] Pass inherited `lang` values to text shaping so language-sensitive shaping can use the declared language.
- [x] `text-overflow: clip|ellipsis` for `nowrap`/`pre` lines and positive `line-clamp` line counts. Clipping and the layout-mode prerequisites used by browser CSS are not modeled.
- [x] Word spacing, including inherited values, lengths and percentages.
- [x] `text-rendering` controls kerning and common ligatures; `text-orientation` parses `mixed`, `upright`, and `sideways`.
- [x] `text-decoration-color` for underline, overline, and line-through.
- [x] Solid, double, dotted, dashed, and wavy text decoration styles.
- [x] `text-decoration-skip-ink` values `auto`, `none`, and `all`.
- [x] `text-decoration-thickness` values `auto`, `from-font`, and non-negative lengths or percentages.
- [x] `text-underline-offset` values `auto`, signed lengths, and percentages.
- [x] `text-shadow` with multiple shadows, signed offsets, non-negative blur lengths, explicit colors, and `currentColor`.
- [x] Keyword and single-grapheme string `text-emphasis` marks (`dot`, `circle`, `double-circle`, `triangle`, and `sesame`), filled/open styles, inherited color, and over/under position.
- [x] Parse and preserve the optional `left`/`right` side in `text-emphasis-position` and apply it when vertical flow is enabled.
- [x] Vertical text-emphasis `left`/`right` side placement.
- [x] `direction: ltr|rtl` establishes the inherited base bidi direction for each rendered paragraph.
- [x] `unicode-bidi: isolate` adds Unicode isolate controls around the styled inline range without adding controls to rendered text.
- [x] Page-root `writing-mode: vertical-rl|vertical-lr` creates vertical columns and maps pointer coordinates through the rendered orientation.
- [x] Resolve logical `inline-size`/`block-size` and their min/max forms against page-root horizontal or vertical flow axes, including nested block sizes.
- [ ] Apply a nested block's own vertical text flow and glyph orientation independently of the page root. Under a horizontal page root, nested vertical text now wraps, composes glyphs, fragment backgrounds, selection highlights, and decorations locally, and maps pointer hits back to the buffer; cross-axis block sizing still constrains page flow. Nested orthogonal blocks under a vertical page root and other logical properties remain unsupported.
- [x] Apply `text-orientation: mixed|upright|sideways` per glyph in vertical writing, using Unicode Vertical_Orientation data.
- [x] Basic list markers and inherited list marker types, including decimal-leading-zero, alphabetic, Greek, and Roman counters.
- [x] Custom string markers through `list-style-type`.
- [x] Hebrew, Armenian, Georgian, basic CJK, localized decimal, and kana alphabet predefined `list-style-type` systems, with decimal fallback outside supported ranges.
- [x] `ethiopic-numeric` built-in counter style using two-digit groups.
- [x] Japanese informal and formal longhand counter styles through 9,999.
- [x] Simplified and traditional Chinese informal and formal longhand counter styles through 9,999.
- [x] Korean Hangul formal, Hanja informal, and Hanja formal longhand counter styles through 9,999.
- [x] `disclosure-open` and `disclosure-closed` list markers with open/down and closed/right triangle glyphs.
- [x] All simple and complex predefined `list-style-type` and `counter()`/`counters()` styles from CSS Counter Styles Level 3, including disclosure, localized decimal, kana, Ethiopic, Japanese, Chinese, and Korean systems.
- [x] Anonymous `symbols()` counter styles for cyclic, fixed, symbolic, alphabetic, and numeric systems with string symbols.
- [x] Generated `open-quote`, `close-quote`, `no-open-quote`, and `no-close-quote` content with inherited custom `quotes` pairs.
- [x] `counter-reset`, `counter-increment`, and `counter-set` with scoped values, plus decimal, decimal-leading-zero, alphabetic, Greek, and Roman `counter()`/`counters()` formatting.
- [x] Typed `attr()` values and fallbacks in generated `content`.
- [ ] `target-counter()` and author-defined `@counter-style` systems.
- [ ] Ruby annotations and other specialized typography where relevant.

## Backgrounds, borders, and visual effects

- [x] `background-color` on rendered text fragments, including alpha compositing and nested elements.
- [x] Paint backgrounds across the border boxes of empty sized blocks, including their padding area.
- [x] Paint block backgrounds across used border boxes for content-bearing blocks, with nested clipping and writing-mode placement.
- [x] Paint backgrounds on empty inline elements and extend fragmented inline backgrounds through their padding area.
- [ ] Background painting for empty elements and complete CSS boxes, including padding, sizing, and fragmented inline boxes.
- [ ] Multiple backgrounds, images, repeat, position, size, attachment, and origin/clip.
  - [x] Decode embedded base64 PNG/JPEG `data:` URLs in `background-image`; paint comma-separated layers in CSS order with intrinsic-size repeat and element opacity/clipping.
  - [x] `background-repeat` supports `repeat`, `no-repeat`, `repeat-x`, `repeat-y`, and per-axis keyword pairs across comma-separated layers.
  - [x] `background-position` supports one or two axis keywords, percentages, and font-relative or absolute lengths per layer.
  - [x] `background-size` supports auto dimensions, lengths, percentages, `cover`, and `contain` for embedded images.
  - [x] `background-attachment` supports per-layer `scroll`, `fixed`, and `local`; fixed images stay viewport-anchored through nested scrolling.
  - [x] `background-origin` and `background-clip` support per-layer border-box, padding-box, and content-box geometry; background colors use the final clip value.
- [x] Linear, radial, conic, and repeating gradients.
  - [x] `linear-gradient()` with direction or angle, CSS colors, and optional percentage stop positions.
  - [x] `repeating-linear-gradient()` repeats the pattern between its first and last color stops.
  - [x] `radial-gradient()` supports circle or ellipse shapes, side/corner extents, center positions, and CSS color stops.
  - [x] `repeating-radial-gradient()` repeats percentage stop patterns that start at `0%`.
  - [x] `conic-gradient()` supports a starting angle, center position, angular color stops, and clockwise rendering.
  - [x] `repeating-conic-gradient()` repeats the color-stop pattern around the full sweep.
- [ ] Border images and nine-slice behavior.
- [ ] Filters and backdrop filters.
- [ ] Masks, clipping paths, and mask compositing.
- [ ] Transform properties, transform origin, and transform box.
- [ ] 2D and 3D transforms and perspective.
- [ ] Blend modes, compositing, and isolation.
- [ ] `mix-blend-mode` and `background-blend-mode`.
- [ ] Image rendering and object fit/position for replaced content.

## Interaction and user interface

- [x] Native cursor mapping for supported `cursor` keywords.
- [x] Hide the native cursor with `cursor: none`.
- [x] Text selection by mouse drag and double-click word selection.
- [x] Standard cursor keywords, including context-menu, grab/grabbing, column and row resize, plus the platform cursor extensions `all-resize` and `dnd-ask`.
- [ ] Custom cursor images and hotspot coordinates.
- [x] `pointer-events: auto|none` for rendered text targets, including descendant overrides.
- [x] `user-select: auto|text|none|all` for rendered text, including descendant overrides and atomic element selection.
- [ ] `caret-color` and `accent-color`.
- [ ] Focus indicators, outlines, and keyboard navigation states.
- [ ] Form control appearance, sizing, and styling if CTML provides native form controls.
- [ ] Resize behavior and `resize` property.
- [ ] Touch and pointer action properties where relevant.
- [ ] Scrollbar styling and platform-native scrollbar policy.
- [ ] Reduced motion, contrast, and other user preference media features.

## Responsive rules and conditional application

- [ ] `@media` queries for viewport, resolution, color, input capabilities, and user preferences.
- [ ] Media query ranges and logical combinations.
- [ ] `@supports` feature queries.
- [ ] `@container` size and style queries.
- [ ] Container naming and container-relative units.
- [ ] Page size and print media rules if CTML targets printing/export.
- [ ] Environment variables such as safe-area insets where applicable.

## Animation and transitions

- [ ] Transitions: property, duration, delay, timing function, and behavior.
- [ ] Keyframe animations and `@keyframes`.
- [ ] Animation composition, direction, fill mode, iteration, and play state.
- [ ] Animatable property definitions and interpolation.
- [ ] Scroll-driven and view-driven animations if appropriate for CTML.
- [ ] View transitions between UI states if appropriate for CTML.
- [ ] Respect reduced-motion preferences and allow animations to be disabled.

## At-rules and resource definitions

- [x] Explicitly omit legacy `@charset` encoding overrides; CTML stylesheets are always UTF-8.
- [x] Expand local `@import` rules in source order, including nested imports; reject unsupported external and qualified imports.
- [x] Named `@layer` statements and blocks.
- [ ] `@media`, `@supports`, `@container`, and other conditional at-rules.
- [x] `@font-face` local URLs and static font weight/style/stretch descriptors.
- [ ] Font feature/value rules and variable descriptors.
- [ ] `@keyframes`.
- [ ] `@page` and margin at-rules for print output.
- [ ] `@property` registration for custom properties.
- [ ] Counter style and font palette at-rules where relevant.
- [x] Skip unknown and unsupported at-rules while continuing to parse later rules.

## Accessibility, platform integration, and native CTML scope

- [ ] Ensure visual styling never becomes the only source of essential state or meaning.
- [ ] Support keyboard-visible focus and system text scaling where applicable.
- [ ] Respect OS high-contrast, reduced-motion, and color-scheme preferences.
- [ ] Define how CSS styling interacts with CTML semantics and accessibility metadata.
- [ ] Define native resource loading, font discovery, and image decoding behavior.
- [ ] Keep CSS parsing and compilation out of release runtime builds.
- [ ] Keep development-only refresh and diagnostics out of the minimal release runtime.
- [ ] Establish memory and performance budgets for stylesheet compilation and native rendering.
- [x] Keep web-only and browser API-dependent features out of scope; CTML applications render natively.

## References

- [W3C CSS Snapshot 2025](https://www.w3.org/TR/css-2025/) — a stable overview of CSS modules and their standardization status.
- [W3C CSS specifications and drafts](https://www.w3.org/Style/CSS/current-work) — current modules and active work.
- [MDN CSS reference](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference) — searchable reference for properties, selectors, data types, functions, and at-rules.
