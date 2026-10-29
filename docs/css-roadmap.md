# CSS Feature Checklist

This document tracks CSS features that CTML may support. It is a broad implementation checklist based on the CSS standards, not a promise to reproduce every browser feature. CSS is developed as many independent modules, so this list should evolve as the project defines its native rendering scope.

Checkboxes describe CTML support today. A checked item means the feature is available in the current implementation; an unchecked item is not implemented or has not yet been verified. See [CSS documentation](css.md) for the exact currently supported syntax and values.

## Stylesheet processing and core language

- [x] Load a linked `.css` stylesheet during the Rust build.
- [x] Parse style rules and declarations at build time.
- [x] Parse multiple linked stylesheets in document header order.
- [x] Support local `@import` resolution at build time, preserving import order and rejecting cycles or paths outside the stylesheet root.
- [x] Require UTF-8 stylesheet files, accept and remove an optional UTF-8 BOM, discard comments outside strings, and report unterminated comments; `@charset` does not select another encoding.
- [ ] Implement complete CSS tokenization, escapes, strings, comments, and error recovery.
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
- [ ] `:focus-visible`, keyboard focus navigation, and form-state pseudo-classes; CTML does not yet have native form controls.
- [x] Selector logic: `:is()`, `:where()`, and `:not()`.
- [x] Relational selector `:has()` with descendant, child, adjacent-sibling, and general-sibling relative selectors.
- [x] Language and direction selectors: inherited `:lang()` ranges and explicit/inherited `:dir(ltr|rtl)`.
- [x] Link pseudo-classes `:link` and `:any-link` for `<a href>` elements. Form, range, visited-history, and target pseudo-classes do not currently apply because CTML has no corresponding controls, history state, or navigation target.
- [x] `::before` and `::after` generated text from quoted `content` strings, with supported text properties.
- [ ] `::first-line`, `::first-letter`, and `::selection`; these need line layout or selection-aware shaping.
- [ ] `::placeholder`; CTML does not currently define form controls.
- [ ] Shadow-tree and part selectors if CTML introduces component encapsulation.
- [x] CSS escapes in type, class, and ID identifiers, attribute names and values, plus specificity for the currently supported selector constructs.
- [x] Filtered `:nth-child(... of S)` and `:nth-last-child(... of S)` matching.
- [ ] Remaining selector specificity edge cases.

## Cascade, inheritance, and computed values

- [x] Selector specificity and source order.
- [x] Inheritance for supported text properties.
- [ ] Complete origin and importance ordering, including user-agent, author, and user origins where applicable.
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
- [ ] Remaining shorthand and longhand expansion, including conflict resolution.
- [ ] Complete inheritance rules per property.
- [x] Custom properties (`--name`) and `var()` substitution, including inherited values, fallback handling, and cycle invalidation.
- [ ] Define specified, computed, used, and actual value processing for supported properties.
- [ ] Define style scoping and encapsulation rules for future CTML components.

## Values, units, and calculations

- [ ] All standard CSS number, integer, percentage, dimension, string, URL, and identifier value forms.
- [x] Absolute lengths: `px`, `cm`, `mm`, `Q`, `in`, `pt`, and `pc` for font sizes and line heights.
- [ ] Font-relative lengths: `em` and `rem` for font sizes and line heights; remaining units `ex`, `ch`, `ic`, `cap`, and `lh`.
- [ ] Viewport-relative units, including small, large, and dynamic viewport variants.
- [ ] Container query units, if container queries are implemented.
- [ ] Angles, times, frequencies, and resolutions where relevant.
- [ ] `calc()`, `min()`, `max()`, and `clamp()`.
- [ ] Typed arithmetic and calculation functions from newer CSS Values modules.
- [ ] `attr()` and `env()` value functions where relevant.
- [ ] URL resolution, resource loading policy, and safe handling of external resources.
- [ ] Locale-independent numeric parsing and well-defined rounding behavior.

## Colors and color effects

- [x] Standard named CSS color set.
- [x] `transparent` color keyword.
- [x] `currentColor` color keyword for text color and page background.
- [x] Hex colors with alpha: `#RGBA` and `#RRGGBBAA`.
- [x] Legacy comma-separated `rgb()`/`rgba()` with numeric or percentage channels and number or percentage alpha.
- [x] Legacy comma-separated `hsl()`/`hsla()` with degree, gradian, radian, or turn hue and alpha.
- [x] Modern space-separated `rgb()`/`hsl()` syntax and slash alpha.
- [x] `hwb()` colors with optional alpha.
- [x] `lab()`, `lch()`, `oklab()`, `oklch()`, and `color()` for `srgb`, `srgb-linear`, `display-p3`, `xyz`, `xyz-d50`, and `xyz-d65` spaces.
- [ ] Other predefined and custom color profiles, perceptual gamut mapping, and full CSS missing-component handling.
- [ ] Color profiles, wide-gamut spaces, and interpolation rules.
- [ ] `color-mix()` and relative color syntax.
- [ ] System colors and forced-color behavior where supported by the host OS.
- [x] Alpha colors for text and page background, including page-background compositing over white.
- [ ] Element `opacity` and group compositing.
- [ ] `accent-color`, `color-scheme`, and color adjustment properties.
- [ ] Blend modes and isolation.

