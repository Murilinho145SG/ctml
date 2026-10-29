# CTML

**CTML (Compiled Text Markup Language)** is a Rust UI library with an HTML-inspired markup language for describing interfaces. A CTML application is a Rust project that uses the library; CTML is not a standalone executable.

The project builds 100% native interfaces without a browser runtime or a separate CTML interpreter. It combines native application performance with familiar markup and CSS-inspired styling. Web interfaces are out of scope.

## Goals

- Describe interfaces with familiar markup.
- Build compiled native applications without a dedicated CTML runtime.
- Provide styling concepts inspired by CSS.
- Keep application rendering and interaction native to the host platform.

## Project status

CTML is in early development. This repository contains a Rust workspace with the `ctml` and `ctml-compiler` crates; the compiler is still a prototype. The syntax, APIs, and build and execution workflows may change.

## Repository structure

```text
crates/
├── ctml/           # Main crate
└── ctml-compiler/  # Compiler prototype
```

## Markup example

A CTML interface can be described with elements and attributes in a format familiar to HTML users:

```ctml
<headers>
    <application name="my-application" desktop />
</headers>

<body>
    <div class="message">Hello, world!</div>
</body>
```

This example illustrates the language's direction. It is not yet a stable API or a complete running interface.

The initial syntax is documented in [docs/syntax.md](docs/syntax.md).

## Development

The project uses a Cargo workspace. Build the crates with:

```sh
cargo build --workspace
```

## Native desktop window

The first desktop feature exposes a native window through the Rust library. `Window::create` runs the application event loop until the window closes. The renderer presents a white page, wraps body text using a system sans-serif font at 16 px, reflows the text when the window is resized, supports word selection by double-click, and supports free text selection by dragging with the left mouse button. CSS `cursor` values map to native cursor icons. A small set of common block elements starts a new line. Link a regular `.css` file in `<headers>` to style the page. CTML compiles the markup and stylesheet into Rust data at build time; it does not ship a browser or parse them at runtime.

```rust
use ctml::{Window, WindowConfig};

fn main() -> Result<(), ctml::WindowError> {
    Window::create(WindowConfig {
        title: "My CTML application".to_owned(),
        ..WindowConfig::default()
    })
}
```

The integrated demo parses `examples/native-app/src/main.ctml` and its linked `main.css` stylesheet in its build script. The `.ctml` and `.css` parsers are not part of the running application. The current CSS subset covers common selectors and text styling. See [docs/css.md](docs/css.md) for supported syntax and [docs/css-roadmap.md](docs/css-roadmap.md) for the broader CSS feature checklist.

Run the integrated demo with:

```sh
cargo run -p ctml-native-demo
```

## Vision

CTML brings the simplicity of describing interfaces with markup together with the performance and integration of compiled native applications. Browser-based rendering is not part of the project runtime.

## License

CTML is licensed under the GNU General Public License v2.0 only. Copyright (C) 2026 Murillo Deolino. See [LICENSE](LICENSE) for the full license text.
