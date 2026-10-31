use cssparser::{Parser, ParserInput, Token};

use crate::style::TextStyle;

pub(super) fn apply_color_scheme(style: &mut TextStyle, value: &str) {
    let Some(dark_scheme) = parse_color_scheme(value) else {
        return;
    };
    style.dark_color_scheme = dark_scheme;
}

fn parse_color_scheme(value: &str) -> Option<bool> {
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    let mut supported_scheme = None;
    let mut found_scheme = false;
    let mut only = false;

    while !parser.is_exhausted() {
        let Token::Ident(keyword) = parser.next().ok()? else {
            return None;
        };
        if keyword.eq_ignore_ascii_case("normal") {
            if found_scheme || only || !parser.is_exhausted() {
                return None;
            }
            return Some(false);
        }
        if keyword.eq_ignore_ascii_case("only") {
            if !found_scheme || only || !parser.is_exhausted() {
                return None;
            }
            only = true;
            continue;
        }
        if !is_custom_ident(keyword) {
            return None;
        }
        found_scheme = true;
        let scheme = if keyword.eq_ignore_ascii_case("light") {
            Some(false)
        } else if keyword.eq_ignore_ascii_case("dark") {
            Some(true)
        } else {
            None
        };
        if supported_scheme.is_none() {
            supported_scheme = scheme;
        }
    }

    found_scheme.then_some(supported_scheme.unwrap_or(false))
}

fn is_custom_ident(value: &str) -> bool {
    ![
        "default",
        "none",
        "inherit",
        "initial",
        "revert",
        "revert-layer",
        "unset",
        "only",
        "normal",
    ]
    .into_iter()
    .any(|reserved| value.eq_ignore_ascii_case(reserved))
}

#[cfg(test)]
mod tests {
    use super::parse_color_scheme;

    #[test]
    fn selects_the_first_supported_scheme_in_the_list() {
        assert_eq!(parse_color_scheme("dark light"), Some(true));
        assert_eq!(parse_color_scheme("light dark"), Some(false));
        assert_eq!(parse_color_scheme("future dark"), Some(true));
    }

    #[test]
    fn validates_normal_and_only_forms() {
        assert_eq!(parse_color_scheme("normal"), Some(false));
        assert_eq!(parse_color_scheme("dark only"), Some(true));
        assert_eq!(parse_color_scheme("only dark"), None);
        assert_eq!(parse_color_scheme("normal dark"), None);
        assert_eq!(parse_color_scheme("dark, light"), None);
    }
}
