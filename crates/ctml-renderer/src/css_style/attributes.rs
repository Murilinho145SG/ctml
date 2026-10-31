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
    dark_scheme: bool,
) -> Option<String> {
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    let mut output = String::with_capacity(value.len());
    serialize_tokens(
        &mut parser,
        attributes,
        color_profiles,
        dark_scheme,
        &mut output,
    )
    .ok()?;
    Some(output)
}

fn serialize_tokens<'i, 't>(
    parser: &mut Parser<'i, 't>,
    attributes: &[(String, Option<String>)],
    color_profiles: &[ctml_css::CssColorProfile],
    dark_scheme: bool,
    output: &mut String,
) -> Result<(), cssparser::ParseError<'i, ()>> {
    while !parser.is_exhausted() {
        let token = parser.next_including_whitespace()?;
        match token {
            Token::Function(name) if name.eq_ignore_ascii_case("env") => {
                let (name, fallback) = parser.parse_nested_block(|nested| {
                    parse_env_arguments(nested, attributes, color_profiles, dark_scheme)
                })?;
                if is_zero_safe_area_environment_variable(&name) {
                    output.push_str("0px");
                } else if let Some(fallback) = fallback {
                    output.push_str(&fallback);
                } else {
                    return Err(parser.new_custom_error(()));
                }
            }
            Token::Function(name) if name.eq_ignore_ascii_case("attr") => {
                let (name, attr_type, fallback) = parser.parse_nested_block(|nested| {
                    parse_attr_arguments(nested, color_profiles, dark_scheme)
                })?;
                let raw_value = attributes
                    .iter()
                    .find(|(attribute, _)| attribute.eq_ignore_ascii_case(&name))
                    .and_then(|(_, value)| value.as_deref());
                let converted = raw_value.and_then(|value| {
                    convert_attribute(value, attr_type, color_profiles, dark_scheme)
                });
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
                    serialize_tokens(nested, attributes, color_profiles, dark_scheme, output)
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

fn parse_env_arguments<'i, 't>(
    parser: &mut Parser<'i, 't>,
    attributes: &[(String, Option<String>)],
    color_profiles: &[ctml_css::CssColorProfile],
    dark_scheme: bool,
) -> Result<(String, Option<String>), cssparser::ParseError<'i, ()>> {
    let name = parser.expect_ident_cloned()?.to_string();
    parser.skip_whitespace();
    if parser.is_exhausted() {
        return Ok((name, None));
    }
    parser.expect_comma()?;
    let mut fallback = String::new();
    serialize_tokens(
        parser,
        attributes,
        color_profiles,
        dark_scheme,
        &mut fallback,
    )?;
    Ok((name, Some(fallback)))
}

fn is_zero_safe_area_environment_variable(name: &str) -> bool {
    ["safe-area-inset-", "safe-area-max-inset-"]
        .iter()
        .any(|prefix| {
            name.strip_prefix(prefix)
                .is_some_and(|side| matches!(side, "top" | "right" | "bottom" | "left"))
        })
}

fn parse_attr_arguments<'i, 't>(
    parser: &mut Parser<'i, 't>,
    color_profiles: &[ctml_css::CssColorProfile],
    dark_scheme: bool,
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
        serialize_tokens(parser, &[], color_profiles, dark_scheme, &mut output)?;
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
    dark_scheme: bool,
) -> Option<String> {
    match attr_type {
        None | Some(AttrType::String) => {
            let mut serialized = String::new();
            Token::QuotedString(value.into())
                .to_css(&mut serialized)
                .expect("writing CSS tokens into a String cannot fail");
            Some(serialized)
        }
        Some(AttrType::Integer) => ctml_css::parse_css_integer(value).or_else(|| {
            let value = ctml_css::parse_math_number(value)?;
            let rounded = (value as f64 + 0.5).floor();
            (rounded >= i64::MIN as f64 && rounded <= i64::MAX as f64)
                .then(|| (rounded as i64).to_string())
        }),
        Some(AttrType::Number) => ctml_css::parse_math_number(value)
            .or_else(|| ctml_css::parse_css_number(value))
            .map(|value| value.to_string()),
        Some(AttrType::Percentage) => ctml_css::parse_math_percentage(value)
            .or_else(|| ctml_css::parse_css_percentage(value))
            .map(|value| format!("{value}%")),
        Some(AttrType::Length) => {
            parse_dimension(value, LENGTH_UNITS).or_else(|| parse_length_calculation(value, false))
        }
        Some(AttrType::LengthPercentage) => ctml_css::parse_css_percentage(value)
            .map(|value| format!("{value}%"))
            .or_else(|| parse_dimension(value, LENGTH_UNITS))
            .or_else(|| parse_length_calculation(value, true)),
        Some(AttrType::Angle) => parse_dimension(value, ANGLE_UNITS)
            .or_else(|| ctml_css::parse_math_angle(value).map(|radians| format!("{radians}rad"))),
        Some(AttrType::Color) => ctml_css::parse_color_with_opacity_and_profiles_for_scheme(
            value,
            color_profiles,
            dark_scheme,
        )
        .map(|(rgb, alpha)| {
            format!(
                "#{:02x}{:02x}{:02x}{:02x}",
                rgb[0],
                rgb[1],
                rgb[2],
                (alpha * 255.0).round() as u8
            )
        }),
        Some(AttrType::CustomIdent) => parse_custom_ident(value),
    }
}

fn parse_length_calculation(value: &str, allow_percentage: bool) -> Option<String> {
    ctml_css::parse_math_expression(value, |component| {
        ctml_css::parse_css_number(component)
            .filter(|number| *number == 0.0)
            .or_else(|| {
                allow_percentage
                    .then(|| ctml_css::parse_css_percentage(component))
                    .flatten()
            })
            .or_else(|| {
                let (number, unit) = ctml_css::parse_css_dimension(component)?;
                LENGTH_UNITS.contains(&unit.as_str()).then_some(number)
            })
    })
    .map(|_| value.trim().to_owned())
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
