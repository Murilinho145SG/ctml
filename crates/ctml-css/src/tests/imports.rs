#[test]
fn expands_relative_imports_in_place_and_tracks_build_dependencies() {
    let root = temporary_directory("ordered-imports");
    std::fs::create_dir_all(&root).expect("temporary stylesheet root should be created");
    std::fs::write(root.join("tokens.css"), ".item { font-weight: 700; }")
        .expect("tokens stylesheet should be written");
    std::fs::write(
        root.join("base.css"),
        "@import \"tokens.css\";\n.item { color: red; }",
    )
    .expect("base stylesheet should be written");
    std::fs::write(
        root.join("main.css"),
        "@IMPORT url(\"base.css\");\n.item { color: blue; }",
    )
    .expect("main stylesheet should be written");

    let (stylesheet, dependencies) =
        super::parse_stylesheet_file(root.join("main.css"), &root).unwrap();

    assert_eq!(stylesheet.rules.len(), 3);
    assert_eq!(stylesheet.rules[0].declarations[0].value, "700");
    assert_eq!(stylesheet.rules[1].declarations[0].value, "red");
    assert_eq!(stylesheet.rules[2].declarations[0].value, "blue");
    assert_eq!(
        dependencies
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        ["main.css", "base.css", "tokens.css"]
    );
    std::fs::remove_dir_all(root).expect("temporary stylesheet root should be removed");
}

#[test]
fn merges_layer_order_across_imported_stylesheets() {
    let root = temporary_directory("imported-layers");
    std::fs::create_dir_all(&root).expect("temporary stylesheet root should be created");
    std::fs::write(root.join("theme.css"), "@layer theme { p { color: red; } }")
        .expect("layered stylesheet should be written");
    std::fs::write(
        root.join("main.css"),
        "@layer base; @import \"theme.css\"; @layer theme { p { color: blue; } }",
    )
    .expect("entry stylesheet should be written");

    let (stylesheet, _) = super::parse_stylesheet_file(root.join("main.css"), &root).unwrap();

    assert_eq!(stylesheet.layers, ["base", "theme"]);
    assert_eq!(stylesheet.rules.len(), 2);
    assert_eq!(stylesheet.rules[0].layer, Some(1));
    assert_eq!(stylesheet.rules[1].layer, Some(1));
}

#[test]
fn rejects_import_cycles_and_paths_outside_the_root() {
    let root = temporary_directory("invalid-imports");
    std::fs::create_dir_all(&root).expect("temporary stylesheet root should be created");
    std::fs::write(root.join("first.css"), "@import \"second.css\";")
        .expect("first stylesheet should be written");
    std::fs::write(root.join("second.css"), "@import \"first.css\";")
        .expect("second stylesheet should be written");

    let cycle = super::parse_stylesheet_file(root.join("first.css"), &root)
        .expect_err("cyclic imports should be rejected");
    assert!(cycle.contains("stylesheet import cycle"));

    std::fs::write(root.join("escape.css"), "@import \"../outside.css\";")
        .expect("escape stylesheet should be written");
    let escape = super::parse_stylesheet_file(root.join("escape.css"), &root)
        .expect_err("imports outside the root should be rejected");
    assert!(escape.contains("stay inside the stylesheet root"));

    std::fs::write(
        root.join("external.css"),
        "@import url(\"https://example.com/site.css\");",
    )
    .expect("external stylesheet should be written");
    let external = super::parse_stylesheet_file(root.join("external.css"), &root)
        .expect_err("network imports should be rejected");
    assert!(external.contains("stay inside the stylesheet root"));

    std::fs::write(
        root.join("late.css"),
        ".item { color: red; } @import \"first.css\";",
    )
    .expect("late-import stylesheet should be written");
    let late = super::parse_stylesheet_file(root.join("late.css"), &root)
        .expect_err("imports after qualified rules should be rejected");
    assert!(late.contains("must precede qualified rules"));
    std::fs::remove_dir_all(root).expect("temporary stylesheet root should be removed");
}

#[test]
fn stylesheet_files_require_utf8_encoding() {
    let root = temporary_directory("encoding");
    std::fs::create_dir_all(&root).expect("temporary stylesheet root should be created");
    std::fs::write(root.join("invalid.css"), [0xff, 0xfe])
        .expect("invalid UTF-8 stylesheet should be written");

    let error = super::parse_stylesheet_file(root.join("invalid.css"), &root)
        .expect_err("non-UTF-8 stylesheet files should be rejected");
    assert!(error.contains("valid UTF-8"));
    std::fs::remove_dir_all(root).expect("temporary stylesheet root should be removed");
}

#[test]
fn reports_stylesheet_file_and_line_for_syntax_errors() {
    let root = temporary_directory("source-locations");
    std::fs::create_dir_all(&root).expect("temporary stylesheet root should be created");
    std::fs::write(root.join("broken.css"), "p { color: red; }\n@unsupported;")
        .expect("malformed stylesheet should be written");

    let error = super::parse_stylesheet_file(root.join("broken.css"), &root)
        .expect_err("unsupported malformed rules should be reported");
    assert!(error.contains("broken.css: line 2, column"), "{error}");

    std::fs::write(
        root.join("late-import.css"),
        ".item {}\n@import \"missing.css\";",
    )
    .expect("late import stylesheet should be written");
    let error = super::parse_stylesheet_file(root.join("late-import.css"), &root)
        .expect_err("late imports should include a diagnostic location");
    assert!(error.contains("late-import.css: line 2, column"), "{error}");

    std::fs::create_dir_all(root.join("nested"))
        .expect("nested stylesheet directory should be created");
    std::fs::write(root.join("entry.css"), "@import \"nested/broken.css\";")
        .expect("entry stylesheet should be written");
    std::fs::write(root.join("nested/broken.css"), ".item {}\n@unsupported;")
        .expect("nested malformed stylesheet should be written");
    let error = super::parse_stylesheet_file(root.join("entry.css"), &root)
        .expect_err("nested syntax errors should identify their source file");
    assert!(
        error.contains("nested/broken.css: line 2, column"),
        "{error}"
    );
    std::fs::remove_dir_all(root).expect("temporary stylesheet root should be removed");
}

fn temporary_directory(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("ctml-css-{name}-{}-{nonce}", std::process::id()))
}