## Box model and sizing

- [ ] `display` values and formatting box generation, including `none`, block, inline, flow-root, contents, and list-item.
- [ ] `width`, `height`, inline/block size, and min/max sizing.
- [ ] `box-sizing` and intrinsic sizing keywords.
- [ ] Margin, padding, and logical margin/padding properties.
- [ ] Borders, per-side styles/colors/widths, and logical border properties.
- [ ] `border-radius` and corner shaping.
- [ ] `outline` and `outline-offset`.
- [ ] `aspect-ratio`.
- [ ] Box shadows and border images.
- [ ] Box decorations across fragmented content.
- [ ] `visibility`, `content-visibility`, and containment behavior.

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
- [ ] Logical properties and writing-mode-aware layout.
- [ ] Anchor positioning and advanced layout modules if they become stable and useful.

## Text, fonts, and writing systems

- [x] `color`, `font-family`, `font-size` in supported lengths and percentages relative to the inherited size, numeric `font-weight` values from 1 to 1000, `normal`/`bold`, and relative `bolder`/`lighter`, `font-style` (`normal`, `italic`, `oblique`), `font-stretch` keywords and percentages mapped to available widths, `line-height`, and `letter-spacing`.
- [x] `text-transform` and basic `text-decoration` line values.
- [x] `font-kerning` (`auto`, `normal`, and `none`).
- [x] Common, discretionary, and contextual ligature controls through `font-variant-ligatures`.
- [x] Subscript and superscript glyph variants through `font-variant-position`.
- [x] Numeric glyph variants through `font-variant-numeric` for lining/oldstyle, proportional/tabular, fraction, ordinal, and slashed-zero features.
- [x] East Asian glyph variants through `font-variant-east-asian` for JIS encodings, simplified/traditional forms, full/proportional width, and ruby.
- [ ] Font shorthand, optical sizing, feature settings, and variation settings.
- [ ] Font loading (`@font-face`), fallback selection, and local font lookup policy.
- [x] `font-variant`/`font-variant-caps`: `normal`, `small-caps`, `all-small-caps`, `petite-caps`, `all-petite-caps`, `unicase`, and `titling-caps` through OpenType features.
- [x] Text alignment: left, right, center, justify, start, and end.
- [ ] Alignment last line, indentation, and advanced justification controls.
- [ ] White-space collapsing/wrapping controls, including `white-space`, `overflow-wrap`, and `word-break`.
- [ ] Hyphenation, line breaking, and language-sensitive text shaping controls.
- [ ] Text overflow and line clamping.
- [ ] Word spacing, text rendering, and text orientation.
- [x] `text-decoration-color` for underline, overline, and line-through.
- [x] Solid, double, dotted, dashed, and wavy text decoration styles.
- [x] `text-decoration-skip-ink` values `auto`, `none`, and `all`.
- [x] `text-decoration-thickness` values `auto`, `from-font`, and non-negative lengths or percentages.
- [x] `text-underline-offset` values `auto`, signed lengths, and percentages.
- [ ] Text emphasis marks.
- [ ] Text shadows and `text-emphasis`.
- [ ] Writing modes, direction, bidirectional isolation, and vertical text.
- [ ] List markers and counters.
- [ ] Generated content with `content`, counters, and quotes.
- [ ] Ruby annotations and other specialized typography where relevant.

## Backgrounds, borders, and visual effects

- [ ] Per-element background color (currently only the page background is supported).
- [ ] Multiple backgrounds, images, repeat, position, size, attachment, and origin/clip.
- [ ] Linear, radial, conic, and repeating gradients.
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
- [ ] Complete cursor values, including context-menu, drag, column, and platform-specific cursors.
- [ ] Custom cursor images and hotspot coordinates.
- [ ] `pointer-events`, `user-select`, `caret-color`, and `accent-color`.
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

- [ ] `@charset` handling or explicit omission as obsolete stylesheet syntax.
- [ ] `@import`.
- [x] Named `@layer` statements and blocks.
- [x] Local `@import` resolution and named `@layer` statements and blocks.
- [ ] `@media`, `@supports`, `@container`, and other conditional at-rules.
- [ ] `@font-face` and font feature/value rules.
- [ ] `@keyframes`.
- [ ] `@page` and margin at-rules for print output.
- [ ] `@property` registration for custom properties.
- [ ] Counter style and font palette at-rules where relevant.
- [ ] Define a policy for unknown and future at-rules.

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
