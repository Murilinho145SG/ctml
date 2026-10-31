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
fn merges_keyframe_definitions_from_imported_stylesheets_in_order() {
    let root = temporary_directory("imported-keyframes");
    std::fs::create_dir_all(&root).expect("temporary stylesheet root should be created");
    std::fs::write(
        root.join("motion.css"),
        "@keyframes imported { from { opacity: 0; } to { opacity: 1; } }",
    )
    .expect("motion stylesheet should be written");
    std::fs::write(
        root.join("main.css"),
        "@import \"motion.css\"; @keyframes after { to { opacity: 1; } }",
    )
    .expect("main stylesheet should be written");

    let (stylesheet, _) = super::parse_stylesheet_file(root.join("main.css"), &root).unwrap();

    assert_eq!(
        stylesheet
            .keyframes
            .iter()
            .map(|rule| rule.name.as_str())
            .collect::<Vec<_>>(),
        ["imported", "after"]
    );
    std::fs::remove_dir_all(root).expect("temporary stylesheet root should be removed");
}

#[test]
fn loads_relative_local_resources_and_decodes_url_paths() {
    let root = temporary_directory("local-resources");
    std::fs::create_dir_all(root.join("styles")).expect("stylesheet directory should exist");
    std::fs::create_dir_all(root.join("assets")).expect("resource directory should exist");
    std::fs::write(root.join("styles/main.css"), "p { color: red; }")
        .expect("stylesheet should be written");
    std::fs::write(root.join("assets/font file.bin"), [0, 1, 2, 255])
        .expect("resource should be written");

    let resource = crate::load_css_resource(
        "url('../assets/font%20file.bin')",
        root.join("styles/main.css"),
        &root,
    )
    .expect("relative local resource should load");

    assert_eq!(
        resource.path,
        root.join("assets/font file.bin").canonicalize().unwrap()
    );
    assert_eq!(resource.bytes, [0, 1, 2, 255]);
    std::fs::remove_dir_all(root).expect("temporary resource root should be removed");
}

