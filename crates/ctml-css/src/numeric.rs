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
