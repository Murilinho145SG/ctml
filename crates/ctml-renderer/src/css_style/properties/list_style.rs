use crate::style::TextStyle;
use cssparser::{Parser, ParserInput, Token};

const SUPPORTED_LIST_STYLE_TYPES: &[&str] = &[
    "disc",
    "circle",
    "square",
    "disclosure-open",
    "disclosure-closed",
    "decimal",
    "decimal-leading-zero",
    "lower-alpha",
    "upper-alpha",
    "lower-greek",
    "lower-roman",
    "upper-roman",
    "armenian",
    "upper-armenian",
    "lower-armenian",
    "hebrew",
    "georgian",
    "cjk-decimal",
    "cjk-ideographic",
    "cjk-earthly-branch",
    "cjk-heavenly-stem",
    "arabic-indic",
    "bengali",
    "cambodian",
    "khmer",
    "devanagari",
    "gujarati",
    "gurmukhi",
    "kannada",
    "lao",
    "malayalam",
    "mongolian",
    "myanmar",
    "oriya",
    "persian",
    "tamil",
    "telugu",
    "thai",
    "tibetan",
    "ethiopic-numeric",
    "japanese-informal",
    "japanese-formal",
    "simp-chinese-informal",
    "simp-chinese-formal",
    "trad-chinese-informal",
    "trad-chinese-formal",
    "korean-hangul-formal",
    "korean-hanja-informal",
    "korean-hanja-formal",
    "lower-latin",
    "upper-latin",
    "hiragana",
    "hiragana-iroha",
    "katakana",
    "katakana-iroha",
    "none",
];

pub(super) fn apply_list_style_type(style: &mut TextStyle, value: &str) {
    let value = value.trim();
    let keyword = value.to_ascii_lowercase();
    if SUPPORTED_LIST_STYLE_TYPES.contains(&keyword.as_str()) {
        style.list_style_type = Some(keyword);
    } else if let Some(symbols_style) = parse_symbols_function(value) {
        style.list_style_type = Some(symbols_style);
    } else if let Some(marker) = ctml_css::parse_css_string(value) {
        style.list_style_type = Some(format!("custom:{marker}"));
    }
}

fn parse_symbols_function(value: &str) -> Option<String> {
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    let Token::Function(name) = parser.next().ok()? else {
        return None;
    };
    if !name.eq_ignore_ascii_case("symbols") {
        return None;
    }
    let (system, first_value, symbols) = parser
        .parse_nested_block(|nested| {
            nested.skip_whitespace();
            let (system, first_value, first_symbol) = match nested.next()? {
                Token::Ident(name) => {
                    let system = parse_symbols_system(&name)
                        .ok_or_else(|| nested.new_custom_error::<_, ()>(()))?;
                    let first_value = if system == "fixed" {
                        nested.skip_whitespace();
                        match nested.try_parse(|nested| nested.expect_integer()) {
                            Ok(value) => value,
                            Err(_) => 1,
                        }
                    } else {
                        1
                    };
                    (system, first_value, None)
                }
                Token::QuotedString(symbol) => ("cyclic", 1, Some(symbol.to_string())),
                _ => return Err(nested.new_custom_error::<_, ()>(())),
            };
            let mut symbols = Vec::new();
            if let Some(symbol) = first_symbol {
                symbols.push(symbol);
            }
            while !nested.is_exhausted() {
                nested.skip_whitespace();
                if nested.is_exhausted() {
                    break;
                }
                let Token::QuotedString(symbol) = nested.next()? else {
                    return Err(nested.new_custom_error::<_, ()>(()));
                };
                symbols.push(symbol.to_string());
            }
            let minimum = if matches!(system, "alphabetic" | "numeric") {
                2
            } else {
                1
            };
            if symbols.len() < minimum {
                return Err(nested.new_custom_error::<_, ()>(()));
            }
            Ok((system, first_value, symbols))
        })
        .ok()?;
    if !parser.is_exhausted() {
        return None;
    }
    let encoded_symbols = symbols
        .iter()
        .map(|symbol| format!("{}:{symbol}", symbol.len()))
        .collect::<String>();
    Some(format!("symbols:{system}:{first_value}:{encoded_symbols}"))
}

fn parse_symbols_system(value: &str) -> Option<&'static str> {
    match value.to_ascii_lowercase().as_str() {
        "cyclic" => Some("cyclic"),
        "fixed" => Some("fixed"),
        "symbolic" => Some("symbolic"),
        "alphabetic" => Some("alphabetic"),
        "numeric" => Some("numeric"),
        _ => None,
    }
}

pub(super) fn apply_list_style_shorthand(style: &mut TextStyle, value: &str) {
    apply_list_style_type(style, value);
}

pub(super) fn reset_list_style_type(style: &mut TextStyle) {
    style.list_style_type = Some("disc".to_owned());
}
