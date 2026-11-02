# CTML

**CTML (Compiled Text Markup Language)** is a Rust UI library with an HTML-inspired markup language for describing interfaces. A CTML application is a Rust project that uses the library; CTML is not a standalone executable.

The project builds 100% native interfaces without a browser runtime or a separate CTML interpreter. It combines native application performance with familiar markup and CSS-inspired styling. Web interfaces are out of scope.

## Goals

- Describe interfaces with familiar markup.
- Build compiled native applications without a dedicated CTML runtime.
- Provide styling concepts inspired by CSS.
- Keep application rendering and interaction native to the host platform.

## Project status

CTML is in early development. This repository contains a Rust workspace with separate API, renderer, CSS, and markup compiler crates. The syntax, APIs, and build and execution workflows may change.

## Repository structure

```text
crates/
├── ctml/           # Public API facade
├── ctml-css/       # CSS parser, selectors, colors, and lengths
├── ctml-compiler/  # CTML lexer and parser
└── ctml-renderer/  # Native window, text styling, layout, and rendering
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

See [docs/architecture.md](docs/architecture.md) for crate boundaries and dependencies.

## Development

The project uses a Cargo workspace. Build the crates with:

```sh
cargo build --workspace
```

## Native desktop window

The desktop renderer exposes a native window through the Rust library. `Window::create` runs the application event loop until the window closes. The renderer wraps and reflows body text, supports word and free text selection, and maps CSS `cursor` values to native cursor icons. Buttons support pointer and keyboard activation through named actions; single-line text inputs support keyboard editing and input callbacks. Link one or more regular `.css` files in `<headers>` to style the page; rules follow header order. CTML compiles the markup and stylesheets into Rust data at build time; it does not ship a browser or parse them at runtime.

```rust
use ctml::{Window, WindowConfig};

fn main() -> Result<(), ctml::WindowError> {
    Window::create(WindowConfig {
        title: "My CTML application".to_owned(),
        ..WindowConfig::default()
    })
}
```

The integrated demo presents a visual overview of CTML with a responsive layout, local artwork, and a compact set of interface principles. Its build script parses `examples/native-app/src/main.ctml` and the linked CSS files into Rust data; the `.ctml` and `.css` parsers are not part of the running application. The current CSS subset covers common selectors, typography, block styling, flex, and text-grid layout. See [docs/css.md](docs/css.md) for supported syntax and [docs/css-roadmap.md](docs/css-roadmap.md) for the broader CSS feature checklist.

Run the integrated demo with:

```sh
cargo run -p ctml-native-demo
```

The second example is a scrollable feature tour of the supported CTML surface. It includes typography, modern color spaces, block/flex/grid layout, backgrounds and box paint, generated content, focus states, native buttons and editable text inputs, scrolling, and motion support. The interactive controls demonstrate `WindowConfig::on_action` and `WindowConfig::on_input`. It also calls out features that remain on the roadmap.

Run the feature showcase with:

```sh
cargo run -p ctml-feature-showcase
```

Open `examples/feature-showcase/src/main.html` in a browser to compare the same markup and styles against native CTML rendering.

The showcase package shares the integrated demo's build-time CTML and stylesheet compiler, so the compiled application receives markup, CSS, and keyframes without adding a runtime parser.

## Vision

CTML brings the simplicity of describing interfaces with markup together with the performance and integration of compiled native applications. Browser-based rendering is not part of the project runtime.

## License

CTML is licensed under the MIT License. Copyright (c) 2026 Murillo Deolino. See [LICENSE](LICENSE) for the full license text. The vendored `cosmic-text` crate retains its upstream MIT OR Apache-2.0 notices in its own directory.