#[test]
fn loads_and_validates_local_color_profiles_from_the_declaring_stylesheet() {
    let root = temporary_directory("color-profile-resource");
    std::fs::create_dir_all(root.join("styles")).unwrap();
    std::fs::create_dir_all(root.join("profiles")).unwrap();
    let profile_bytes = lcms2::Profile::new_srgb().icc().unwrap();
    std::fs::write(root.join("profiles/srgb.icc"), &profile_bytes).unwrap();
    std::fs::write(
        root.join("styles/profiles.css"),
        "@color-profile --screen { src: url('https://invalid.example/unused.icc'); } @color-profile --screen { src: url('../profiles/srgb.icc'); components: red, green, blue; }",
    )
    .unwrap();
    std::fs::write(
        root.join("styles/main.css"),
        "@import 'profiles.css'; p { color: color(--screen 1 0 0); }",
    )
    .unwrap();

    let (stylesheet, dependencies) =
        super::parse_stylesheet_file(root.join("styles/main.css"), &root).unwrap();

    assert_eq!(stylesheet.color_profiles.len(), 1);
    let profile = &stylesheet.color_profiles[0];
    assert_eq!(profile.name, "--screen");
    assert_eq!(
        profile.profile_data.as_deref(),
        Some(profile_bytes.as_slice())
    );
    let converted = profile
        .to_srgb(&[1.0, 0.0, 0.0])
        .expect("loaded ICC profile should convert to sRGB");
    assert!((converted[0] - 1.0).abs() < 0.001);
    assert!(converted[1].abs() < 0.001);
    assert!(converted[2].abs() < 0.001);
    let round_trip = profile
        .from_srgb([1.0, 0.0, 0.0])
        .expect("sRGB should convert back into the loaded ICC profile");
    assert_eq!(round_trip.len(), 3);
    assert!((round_trip[0] - 1.0).abs() < 0.001);
    assert!(round_trip[1].abs() < 0.001);
    assert!(round_trip[2].abs() < 0.001);
    assert_eq!(
        crate::parse_color_with_profiles("color(--screen 1 0 0 / 50%)", &stylesheet.color_profiles),
        Some([255, 0, 0, 128])
    );
    assert_eq!(
        crate::parse_color_with_profiles(
            "color-mix(in --screen, color(--screen 1 0 0), blue)",
            &stylesheet.color_profiles,
        ),
        Some([127, 0, 128, 255])
    );
    assert_eq!(
        crate::parse_color_with_profiles(
            "color(from red --screen red green blue / alpha)",
            &stylesheet.color_profiles,
        ),
        Some([255, 0, 0, 255])
    );
    assert_eq!(
        crate::parse_color_with_profiles("color(--missing 1 0 0)", &stylesheet.color_profiles),
        None
    );
    assert_eq!(
        crate::parse_color_with_profiles("color(--Screen 1 0 0)", &stylesheet.color_profiles),
        None
    );
    assert_eq!(
        profile.stylesheet_path.as_deref(),
        Some(
            root.join("styles/profiles.css")
                .canonicalize()
                .unwrap()
                .as_path()
        )
    );
    assert!(dependencies.iter().any(|path| path.ends_with("srgb.icc")));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn resolves_font_face_sources_from_the_declaring_stylesheet() {
    let root = temporary_directory("font-face-import");
    std::fs::create_dir_all(root.join("styles")).unwrap();
    std::fs::create_dir_all(root.join("fonts")).unwrap();
    std::fs::write(root.join("fonts/custom.bin"), [1, 2, 3]).unwrap();
    std::fs::write(
        root.join("styles/fonts.css"),
        "@font-face { font-family: 'Custom'; src: url('../fonts/custom.bin'); }",
    )
    .unwrap();
    std::fs::write(
        root.join("styles/main.css"),
        "@import 'fonts.css'; p { font-family: Custom; }",
    )
    .unwrap();

    let (stylesheet, _) =
        super::parse_stylesheet_file(root.join("styles/main.css"), &root).unwrap();
    let face = &stylesheet.font_faces[0];
    assert_eq!(face.family, "Custom");
    let resource =
        crate::load_css_resource(&face.source, face.stylesheet_path.as_ref().unwrap(), &root)
            .unwrap();
    assert_eq!(resource.bytes, [1, 2, 3]);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn parses_ordered_font_face_source_lists_and_hints() {
    let root = temporary_directory("font-face-format-hint");
    std::fs::create_dir_all(root.join("styles")).unwrap();
    std::fs::write(root.join("styles/font.bin"), [4, 5, 6]).unwrap();
    std::fs::write(root.join("styles/main.css"), "").unwrap();

    let candidates = crate::parse_font_face_sources(
        "local('System Sans'), local(System Sans), url('font.bin') format('opentype') tech(color-COLRv1), url('fallback.bin')",
    )
    .unwrap();
    assert_eq!(
        candidates,
        [
            crate::CssFontSource::Local("System Sans".to_owned()),
            crate::CssFontSource::Local("System Sans".to_owned()),
            crate::CssFontSource::Url("font.bin".to_owned()),
            crate::CssFontSource::Url("fallback.bin".to_owned()),
        ]
    );
    let crate::CssFontSource::Url(url) = &candidates[2] else {
        panic!("second candidate should be a URL")
    };
    let resource = crate::load_local_resource(url, root.join("styles/main.css"), &root).unwrap();
    assert_eq!(resource.bytes, [4, 5, 6]);
    assert!(crate::parse_font_face_sources("url(font.bin) bogus").is_none());

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn parses_css_url_tokens_strings_and_escapes() {
    assert_eq!(
        crate::parse_css_url("url(font.woff2)"),
        Some("font.woff2".to_owned())
    );
    assert_eq!(
        crate::parse_css_url("url('font file.woff2')"),
        Some("font file.woff2".to_owned())
    );
    assert_eq!(
        crate::parse_css_url("url('font)name.woff2')"),
        Some("font)name.woff2".to_owned())
    );
    assert_eq!(
        crate::parse_css_url(r"url(font\20 file.woff2)"),
        Some("font file.woff2".to_owned())
    );
    for invalid in [
        "font.woff2",
        "url(font.woff2) url(other.woff2)",
        "url(font.woff2 more)",
        "url('unterminated)",
        "url('font.woff2' extra)",
    ] {
        assert!(
            crate::parse_css_url(invalid).is_none(),
            "accepted {invalid}"
        );
    }
}

#[test]
fn rejects_external_and_out_of_root_resource_urls() {
    let root = temporary_directory("unsafe-resources");
    std::fs::create_dir_all(root.join("styles")).expect("stylesheet directory should exist");
    std::fs::write(root.join("styles/main.css"), "p { color: red; }")
        .expect("stylesheet should be written");

    for url in [
        "https://example.com/font.woff2",
        "data:font/woff2;base64,AA==",
        "file:///etc/passwd",
        "/etc/passwd",
        "../../outside.bin",
        "%2e%2e/%2e%2e/outside.bin",
        "font.bin?download=1",
        "font.bin#fragment",
    ] {
        assert!(
            crate::load_local_resource(url, root.join("styles/main.css"), &root).is_err(),
            "resource URL should be rejected: {url}"
        );
    }
    std::fs::remove_dir_all(root).expect("temporary resource root should be removed");
}

#[cfg(unix)]
#[test]
fn rejects_resource_symlinks_that_escape_the_root() {
    use std::os::unix::fs::symlink;

    let root = temporary_directory("resource-symlinks");
    let outside = temporary_directory("resource-symlink-target");
    std::fs::create_dir_all(root.join("styles")).expect("stylesheet directory should exist");
    std::fs::create_dir_all(&outside).expect("outside directory should exist");
    std::fs::write(root.join("styles/main.css"), "p { color: red; }")
        .expect("stylesheet should be written");
    std::fs::write(outside.join("secret.bin"), [1, 2, 3]).expect("target should be written");
    symlink(&outside, root.join("styles/external")).expect("resource symlink should be created");

    assert!(
        crate::load_local_resource("external/secret.bin", root.join("styles/main.css"), &root,)
            .is_err()
    );
    std::fs::remove_dir_all(root).expect("temporary resource root should be removed");
    std::fs::remove_dir_all(outside).expect("temporary symlink target should be removed");
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
fn keeps_anonymous_layers_distinct_across_imports() {
    let root = temporary_directory("imported-anonymous-layers");
    std::fs::create_dir_all(&root).expect("temporary stylesheet root should be created");
    std::fs::write(root.join("theme.css"), "@layer { p { color: red; } }")
        .expect("imported anonymous layer should be written");
    std::fs::write(
        root.join("main.css"),
        "@import \"theme.css\"; @layer { p { color: blue; } }",
    )
    .expect("main anonymous layer should be written");

    let (stylesheet, _) = super::parse_stylesheet_file(root.join("main.css"), &root).unwrap();

    assert_eq!(stylesheet.layers.len(), 2);
    assert_ne!(stylesheet.layers[0], stylesheet.layers[1]);
    assert_eq!(stylesheet.rules[0].layer, Some(0));
    assert_eq!(stylesheet.rules[1].layer, Some(1));
    std::fs::remove_dir_all(root).expect("temporary stylesheet root should be removed");
}

#[test]
fn merges_escaped_layer_names_across_imports() {
    let root = temporary_directory("imported-escaped-layers");
    std::fs::create_dir_all(&root).expect("temporary stylesheet root should be created");
    std::fs::write(
        root.join("theme.css"),
        r"@layer foo\.bar { p { color: red; } }",
    )
    .expect("imported escaped layer should be written");
    std::fs::write(
        root.join("main.css"),
        r#"@import "theme.css"; @layer foo\.bar { p { color: blue; } }"#,
    )
    .expect("main escaped layer should be written");

    let (stylesheet, _) = super::parse_stylesheet_file(root.join("main.css"), &root).unwrap();

    assert_eq!(stylesheet.layers, [r"foo\.bar"]);
    assert_eq!(stylesheet.rules.len(), 2);
    assert_eq!(stylesheet.rules[0].layer, Some(0));
    assert_eq!(stylesheet.rules[1].layer, Some(0));
    std::fs::remove_dir_all(root).expect("temporary stylesheet root should be removed");
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
    std::fs::write(
        root.join("broken.css"),
        "p { color: red; }\n@layer first,,second;",
    )
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
    std::fs::write(
        root.join("nested/broken.css"),
        ".item {}\n@layer first,,second;",
    )
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
