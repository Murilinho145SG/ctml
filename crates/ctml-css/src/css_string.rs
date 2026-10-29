use cssparser::{Parser, ParserInput};

pub fn parse_css_string(value: &str) -> Option<String> {
    let value = value.trim();
    let quote = value.chars().next()?;
    if !matches!(quote, '\'' | '"') || !value.ends_with(quote) {
        return None;
    }
    let trailing_backslashes = value[..value.len() - quote.len_utf8()]
        .bytes()
        .rev()
        .take_while(|byte| *byte == b'\\')
        .count();
    if trailing_backslashes % 2 != 0 {
        return None;
    }

    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    let string = parser.expect_string().ok()?.as_ref().to_owned();
    parser.skip_whitespace();
    parser.is_exhausted().then_some(string)
}

#[cfg(test)]
mod tests {
    use super::parse_css_string;

    #[test]
    fn parses_quoted_css_strings_and_escapes() {
        assert_eq!(parse_css_string("\"mark\""), Some("mark".to_owned()));
        assert_eq!(parse_css_string("'\\25cf'"), Some("●".to_owned()));
        assert_eq!(parse_css_string("\"a\\\"b\""), Some("a\"b".to_owned()));
    }

    #[test]
    fn rejects_non_string_and_trailing_tokens() {
        for value in ["mark", "\"mark\" other", "\"unterminated", "\"mark\\\""] {
            assert_eq!(parse_css_string(value), None, "{value}");
        }
    }
}
