use super::{parse_math_expression, parse_math_number};

fn parse(value: &str) -> Option<f32> {
    parse_math_expression(value, |dimension| {
        let dimension = dimension.trim().to_ascii_lowercase();
        dimension
            .strip_suffix("px")
            .and_then(|number| number.trim().parse::<f32>().ok())
            .or_else(|| {
                dimension
                    .strip_suffix('%')
                    .and_then(|number| number.trim().parse::<f32>().ok())
                    .map(|percentage| percentage / 2.0)
            })
    })
}

#[test]
fn evaluates_calc_with_dimension_arithmetic_and_precedence() {
    assert_eq!(parse("calc(2 * 5px + 25%)"), Some(22.5));
    assert_eq!(parse("calc((10px - 2px) / 2)"), Some(4.0));
    assert_eq!(parse("calc(1e1px + 2.5px)"), Some(12.5));
}

#[test]
fn evaluates_min_max_clamp_and_nested_functions() {
    assert_eq!(parse("min(10px, 25%)"), Some(10.0));
    assert_eq!(parse("max(10px, 25%)"), Some(12.5));
    assert_eq!(parse("clamp(5px, 25%, 20px)"), Some(12.5));
    assert_eq!(parse("min(30px, calc(10px + 5px))"), Some(15.0));
    assert_eq!(parse("clamp(20px, 10px, 5px)"), Some(20.0));
}

#[test]
fn rejects_mixed_types_invalid_grammar_and_non_finite_results() {
    for value in [
        "calc(10px + 2)",
        "calc(10px +2px)",
        "calc(10px / 0)",
        "calc(10px * 2px)",
        "min(10px, 2)",
        "clamp(1px, 2px)",
        "calc(10px + 2px) trailing",
        "calc(1e999px)",
    ] {
        assert_eq!(parse(value), None, "{value}");
    }
}

#[test]
fn evaluates_number_expressions_separately_from_lengths() {
    assert_eq!(parse_math_number("calc(400 + 200)"), Some(600.0));
    assert_eq!(parse_math_number("max(0.5, 1 / 4)"), Some(0.5));
    assert_eq!(parse_math_number("calc(10px + 2px)"), None);
}
