use cssparser::{Parser, ParserInput, Token};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CssNumericToken {
    Number(f32),
    Percentage(f32),
    Dimension(f32, String),
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
    use super::{parse_css_number, parse_css_percentage};

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
}
