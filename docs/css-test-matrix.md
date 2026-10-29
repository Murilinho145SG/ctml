# CSS Test Matrix

This matrix covers every CSS property and selector currently handled by the native CTML renderer. It does not mark the unchecked features in the [CSS roadmap](css-roadmap.md) as implemented. Each parity case has matching `native.ctml`, `browser.html`, and `style.css` files and runs as its own Rust test against Chromium. Chromium is a test-only reference; it is not shipped with or used by CTML applications.

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
| `font-variant`, `font-variant-caps` | `font-caps` | all seven supported caps values |
| `font-variant-position` | `font-position` | `normal`, `sub`, `super` |
| `letter-spacing` | `letter-spacing` | normal, positive and negative `px`/`em` values |
| `text-transform` | `text-transform` | all four supported values |
| `text-align` | `text-alignment`, `text-alignment-values` | all six supported values |
| `text-decoration`, `text-decoration-line` | `text-decoration`, `text-decoration-lines` | none, each line, combinations, and both supported styles |
| `text-decoration-color` | `text-decoration`, `text-decoration-lines` | color, `currentColor`, and active decoration lines |
| `cursor` | `cursor` | every mapped cursor keyword, aliases, and `none` |

## Selector and cascade coverage

| Feature | Chromium fixture | Unit coverage |
| --- | --- | --- |
| Type, class, ID, universal, and compound selectors | `selector-matrix` | selector match matrix |
| Descendant and direct-child combinators | `selector-matrix`, `child-selector` | descendant and child matching |
| Comma-separated selector groups | `selector-matrix` | group matching and specificity |
| Specificity and source order | `selector-matrix`, `child-selector` | cascade precedence |

## Running the complete matrix

Run unit coverage without Chromium:

```sh
cargo test --offline -p ctml --lib css_style::matrix_tests
cargo test --offline -p ctml --lib css_style::tests
cargo test --offline -p ctml --lib css_color::tests
```

Run every Chromium comparison serially:

```sh
cargo test --offline -p ctml page::tests::parity -- --test-threads=1
```

The Chromium suite compares native and browser screenshots with a 0.05% differing-pixel limit. This comparison helps identify rendering differences while the application stays fully native. Artifacts are written to `target/css-parity/<case>/`.
