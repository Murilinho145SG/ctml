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
| `text-decoration`, `text-decoration-line`, `text-decoration-style`, `text-decoration-skip-ink`, `text-decoration-thickness` | `text-decoration`, `text-decoration-lines`, `text-decoration-style`, `text-decoration-skip-ink`, `text-decoration-thickness` | none, each line, combinations, all five styles, all skip-ink values, and thickness lengths |
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

The suite reports every exact RGB pixel mismatch and highlights mismatches above the per-channel tolerance. Exact parity cases fail on any RGB difference; other cases retain a 0.05% limit for pixels above that tolerance. Artifacts are written to `target/css-parity/<case>/`.
