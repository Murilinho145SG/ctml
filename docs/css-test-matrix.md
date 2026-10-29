# CSS Test Matrix

This matrix covers every CSS property and selector currently handled by the native CTML renderer. It does not mark the unchecked features in the [CSS roadmap](css-roadmap.md) as implemented. Each parity case has matching `native.ctml`, `browser.html`, and `style.css` files and runs as its own Rust test against Chromium 156.0.8078.47. Chromium is a test-only reference; it is not shipped with or used by CTML applications.

## Property coverage

| CSS property or group | Chromium fixture | Unit coverage |
| --- | --- | --- |
| `color`, `background-color` | `colors`, `transparent` | `css_color::tests`; color cascade and alpha tests |
| `font-family` | `font-family` | family selection and fallback-list tests |
| `font-size` | `font-size`, `font-size-units`, `font-relative-sizes` | percent, absolute-unit, and inherited-relative tests |
| `line-height` | `line-height` | number, percentage, length, and multiplier tests |
| `font-weight` | `font-weight` | keyword, relative-weight, numeric-boundary, invalid-value tests |
| `font-style` | `font-style` | `normal`, `italic`, `oblique` |
| `font-stretch` | `font-stretch` | all nine keywords and percentage mapping buckets |
| `font-kerning` | `font-kerning` | `auto`, `normal`, `none` |
| `font-variant-ligatures` | `font-features` | enable, disable, and reset switches |
| `font-variant`, `font-variant-caps` | `font-caps`, `font-variant-shorthand` | all supported caps values, shorthand combinations, and shorthand reset |
| `font-variant-position` | `font-position` | `normal`, `sub`, `super` |
| `letter-spacing` | `letter-spacing` | normal, positive and negative `px`/`em` values |
| `text-transform` | `text-transform` | all four supported values |
| `text-align` | `text-alignment`, `text-alignment-values` | all six supported values |
| `text-decoration`, `text-decoration-line`, `text-decoration-style`, `text-decoration-skip-ink`, `text-decoration-thickness`, `text-underline-offset` | `text-decoration`, `text-decoration-lines`, `text-decoration-style`, `text-decoration-skip-ink`, `text-decoration-thickness`, `text-underline-offset` | none, each line, combinations, all five styles, skip-ink values, thickness, and signed offsets |
| `text-decoration-color` | `text-decoration`, `text-decoration-lines` | color, `currentColor`, and active decoration lines |
| `cursor` | `cursor` | every mapped cursor keyword, aliases, and `none` |

## Selector and cascade coverage

| Feature | Chromium fixture | Unit coverage |
| --- | --- | --- |
| Type, class, ID, universal, and compound selectors | `selector-matrix` | selector match matrix |
| Descendant and direct-child combinators | `selector-matrix`, `child-selector` | descendant and child matching |
| Adjacent and general sibling combinators | `sibling-selectors` | sibling matching, chained relations, and renderer cascade |
| Structural pseudo-classes and type equivalents | `structural-selectors` | root, empty, child/type positions, An+B formulas, `of S` filtered child positions, and renderer cascade |
| Pointer-state pseudo-classes | — | selector-state cascade and live text rerender tests for hover/active |
| Attribute selector presence, operators, and case flags | `attribute-selectors` | `ctml-css` selector tests and renderer cascade test |
| Inline style attributes and importance ordering | `inline-style` | Declaration-list parser and renderer cascade tests |
| `:is()`, `:where()`, and `:not()` | — | `ctml-css` selector and specificity tests |
| Relational `:has()` selectors | `has-selectors` | descendant, child, and adjacent-sibling relative matching |
| Language and direction pseudo-classes | — | inherited language ranges, direction values, default direction, and renderer cascade |
| Link pseudo-classes | — | `:link` and `:any-link` require `<a href>`; `:visited` remains false without history state |
| Generated content pseudo-elements | — | before/after text order, quoted content strings, and pseudo-element text styling |
| Comma-separated selector groups | `selector-matrix` | group matching and specificity |
| Escaped selector identifiers | — | type, class, ID, attribute names and values, and escaped selector-list commas |
| `!important`, specificity, and source order | `selector-matrix`, `child-selector` | cascade precedence and important priority |
| `inherit`, `initial`, `unset`, and `revert` | — | supported property cascade keyword unit tests |

## Stylesheet processing coverage

| Feature | Integration coverage | Unit coverage |
| --- | --- | --- |
| Multiple linked stylesheets preserve header order | Native demo build script | Compiler document stylesheet-order test |
| Local `@import` expands at its source position, tracks dependencies, and rejects cycles or root escapes | Native demo build script | Nested import order, dependency, cycle, and path validation tests |
| UTF-8 encoding, optional BOM, and CSS comments | Native demo build script | BOM, comment, unterminated-comment, and invalid-UTF-8 tests |
| CSS-tokenized declaration names and values; malformed declaration recovery | — | Escaped property, quoted punctuation, and invalid declaration tests |
| File, line, and column diagnostics for malformed stylesheets and imports | — | Top-level, nested-import, and late-import diagnostic tests |
| Invalid declaration and unknown property policy | — | Malformed declaration recovery, parser preservation, and renderer ignore tests |

## Running the complete matrix

Run unit coverage without Chromium:

```sh
cargo test --offline -p ctml-renderer --lib css_style::matrix_tests
cargo test --offline -p ctml-renderer --lib css_style::tests
cargo test --offline -p ctml-css color::tests
```

Run every Chromium comparison serially:

```sh
cargo test --offline -p ctml-renderer --lib page::tests::parity -- --test-threads=1
```

The suite reports every exact RGB pixel mismatch and highlights mismatches above the per-channel tolerance. Exact parity cases fail on any RGB difference. Most other cases allow up to 0.05% of pixels above that tolerance; `font-stretch` allows 0.055%, sibling selectors 0.07%, `text-decoration-style` 0.11%, `attribute-selectors` 1.25%, `font-variant-shorthand` 0.27%, and `line-height-normal` 0.48% for observed rasterization differences. Artifacts are written to `target/css-parity/<case>/`.
