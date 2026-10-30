use cssparser::{Parser, ParserInput, Token};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CssNumericToken {
    Number(f32),
    Percentage(f32),
    Dimension(f32, String),
}

/// Parses a CSS integer and returns a canonical decimal representation.
pub fn parse_css_integer(value: &str) -> Option<String> {
    let value = value.trim();
    let (negative, digits) = match value.as_bytes().first() {
        Some(b'+') => (false, &value[1..]),
        Some(b'-') => (true, &value[1..]),
        _ => (false, value),
    };
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return Some("0".to_owned());
    }
    Some(if negative {
        format!("-{digits}")
    } else {
        digits.to_owned()
    })
}

/// Parses a CSS `font-weight` value as a number, resolving its keywords.
pub fn parse_css_font_weight(value: &str) -> Option<f32> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(400.0),
        "bold" => Some(700.0),
        _ => parse_css_number(value).filter(|weight| (1.0..=1000.0).contains(weight)),
    }
}

pub(crate) fn parse_css_numeric_token(value: &str) -> Option<CssNumericToken> {
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    let token = parser.next().ok()?;
    let result = match token {
        Token::Number { value, .. } if value.is_finite() => CssNumericToken::Number(*value),
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => {
            CssNumericToken::Percentage(*unit_value * 100.0)
        }
        Token::Dimension { value, unit, .. } if value.is_finite() => {
            CssNumericToken::Dimension(*value, unit.to_ascii_lowercase())
        }
        _ => return None,
    };
    parser.is_exhausted().then_some(result)
}

pub fn parse_css_number(value: &str) -> Option<f32> {
    match parse_css_numeric_token(value.trim())? {
        CssNumericToken::Number(number) => Some(number),
        _ => None,
    }
}

pub fn parse_css_percentage(value: &str) -> Option<f32> {
    match parse_css_numeric_token(value.trim())? {
        CssNumericToken::Percentage(percentage) => Some(percentage),
        _ => None,
    }
}

pub fn parse_css_dimension(value: &str) -> Option<(f32, String)> {
    match parse_css_numeric_token(value.trim())? {
        CssNumericToken::Dimension(number, unit) => Some((number, unit)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_css_font_weight, parse_css_integer, parse_css_number, parse_css_percentage};

    #[test]
    fn parses_css_font_weight_numbers_keywords_and_fractional_values() {
        for (value, expected) in [
            ("normal", 400.0),
            ("bold", 700.0),
            ("+3e2", 300.0),
            ("300.5", 300.5),
            ("1000.0", 1000.0),
        ] {
            assert_eq!(parse_css_font_weight(value), Some(expected));
        }
        for value in ["bolder", "0", "1001", "1e999", "700 extra"] {
            assert_eq!(parse_css_font_weight(value), None, "{value}");
        }
    }

    #[test]
    fn parses_css_integers_without_a_machine_range_limit() {
        assert_eq!(parse_css_integer("+00042"), Some("42".to_owned()));
        assert_eq!(parse_css_integer("-000"), Some("0".to_owned()));
        assert_eq!(
            parse_css_integer("9999999999999999999999999999999999999999"),
            Some("9999999999999999999999999999999999999999".to_owned())
        );
        for value in ["", "+", "-", "1.0", "1e2", "1 2", "١"] {
            assert_eq!(parse_css_integer(value), None, "{value}");
        }
    }

    #[test]
    fn parses_css_numbers_and_percentages_without_accepting_other_tokens() {
        assert_eq!(parse_css_number("+1.25e2"), Some(125.0));
        assert_eq!(parse_css_number("-.5"), Some(-0.5));
        assert_eq!(parse_css_percentage("12.5%"), Some(12.5));
        for value in ["12%", "12px", "NaN", "Infinity", "1e999", "1 trailing"] {
            assert_eq!(parse_css_number(value), None, "{value}");
        }
        for value in ["12", "12 %", "12px", "1e999%", "12% trailing"] {
            assert_eq!(parse_css_percentage(value), None, "{value}");
        }
    }

    #[test]
    fn parses_decimal_numbers_independently_of_locale_conventions() {
        assert_eq!(parse_css_number("1.5"), Some(1.5));
        assert_eq!(parse_css_number("1,5"), None);
        assert_eq!(parse_css_number("١٫٥"), None);
    }
}
