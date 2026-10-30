use cssparser::{Parser, ParserInput, ToCss, Token};

const LENGTH_UNITS: &[&str] = &[
    "px", "cm", "mm", "q", "in", "pt", "pc", "em", "rem", "ex", "rex", "ch", "rch", "cap", "rcap",
    "ic", "ric", "lh", "rlh", "vw", "vh", "vi", "vb", "vmin", "vmax", "svw", "svh", "svi", "svb",
    "svmin", "svmax", "lvw", "lvh", "lvi", "lvb", "lvmin", "lvmax", "dvw", "dvh", "dvi", "dvb",
    "dvmin", "dvmax",
];
const ANGLE_UNITS: &[&str] = &["deg", "grad", "rad", "turn"];

pub(super) fn substitute_attr_functions(
    value: &str,
    attributes: &[(String, Option<String>)],
    color_profiles: &[ctml_css::CssColorProfile],
) -> Option<String> {
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    let mut output = String::with_capacity(value.len());
    serialize_tokens(&mut parser, attributes, color_profiles, &mut output).ok()?;
    Some(output)
}

fn serialize_tokens<'i, 't>(
    parser: &mut Parser<'i, 't>,
    attributes: &[(String, Option<String>)],
    color_profiles: &[ctml_css::CssColorProfile],
    output: &mut String,
) -> Result<(), cssparser::ParseError<'i, ()>> {
    while !parser.is_exhausted() {
        let token = parser.next_including_whitespace()?;
        match token {
            Token::Function(name) if name.eq_ignore_ascii_case("attr") => {
                let (name, attr_type, fallback) = parser
                    .parse_nested_block(|nested| parse_attr_arguments(nested, color_profiles))?;
                let raw_value = attributes
                    .iter()
                    .find(|(attribute, _)| attribute.eq_ignore_ascii_case(&name))
                    .and_then(|(_, value)| value.as_deref());
                let converted =
                    raw_value.and_then(|value| convert_attribute(value, attr_type, color_profiles));
                if let Some(converted) = converted {
                    output.push_str(&converted);
                } else if let Some(fallback) = fallback {
                    output.push_str(&fallback);
                } else {
                    return Err(parser.new_custom_error(()));
                }
            }
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock => {
                let end = match token {
                    Token::SquareBracketBlock => ']',
                    Token::CurlyBracketBlock => '}',
                    _ => ')',
                };
                token
                    .to_css(output)
                    .expect("writing CSS tokens into a String cannot fail");
                parser.parse_nested_block(|nested| {
                    serialize_tokens(nested, attributes, color_profiles, output)
                })?;
                output.push(end);
            }
            _ => token
                .to_css(output)
                .expect("writing CSS tokens into a String cannot fail"),
        }
    }
    Ok(())
}

fn parse_attr_arguments<'i, 't>(
    parser: &mut Parser<'i, 't>,
    color_profiles: &[ctml_css::CssColorProfile],
) -> Result<(String, Option<AttrType>, Option<String>), cssparser::ParseError<'i, ()>> {
    let name = parser.expect_ident_cloned()?.to_string();
    parser.skip_whitespace();
    if parser.is_exhausted() {
        return Ok((name, None, None));
    }
    let attr_type = match parser.next()? {
        Token::Comma => None,
        Token::Function(function) if function.eq_ignore_ascii_case("type") => {
            Some(parser.parse_nested_block(|nested| {
                nested.expect_delim('<')?;
                let name = nested.expect_ident_cloned()?;
                nested.expect_delim('>')?;
                if !nested.is_exhausted() {
                    return Err(nested.new_custom_error(()));
                }
                AttrType::parse(&name).ok_or_else(|| nested.new_custom_error(()))
            })?)
        }
        _ => return Err(parser.new_custom_error(())),
    };
    if attr_type.is_some() {
        parser.expect_comma()?;
    }
    let fallback = if parser.is_exhausted() {
        None
    } else {
        let mut output = String::new();
        serialize_tokens(parser, &[], color_profiles, &mut output)?;
        Some(output)
    };
    Ok((name, attr_type, fallback))
}

#[derive(Clone, Copy)]
enum AttrType {
    String,
    Integer,
    Number,
    Percentage,
    Length,
    LengthPercentage,
    Angle,
    Color,
    CustomIdent,
}

impl AttrType {
    fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "string" => Some(Self::String),
            "integer" => Some(Self::Integer),
            "number" => Some(Self::Number),
            "percentage" => Some(Self::Percentage),
            "length" => Some(Self::Length),
            "length-percentage" => Some(Self::LengthPercentage),
            "angle" => Some(Self::Angle),
            "color" => Some(Self::Color),
            "custom-ident" => Some(Self::CustomIdent),
            _ => None,
        }
    }
}

fn convert_attribute(
    value: &str,
    attr_type: Option<AttrType>,
    color_profiles: &[ctml_css::CssColorProfile],
) -> Option<String> {
    match attr_type {
        None | Some(AttrType::String) => {
            let mut serialized = String::new();
            Token::QuotedString(value.into())
                .to_css(&mut serialized)
                .expect("writing CSS tokens into a String cannot fail");
            Some(serialized)
        }
        Some(AttrType::Integer) => value
            .trim()
            .parse::<i64>()
            .ok()
            .map(|value| value.to_string()),
        Some(AttrType::Number) => ctml_css::parse_css_number(value).map(|value| value.to_string()),
        Some(AttrType::Percentage) => {
            ctml_css::parse_css_percentage(value).map(|value| format!("{value}%"))
        }
        Some(AttrType::Length) => parse_dimension(value, LENGTH_UNITS),
        Some(AttrType::LengthPercentage) => ctml_css::parse_css_percentage(value)
            .map(|value| format!("{value}%"))
            .or_else(|| parse_dimension(value, LENGTH_UNITS)),
        Some(AttrType::Angle) => parse_dimension(value, ANGLE_UNITS),
        Some(AttrType::Color) => {
            ctml_css::parse_color_with_profiles(value, color_profiles).map(|color| {
                format!(
                    "#{:02x}{:02x}{:02x}{:02x}",
                    color[0], color[1], color[2], color[3]
                )
            })
        }
        Some(AttrType::CustomIdent) => parse_custom_ident(value),
    }
}

fn parse_custom_ident(value: &str) -> Option<String> {
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    let Token::Ident(identifier) = parser.next().ok()? else {
        return None;
    };
    let identifier = identifier.to_string();
    if !parser.is_exhausted()
        || [
            "initial",
            "inherit",
            "unset",
            "revert",
            "revert-layer",
            "default",
        ]
        .iter()
        .any(|keyword| identifier.eq_ignore_ascii_case(keyword))
    {
        return None;
    }
    let mut serialized = String::new();
    Token::Ident(identifier.into())
        .to_css(&mut serialized)
        .expect("writing CSS tokens into a String cannot fail");
    Some(serialized)
}

fn parse_dimension(value: &str, units: &[&str]) -> Option<String> {
    let (number, unit) = ctml_css::parse_css_dimension(value)?;
    units
        .contains(&unit.as_str())
        .then(|| format!("{number}{unit}"))
}
