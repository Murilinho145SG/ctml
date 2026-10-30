use crate::style::TextStyle;

use cssparser::{Parser, ParserInput, Token};

pub(super) fn apply_quotes(style: &mut TextStyle, value: &str) {
    if value.eq_ignore_ascii_case("auto") {
        style.quotes = default_quotes();
        return;
    }
    if value.eq_ignore_ascii_case("none") {
        style.quotes.clear();
        return;
    }

    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    let mut strings = Vec::new();
    while !parser.is_exhausted() {
        let Ok(token) = parser.next() else {
            return;
        };
        match token {
            Token::QuotedString(text) => strings.push(text.to_string()),
            Token::WhiteSpace(_) => {}
            _ => return,
        }
    }
    if strings.len() < 2 || strings.len() % 2 != 0 {
        return;
    }
    style.quotes = strings
        .chunks_exact(2)
        .map(|pair| (pair[0].clone(), pair[1].clone()))
        .collect();
}

pub(super) fn default_quotes() -> Vec<(String, String)> {
    vec![
        ("“".to_owned(), "”".to_owned()),
        ("‘".to_owned(), "’".to_owned()),
    ]
}
