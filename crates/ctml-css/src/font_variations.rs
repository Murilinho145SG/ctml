use cssparser::{Parser, ParserInput, Token};

pub fn parse_font_variation_settings(value: &str) -> Option<Vec<([u8; 4], f32)>> {
    if value.trim().eq_ignore_ascii_case("normal") {
        return Some(Vec::new());
    }

    let mut source = ParserInput::new(value);
    let mut input = Parser::new(&mut source);
    let mut settings = Vec::new();
    while !input.is_exhausted() {
        let tag = input.expect_string().ok()?.as_bytes().to_vec();
        if tag.len() != 4 || !tag.iter().all(|byte| (0x20..=0x7e).contains(byte)) {
            return None;
        }
        input.skip_whitespace();
        let value = match input.next().ok()? {
            Token::Number { value, .. } if value.is_finite() => *value,
            _ => return None,
        };
        settings.push((tag.try_into().ok()?, value));
        input.skip_whitespace();
        if input.is_exhausted() {
            break;
        }
        input.expect_comma().ok()?;
        input.skip_whitespace();
        if input.is_exhausted() {
            return None;
        }
    }
    (!settings.is_empty()).then_some(settings)
}

#[cfg(test)]
mod tests {
    use super::parse_font_variation_settings;

    #[test]
    fn parses_normal_and_variation_axis_lists() {
        assert_eq!(parse_font_variation_settings("normal"), Some(Vec::new()));
        assert_eq!(
            parse_font_variation_settings("\"wght\" 620, \"opsz\" 14.5"),
            Some(vec![(*b"wght", 620.0), (*b"opsz", 14.5)])
        );
    }

    #[test]
    fn rejects_invalid_variation_axes() {
        for value in [
            "\"wgt\" 500",
            "\"wght\" -infinity",
            "\"wght\" 500,",
            "\"wght\" 500 \"opsz\" 12",
            "wght 500",
        ] {
            assert_eq!(parse_font_variation_settings(value), None, "{value}");
        }
    }
}
