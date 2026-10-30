use crate::parse_color;

#[test]
fn color_mix_carries_forward_lab_components_in_lab_spaces() {
    for (space, missing_color, zero_color, other_color) in [
        ("lab", "lab(none 0 0)", "lab(0% 0 0)", "lab(60% 0 0)"),
        ("lab", "lab(60% none 0)", "lab(60% 0 0)", "lab(60% 30 0)"),
        ("lch", "lch(none 0 0)", "lch(0% 0 0)", "lch(60% 0 0)"),
        (
            "lch",
            "lch(60% none 30deg)",
            "lch(60% 0 30deg)",
            "lch(60% 10 30deg)",
        ),
        (
            "lch",
            "lch(60% 10 none)",
            "lch(60% 10 0deg)",
            "lch(60% 10 90deg)",
        ),
        ("oklab", "oklab(none 0 0)", "oklab(0 0 0)", "oklab(.6 0 0)"),
        (
            "oklab",
            "oklab(.6 none 0)",
            "oklab(.6 0 0)",
            "oklab(.6 .1 0)",
        ),
        ("oklch", "oklch(none 0 0)", "oklch(0 0 0)", "oklch(.6 0 0)"),
        (
            "oklch",
            "oklch(.6 none 30deg)",
            "oklch(.6 0 30deg)",
            "oklch(.6 .1 30deg)",
        ),
        (
            "oklch",
            "oklch(.6 .1 none)",
            "oklch(.6 .1 0deg)",
            "oklch(.6 .1 90deg)",
        ),
    ] {
        let mixed = parse_color(&format!(
            "color-mix(in {space}, {missing_color}, {other_color})"
        ))
        .unwrap_or_else(|| panic!("failed to parse missing-component mix in {space}"));
        let zero_component = parse_color(&format!(
            "color-mix(in {space}, {zero_color}, {other_color})"
        ))
        .unwrap_or_else(|| panic!("failed to parse zero-component mix in {space}"));
        assert_ne!(
            mixed, zero_component,
            "missing component was treated as zero in {space}"
        );
    }
}

#[test]
fn color_mix_ignores_hues_of_powerless_colors() {
    for (space, first, second) in [
        ("hsl", "hsl(0 0% 100%)", "hsl(180 0% 100%)"),
        ("hwb", "hwb(0deg 100% 0%)", "hwb(180deg 100% 0%)"),
        ("lch", "lch(100% 0 0deg)", "lch(100% 0 180deg)"),
        ("oklch", "oklch(1 0 0deg)", "oklch(1 0 180deg)"),
    ] {
        assert_eq!(
            parse_color(&format!("color-mix(in {space}, {first}, blue)")),
            parse_color(&format!("color-mix(in {space}, {second}, blue)")),
            "powerless hue affected interpolation in {space}"
        );
    }
}

#[test]
fn color_mix_carries_lab_components_through_nested_mixes() {
    let missing = parse_color(
        "color-mix(in lab, color-mix(in lab, lab(none 0 0), lab(none 30 0)), lab(60% 0 0))",
    )
    .expect("nested Lab missing-component mix should parse");
    let zero = parse_color(
        "color-mix(in lab, color-mix(in lab, lab(0% 0 0), lab(0% 30 0)), lab(60% 0 0))",
    )
    .expect("nested Lab zero-component mix should parse");
    assert_ne!(missing, zero);
}
