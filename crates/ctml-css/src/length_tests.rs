use super::{
    FontRelativeUnits, ViewportSize, parse_font_relative_length,
    parse_font_relative_length_with_viewport, parse_length, parse_length_with_line_height,
    parse_nonnegative_font_relative_length, parse_nonnegative_font_relative_length_with_viewport,
    parse_nonnegative_length_with_line_height, parse_signed_font_relative_length,
    parse_signed_font_relative_length_with_viewport,
    parse_signed_length_percentage_with_line_height,
};

#[test]
fn parses_lh_relative_lengths_and_percentages() {
    assert_eq!(parse_length_with_line_height("2lh", 16.0, 24.0), Some(48.0));
    assert_eq!(
        parse_nonnegative_length_with_line_height("0lh", 16.0, 24.0),
        Some(0.0)
    );
    assert_eq!(
        parse_signed_length_percentage_with_line_height("-0.5lh", 16.0, 24.0),
        Some(-12.0)
    );
    assert_eq!(parse_length_with_line_height("2em", 16.0, 24.0), Some(32.0));
}

#[test]
fn parses_length_components_as_single_css_numeric_tokens() {
    assert_eq!(parse_length("+1.25e2PX", 16.0), Some(125.0));
    assert_eq!(parse_length(".5em", 16.0), Some(8.0));
    assert_eq!(parse_length("10 %", 100.0), None);
    assert_eq!(parse_length("10 px", 16.0), None);
    assert_eq!(parse_length("1e999px", 16.0), None);
    assert_eq!(parse_length("10px trailing", 16.0), None);
}

#[test]
fn evaluates_math_functions_for_font_relative_and_signed_lengths() {
    let units = FontRelativeUnits::default();
    assert_eq!(
        parse_font_relative_length_with_viewport(
            "calc(1em + 4px)",
            16.0,
            20.0,
            units,
            ViewportSize::default(),
        ),
        Some(20.0)
    );
    assert_eq!(
        parse_font_relative_length_with_viewport(
            "calc(0px + 10px)",
            16.0,
            20.0,
            units,
            ViewportSize::default(),
        ),
        Some(10.0)
    );
    assert_eq!(
        parse_font_relative_length_with_viewport(
            "calc(0px)",
            16.0,
            20.0,
            units,
            ViewportSize::default(),
        ),
        Some(0.0)
    );
    assert_eq!(
        parse_nonnegative_font_relative_length("clamp(50%, 1em, 30px)", 16.0, 20.0, units,),
        Some(16.0)
    );
    assert_eq!(
        parse_signed_font_relative_length("min(-1px, calc(2px - 5px))", 16.0, 20.0, units,),
        Some(-3.0)
    );
    assert_eq!(
        parse_nonnegative_font_relative_length("calc(1px - 2px)", 16.0, 20.0, units,),
        None
    );
}

#[test]
fn parses_font_metric_relative_units() {
    let metrics = FontRelativeUnits {
        ex: 0.45,
        ch: 0.52,
        ic: 1.0,
        cap: 0.68,
    };
    assert_eq!(
        parse_font_relative_length("2ex", 20.0, 24.0, metrics),
        Some(18.0)
    );
    assert_eq!(
        parse_font_relative_length("2ch", 20.0, 24.0, metrics),
        Some(20.8)
    );
    assert_eq!(
        parse_font_relative_length("2ic", 20.0, 24.0, metrics),
        Some(40.0)
    );
    assert_eq!(
        parse_font_relative_length("2cap", 20.0, 24.0, metrics),
        Some(27.2)
    );
    assert_eq!(
        parse_font_relative_length("2lh", 20.0, 24.0, metrics),
        Some(48.0)
    );
    assert_eq!(
        parse_nonnegative_font_relative_length("0.5cap", 20.0, 24.0, metrics),
        Some(6.8)
    );
    assert_eq!(
        parse_signed_font_relative_length("-2ch", 20.0, 24.0, metrics),
        Some(-20.8)
    );
    assert_eq!(
        parse_signed_font_relative_length("-0.5lh", 20.0, 24.0, metrics),
        Some(-12.0)
    );
}

#[test]
fn parses_viewport_relative_lengths() {
    let units = FontRelativeUnits::default();
    let viewport = ViewportSize {
        width: 800.0,
        height: 600.0,
    };
    for unit in ["vw", "svw", "lvw", "dvw", "vi", "svi", "lvi", "dvi"] {
        let value = format!("10{unit}");
        assert_eq!(
            parse_font_relative_length_with_viewport(&value, 16.0, 20.0, units, viewport),
            Some(80.0),
            "{unit}"
        );
    }
    for unit in ["vh", "svh", "lvh", "dvh", "vb", "svb", "lvb", "dvb"] {
        let value = format!("10{unit}");
        assert_eq!(
            parse_nonnegative_font_relative_length_with_viewport(
                &value, 16.0, 20.0, units, viewport
            ),
            Some(60.0),
            "{unit}"
        );
    }
    for unit in ["vmin", "svmin", "lvmin", "dvmin"] {
        let value = format!("10{unit}");
        assert_eq!(
            parse_signed_font_relative_length_with_viewport(&value, 16.0, 20.0, units, viewport),
            Some(60.0),
            "{unit}"
        );
    }
    for unit in ["vmax", "svmax", "lvmax", "dvmax"] {
        let value = format!("10{unit}");
        assert_eq!(
            parse_font_relative_length_with_viewport(&value, 16.0, 20.0, units, viewport),
            Some(80.0),
            "{unit}"
        );
    }
    assert_eq!(
        parse_signed_font_relative_length_with_viewport("-5dvw", 16.0, 20.0, units, viewport),
        Some(-40.0)
    );
}
