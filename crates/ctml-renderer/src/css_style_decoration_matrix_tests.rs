use super::*;

#[test]
fn covers_every_text_transform_value() {
    for (value, expected) in [
        ("none", TextTransform::None),
        ("uppercase", TextTransform::Uppercase),
        ("lowercase", TextTransform::Lowercase),
        ("capitalize", TextTransform::Capitalize),
    ] {
        assert_eq!(style_for("text-transform", value).text_transform, expected);
    }
}

#[test]
fn covers_every_text_alignment_value() {
    for (value, expected) in [
        ("left", Align::Left),
        ("start", Align::Start),
        ("right", Align::Right),
        ("end", Align::End),
        ("center", Align::Center),
        ("justify", Align::Justified),
    ] {
        assert_eq!(style_for("text-align", value).text_align, expected);
    }
}

#[test]
fn covers_every_text_decoration_line_and_style() {
    let none = style_for("text-decoration", "none");
    assert!(!none.underline && !none.overline && !none.strikethrough);
    let all = style_for("text-decoration", "underline overline line-through");
    assert!(all.underline && all.overline && all.strikethrough);
    for (value, expected) in [
        ("solid", TextDecorationStyle::Solid),
        ("double", TextDecorationStyle::Double),
        ("dotted", TextDecorationStyle::Dotted),
        ("dashed", TextDecorationStyle::Dashed),
        ("wavy", TextDecorationStyle::Wavy),
    ] {
        assert_eq!(
            style_for("text-decoration-style", value).decoration_style,
            expected,
            "{value}"
        );
    }
}

#[test]
fn text_decoration_shorthand_parses_and_resets_longhands_in_cascade_order() {
    let combined = style_for_declarations(&[
        ("text-decoration-style", "dashed"),
        ("text-decoration-color", "blue"),
        ("text-decoration-thickness", "4px"),
        ("text-decoration", "underline wavy 2px rgb(255, 0, 0)"),
    ]);
    assert!(combined.underline);
    assert!(!combined.overline && !combined.strikethrough);
    assert_eq!(combined.decoration_style, TextDecorationStyle::Wavy);
    assert_eq!(combined.decoration_thickness, Some(2.0));
    assert_eq!(combined.text_decoration_color, Some([255, 0, 0, 255]));

    let reset = style_for_declarations(&[
        ("text-decoration-style", "dashed"),
        ("text-decoration-color", "blue"),
        ("text-decoration-thickness", "4px"),
        ("text-decoration", "overline"),
    ]);
    assert!(reset.overline);
    assert_eq!(reset.decoration_style, TextDecorationStyle::Solid);
    assert_eq!(reset.decoration_thickness, None);
    assert_eq!(reset.text_decoration_color, None);

    let longhand_wins = style_for_declarations(&[
        ("text-decoration", "underline wavy"),
        ("text-decoration-style", "dashed"),
    ]);
    let later_shorthand_wins = style_for_declarations(&[
        ("text-decoration-style", "dashed"),
        ("text-decoration", "underline wavy"),
    ]);
    assert_eq!(longhand_wins.decoration_style, TextDecorationStyle::Dashed);
    assert_eq!(
        later_shorthand_wins.decoration_style,
        TextDecorationStyle::Wavy
    );

    let invalid = style_for_declarations(&[("text-decoration", "underline invalid")]);
    assert!(!invalid.underline);
}

#[test]
fn font_shorthand_expands_and_resets_supported_font_longhands() {
    let style = style_for_declarations(&[(
        "font",
        "italic small-caps 700 condensed 20px/1.5 \"Open Sans\", sans-serif",
    )]);
    assert_eq!(style.font_style, Style::Italic);
    assert_eq!(style.font_variant_caps, FontVariantCaps::SmallCaps);
    assert_eq!(style.weight, Weight(700));
    assert_eq!(style.stretch, Stretch::Condensed);
    assert_eq!(style.font_size, 20.0);
    assert_eq!(style.line_height, 30.0);
    assert!(!style.line_height_normal);
    assert_eq!(style.family, "\"Open Sans\", sans-serif");

    let reset = style_for_declarations(&[
        ("font-kerning", "none"),
        ("font-variant-ligatures", "none"),
        ("font-weight", "700"),
        ("font", "16px serif"),
    ]);
    assert_eq!(reset.kerning, None);
    assert_eq!(reset.common_ligatures, None);
    assert_eq!(reset.weight, Weight::NORMAL);
    assert!(reset.line_height_normal);
    assert_eq!(reset.family, "serif");

    let longhand_after =
        style_for_declarations(&[("font", "italic 24px sans-serif"), ("font-weight", "700")]);
    let shorthand_after =
        style_for_declarations(&[("font-weight", "700"), ("font", "italic 24px sans-serif")]);
    assert_eq!(longhand_after.weight, Weight(700));
    assert_eq!(shorthand_after.weight, Weight::NORMAL);
    assert_eq!(shorthand_after.font_size, 24.0);

    let invalid = style_for_declarations(&[("font", "italic 20px")]);
    assert_eq!(invalid.font_size, 16.0);
    assert_eq!(invalid.font_style, Style::Normal);
}

#[test]
fn covers_every_text_decoration_skip_ink_value() {
    for (value, expected) in [
        ("auto", TextDecorationSkipInk::Auto),
        ("none", TextDecorationSkipInk::None),
        ("all", TextDecorationSkipInk::All),
    ] {
        assert_eq!(
            style_for("text-decoration-skip-ink", value).decoration_skip_ink,
            expected,
            "{value}"
        );
    }
}

