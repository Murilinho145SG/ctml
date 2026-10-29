use cssparser::{Parser, ParserInput, Token};

pub fn parse_font_feature_settings(value: &str) -> Option<Vec<([u8; 4], u32)>> {
    if value.trim().eq_ignore_ascii_case("normal") {
        return Some(Vec::new());
    }
    let mut source = ParserInput::new(value);
    let mut input = Parser::new(&mut source);
    let mut features = Vec::new();
    while !input.is_exhausted() {
        input.skip_whitespace();
        if input.is_exhausted() {
            break;
        }
        let tag = input.expect_string().ok()?.as_bytes().to_vec();
        if tag.len() != 4
            || !tag
                .iter()
                .all(|character| (0x20..=0x7e).contains(character))
        {
            return None;
        }
        let tag: [u8; 4] = tag.try_into().ok()?;
        input.skip_whitespace();
        let comma_consumed =
            !input.is_exhausted() && input.try_parse(|input| input.expect_comma()).is_ok();
        let default = input.is_exhausted() || comma_consumed;
        if comma_consumed {
            input.skip_whitespace();
            if input.is_exhausted() {
                return None;
            }
        }
        let setting = if default {
            1
        } else {
            match input.next().ok()? {
                Token::Ident(value) if value.eq_ignore_ascii_case("on") => 1,
                Token::Ident(value) if value.eq_ignore_ascii_case("off") => 0,
                Token::Number {
                    int_value: Some(value),
                    ..
                } if *value >= 0 => *value as u32,
                _ => return None,
            }
        };
        features.push((tag, setting));
        if input.is_exhausted() {
            break;
        }
        if default {
            continue;
        }
        input.expect_comma().ok()?;
        input.skip_whitespace();
        if input.is_exhausted() {
            return None;
        }
    }
    (!features.is_empty()).then_some(features)
}

#[cfg(test)]
mod tests {
    use super::parse_font_feature_settings;

    #[test]
    fn parses_feature_tags_and_defaults() {
        assert_eq!(
            parse_font_feature_settings("\"sinf\" 3, \"liga\" off, \"ABCD\""),
            Some(vec![(*b"sinf", 3), (*b"liga", 0), (*b"ABCD", 1)])
        );
        assert_eq!(parse_font_feature_settings("normal"), Some(Vec::new()));
    }

    #[test]
    fn rejects_invalid_tags_settings_and_lists() {
        for value in [
            "\"abc\" on",
            "\"éééé\" on",
            "\"\u{0001}abc\" on",
            "\"sinf\" -1",
            "\"sinf\" 1.5",
            "\"sinf\" maybe",
            "\"sinf\",",
            "\"sinf\" on,",
        ] {
            assert_eq!(parse_font_feature_settings(value), None, "{value}");
        }
    }
}
