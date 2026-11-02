# Contributing to CTML

Thanks for your interest in CTML. The project is in early development, so APIs, syntax, and implementation details may change. Contributions that improve correctness, browser parity for supported CSS, documentation, and the native application experience are welcome.

## Before you start

- For a large feature or a change to public syntax or APIs, open an issue first so we can agree on the scope.
- For a bug report, include the CTML or CSS needed to reproduce it, the expected and actual result, your operating system, and the window size when layout is involved. Screenshots are useful for rendering issues.
- Keep pull requests focused on one change. Include or update regression coverage when behavior changes.

## Development setup

CTML is a Rust Cargo workspace. Use a stable Rust toolchain. On Linux, building and running the native window example may require the system libraries listed in [the CI workflow](.github/workflows/ci.yml).

Build the workspace:

```sh
cargo build --workspace
```

Run the same formatting and workspace test checks used by CI:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
```

Run the integrated desktop example:

```sh
cargo run -p ctml-native-demo
```

Some renderer parity tests use Chromium. See [the CSS testing notes](docs/css.md) for the parity test setup and focused CSS test commands.

## Pull requests

Before opening a pull request:

1. Make sure the change builds and the relevant tests pass.
2. Run formatting checks.
3. Update documentation when syntax, behavior, or supported CSS changes.
4. For visual or layout changes, include before-and-after screenshots when practical and note the viewport dimensions.

In the pull request description, explain the problem, the solution, and the checks you ran. Mention known limitations and link related issues. Keep generated files and unrelated formatting changes out of the diff.

## Code and licensing

Follow the existing Rust style and keep changes small and focused. Use English for code, comments, examples, and repository documentation. Credit external work and preserve its license notices. By submitting a contribution, you agree that it may be distributed under the repository's MIT license; third-party components retain their own notices and licenses.