#[test]
fn covers_nonnegative_text_decoration_thickness_lengths() {
    for (value, expected) in [
        ("0", 0.0),
        ("0px", 0.0),
        ("2px", 2.0),
        ("0.5em", 8.0),
        ("25%", 4.0),
    ] {
        assert_eq!(
            style_for("text-decoration-thickness", value).decoration_thickness,
            Some(expected),
            "{value}"
        );
    }
    for value in ["auto", "from-font"] {
        assert_eq!(
            style_for("text-decoration-thickness", value).decoration_thickness,
            None,
            "{value}"
        );
    }
    assert_eq!(
        style_for("text-decoration-thickness", "-1px").decoration_thickness,
        None
    );
}

#[test]
fn covers_signed_text_underline_offset_lengths() {
    for (value, expected) in [
        ("0", 0.0),
        ("2px", 2.0),
        ("-2px", -2.0),
        ("1em", 16.0),
        ("50%", 8.0),
        ("-25%", -4.0),
    ] {
        assert_eq!(
            style_for("text-underline-offset", value).underline_offset,
            Some(expected),
            "{value}"
        );
    }
    assert_eq!(
        style_for("text-underline-offset", "auto").underline_offset,
        None
    );
    assert_eq!(
        style_for("text-underline-offset", "invalid").underline_offset,
        None
    );
}

#[test]
fn covers_every_font_feature_switch() {
    let none = style_for("font-variant-ligatures", "none");
    assert_eq!(none.common_ligatures, Some(false));
    assert_eq!(none.discretionary_ligatures, Some(false));
    assert_eq!(none.historical_ligatures, Some(false));
    assert_eq!(none.contextual_alternates, Some(false));
    let enabled = style_for(
        "font-variant-ligatures",
        "common-ligatures discretionary-ligatures historical-ligatures contextual",
    );
    assert_eq!(enabled.common_ligatures, Some(true));
    assert_eq!(enabled.discretionary_ligatures, Some(true));
    assert_eq!(enabled.historical_ligatures, Some(true));
    assert_eq!(enabled.contextual_alternates, Some(true));
    let normal = style_for("font-variant-ligatures", "normal");
    assert_eq!(normal.common_ligatures, None);
    assert_eq!(normal.discretionary_ligatures, None);
    assert_eq!(normal.historical_ligatures, None);
    assert_eq!(normal.contextual_alternates, None);
}

#[test]
fn covers_every_font_caps_and_position_value() {
    for (value, expected) in [
        ("normal", FontVariantCaps::Normal),
        ("small-caps", FontVariantCaps::SmallCaps),
        ("all-small-caps", FontVariantCaps::AllSmallCaps),
        ("petite-caps", FontVariantCaps::PetiteCaps),
        ("all-petite-caps", FontVariantCaps::AllPetiteCaps),
        ("unicase", FontVariantCaps::Unicase),
        ("titling-caps", FontVariantCaps::TitlingCaps),
    ] {
        assert_eq!(
            style_for("font-variant-caps", value).font_variant_caps,
            expected
        );
    }
    for (value, expected) in [
        ("normal", FontVariantPosition::Normal),
        ("sub", FontVariantPosition::Sub),
        ("super", FontVariantPosition::Super),
    ] {
        assert_eq!(
            style_for("font-variant-position", value).font_variant_position,
            expected
        );
    }
}

#[test]
fn covers_every_mapped_cursor_keyword() {
    let cases = [
        ("auto", CursorIcon::Text),
        ("text", CursorIcon::Text),
        ("default", CursorIcon::Default),
        ("pointer", CursorIcon::Pointer),
        ("crosshair", CursorIcon::Crosshair),
        ("move", CursorIcon::Move),
        ("wait", CursorIcon::Wait),
        ("progress", CursorIcon::Progress),
        ("help", CursorIcon::Help),
        ("not-allowed", CursorIcon::NotAllowed),
        ("no-drop", CursorIcon::NoDrop),
        ("grab", CursorIcon::Grab),
        ("grabbing", CursorIcon::Grabbing),
        ("copy", CursorIcon::Copy),
        ("alias", CursorIcon::Alias),
        ("context-menu", CursorIcon::ContextMenu),
        ("cell", CursorIcon::Cell),
        ("all-scroll", CursorIcon::AllScroll),
        ("all-resize", CursorIcon::AllResize),
        ("dnd-ask", CursorIcon::DndAsk),
        ("vertical-text", CursorIcon::VerticalText),
        ("ew-resize", CursorIcon::EwResize),
        ("col-resize", CursorIcon::EwResize),
        ("ns-resize", CursorIcon::NsResize),
        ("row-resize", CursorIcon::NsResize),
        ("nesw-resize", CursorIcon::NeswResize),
        ("nwse-resize", CursorIcon::NwseResize),
        ("n-resize", CursorIcon::NResize),
        ("e-resize", CursorIcon::EResize),
        ("s-resize", CursorIcon::SResize),
        ("w-resize", CursorIcon::WResize),
        ("ne-resize", CursorIcon::NeResize),
        ("nw-resize", CursorIcon::NwResize),
        ("se-resize", CursorIcon::SeResize),
        ("sw-resize", CursorIcon::SwResize),
        ("zoom-in", CursorIcon::ZoomIn),
        ("zoom-out", CursorIcon::ZoomOut),
    ];
    for (value, expected) in cases {
        assert_eq!(style_for("cursor", value).cursor, expected, "{value}");
        assert!(!style_for("cursor", value).cursor_hidden, "{value}");
    }
    let hidden = style_for("cursor", "none");
    assert_eq!(hidden.cursor, CursorIcon::Default);
    assert!(hidden.cursor_hidden);
}
