# CTML

**CTML (Compiled Text Markup Language)** is a markup language for building user interfaces. It is inspired by HTML and written in Rust.

The project aims to make it possible to build native interfaces without depending on a dedicated runtime. CTML combines the speed and integration of native applications with a familiar way to describe interfaces using markup and CSS-inspired styling. The language is also intended to support web interfaces.

## Goals

- Describe interfaces with familiar markup.
- Build compiled native applications without a dedicated CTML runtime.
- Provide styling concepts inspired by CSS.
- Support creating web interfaces with the same language.

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

## Development

The project uses a Cargo workspace. Build the crates with:

```sh
cargo build --workspace
```

## Vision

CTML aims to bring the simplicity of describing interfaces with markup together with the performance and integration of compiled applications. The goal is to use the same approach for both native and web interfaces.

## License

CTML is licensed under the GNU General Public License v2.0 only. Copyright (C) 2026 Murillo Deolino. See [LICENSE](LICENSE) for the full license text.
