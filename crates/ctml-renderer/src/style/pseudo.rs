use cssparser::{Parser, ParserInput, ToCss, Token};
use std::collections::HashMap;

use crate::{StyleDeclaration, StyleRule, css_style, style::TextStyle};
use ctml_css::{
    ElementIdentity, SelectorPosition, SelectorSibling, SelectorTreeElement,
    selector_matches_with_selector_tree_context, selector_specificity,
};

mod first_letter;
pub(super) use first_letter::style_first_letters;

pub(super) struct GeneratedContent {
    pub(super) text: String,
    pub(super) style: TextStyle,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_generated_content(
    pseudo_element: &str,
    inherited: &TextStyle,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
    attributes: &[(String, Option<String>)],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
    siblings_before_current: &[SelectorSibling],
    siblings_before_ancestors: &[Vec<SelectorSibling>],
    current_position: SelectorPosition,
    ancestor_positions: &[SelectorPosition],
    selector_tree: &[SelectorTreeElement],
    stylesheet: &[StyleRule],
    quote_depth: &mut usize,
    counters: &HashMap<String, Vec<i32>>,
) -> Option<GeneratedContent> {
    let mut pseudo_rules = Vec::new();
    let mut winning_content = None;
    for (rule_index, rule) in stylesheet.iter().enumerate() {
        let mut base_selectors = Vec::new();
        let mut matching_specificity = None;
        for selector in &rule.selectors {
            let Some(base_selector) = strip_pseudo_element(selector, pseudo_element) else {
                continue;
            };
            if !selector_matches_with_selector_tree_context(
                &base_selector,
                current,
                ancestors,
                attributes,
                ancestor_attributes,
                siblings_before_current,
                siblings_before_ancestors,
                current_position,
                ancestor_positions,
                selector_tree,
            ) {
                continue;
            }
            let specificity = add_pseudo_element_specificity(selector_specificity(&base_selector));
            matching_specificity = Some(
                matching_specificity.map_or(specificity, |current: (usize, usize, usize)| {
                    current.max(specificity)
                }),
            );
            base_selectors.push(base_selector);
        }
        let Some(specificity) = matching_specificity else {
            continue;
        };
        pseudo_rules.push(StyleRule {
            selectors: base_selectors,
            declarations: rule.declarations.clone(),
            layer: rule.layer,
        });
        for (declaration_index, declaration) in rule.declarations.iter().enumerate() {
            if !declaration.property.eq_ignore_ascii_case("content") {
                continue;
            }
            let (_, important) = declaration_value(&declaration.value);
            let priority = (important, specificity, rule_index, declaration_index);
            if winning_content.as_ref().is_none_or(
                |(current_priority, _): &(
                    (bool, (usize, usize, usize), usize, usize),
                    &StyleDeclaration,
                )| { priority >= *current_priority },
            ) {
                winning_content = Some((priority, declaration));
            }
        }
    }
    let (_, declaration) = winning_content?;
    let (value, _) = declaration_value(&declaration.value);
    let mut counter_inherited = inherited.clone();
    counter_inherited.counter_reset.clear();
    counter_inherited.counter_increment.clear();
    counter_inherited.counter_set.clear();
    let mut style = css_style::resolve_style_with_selector_tree_context(
        &counter_inherited,
        current,
        ancestors,
        attributes,
        ancestor_attributes,
        siblings_before_current,
        siblings_before_ancestors,
        current_position,
        ancestor_positions,
        selector_tree,
        None,
        &pseudo_rules,
    );
    let mut updated_quote_depth = *quote_depth;
    let text = parse_generated_content(
        value,
        attributes,
        &style.quotes,
        &mut updated_quote_depth,
        counters,
    )?;
    *quote_depth = updated_quote_depth;
    style.opacity *= inherited.opacity;
    Some(GeneratedContent { text, style })
}

fn strip_pseudo_element(selector: &str, pseudo_element: &str) -> Option<String> {
    let selector = selector.trim_end();
    let expected_suffix = format!("::{pseudo_element}");
    let legacy_suffix = format!(":{pseudo_element}");
    let suffix = [expected_suffix.as_str(), legacy_suffix.as_str()]
        .into_iter()
        .find(|suffix| selector.to_ascii_lowercase().ends_with(suffix))?;
    let suffix_start = selector.len().checked_sub(suffix.len())?;
    let escaped_colon = selector[..suffix_start]
        .bytes()
        .rev()
        .take_while(|byte| *byte == b'\\')
        .count()
        % 2
        == 1;
    if escaped_colon {
        return None;
    }
    let base = selector.get(..suffix_start)?;
    if !selector.get(base.len()..)?.eq_ignore_ascii_case(suffix) {
        return None;
    }
    let trimmed_base = base.trim_end();
    if trimmed_base.is_empty() {
        return None;
    }
    Some(if trimmed_base.len() != base.len() {
        format!("{trimmed_base} *")
    } else {
        trimmed_base.to_owned()
    })
}

fn add_pseudo_element_specificity(
    (ids, classes, tags): (usize, usize, usize),
) -> (usize, usize, usize) {
    (ids, classes, tags + 1)
}

fn declaration_value(value: &str) -> (&str, bool) {
    let value = value.trim_end();
    let Some(without_priority) = value.get(..value.len().saturating_sub("!important".len())) else {
        return (value, false);
    };
    if value[without_priority.len()..]
        .trim()
        .eq_ignore_ascii_case("!important")
    {
        (without_priority.trim_end(), true)
    } else {
        (value, false)
    }
}

fn parse_generated_content(
    value: &str,
    attributes: &[(String, Option<String>)],
    quotes: &[(String, String)],
    quote_depth: &mut usize,
    counters: &HashMap<String, Vec<i32>>,
) -> Option<String> {
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    let mut output = String::new();
    while !parser.is_exhausted() {
        let token = parser.next().ok()?;
        match token {
            Token::QuotedString(text) => output.push_str(text),
            Token::Ident(name) if name.eq_ignore_ascii_case("open-quote") => {
                if let Some((open, _)) = quote_at(quotes, *quote_depth) {
                    output.push_str(open);
                }
                *quote_depth = quote_depth.saturating_add(1);
            }
            Token::Ident(name) if name.eq_ignore_ascii_case("close-quote") => {
                *quote_depth = quote_depth.saturating_sub(1);
                if let Some((_, close)) = quote_at(quotes, *quote_depth) {
                    output.push_str(close);
                }
            }
            Token::Ident(name) if name.eq_ignore_ascii_case("no-open-quote") => {
                *quote_depth = quote_depth.saturating_add(1);
            }
            Token::Ident(name) if name.eq_ignore_ascii_case("no-close-quote") => {
                *quote_depth = quote_depth.saturating_sub(1);
            }
            Token::Function(name) if name.eq_ignore_ascii_case("attr") => {
                let (attribute_name, attr_type, fallback) =
                    parser.parse_nested_block(parse_attr_arguments).ok()?;
                let attribute_value = attributes
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case(&attribute_name))
                    .and_then(|(_, value)| value.as_deref());
                let converted = attribute_value.and_then(|value| match attr_type {
                    None | Some(AttrType::String) => Some(value.to_owned()),
                    Some(AttrType::Integer) => value
                        .trim()
                        .parse::<i64>()
                        .ok()
                        .map(|value| value.to_string()),
                    Some(AttrType::Number) => {
                        ctml_css::parse_css_number(value).map(|value| value.to_string())
                    }
                    Some(AttrType::Percentage) => ctml_css::parse_css_percentage(value)
                        .map(|value| format_css_dimension(value, "%")),
                    Some(AttrType::Length) => parse_typed_dimension(value, &LENGTH_UNITS),
                    Some(AttrType::LengthPercentage) => ctml_css::parse_css_percentage(value)
                        .map(|value| format_css_dimension(value, "%"))
                        .or_else(|| parse_typed_dimension(value, &LENGTH_UNITS)),
                    Some(AttrType::Angle) => parse_typed_dimension(value, &ANGLE_UNITS),
                    Some(AttrType::Time) => parse_typed_dimension(value, &TIME_UNITS),
                    Some(AttrType::Frequency) => parse_typed_dimension(value, &FREQUENCY_UNITS),
                    Some(AttrType::Resolution) => parse_typed_dimension(value, &RESOLUTION_UNITS),
                });
                if let Some(value) = converted {
                    output.push_str(&value);
                } else if let Some(fallback) = fallback {
                    output.push_str(&fallback);
                }
            }
            Token::Function(name) if name.eq_ignore_ascii_case("counter") => {
                let (counter_name, style_name) =
                    parser.parse_nested_block(parse_counter_arguments).ok()?;
                if let Some(value) = counters.get(&counter_name).and_then(|values| values.last()) {
                    output.push_str(&super::list_marker::format_counter_value(
                        style_name.as_deref().unwrap_or("decimal"),
                        *value,
                    )?);
                }
            }
            Token::Function(name) if name.eq_ignore_ascii_case("counters") => {
                let (counter_name, separator, style_name) =
                    parser.parse_nested_block(parse_counters_arguments).ok()?;
                if let Some(values) = counters.get(&counter_name) {
                    let formatted = values
                        .iter()
                        .map(|value| {
                            super::list_marker::format_counter_value(
                                style_name.as_deref().unwrap_or("decimal"),
                                *value,
                            )
                        })
                        .collect::<Option<Vec<_>>>()?;
                    output.push_str(&formatted.join(&separator));
                }
            }
            Token::WhiteSpace(_) => {}
            _ => return None,
        }
    }
    Some(output)
}

fn parse_attr_arguments<'i, 't>(
    parser: &mut Parser<'i, 't>,
) -> Result<(String, Option<AttrType>, Option<String>), cssparser::ParseError<'i, ()>> {
    let name = parser.expect_ident_cloned()?;
    parser.skip_whitespace();
    if parser.is_exhausted() {
        return Ok((name.to_string(), None, None));
    }
    let attr_type = match parser.next()? {
        Token::Comma => None,
        Token::Function(function) if function.eq_ignore_ascii_case("type") => {
            Some(parser.parse_nested_block(|nested| {
                nested.expect_delim('<')?;
                let type_name = nested.expect_ident_cloned()?;
                nested.expect_delim('>')?;
                if !nested.is_exhausted() {
                    return Err(nested.new_custom_error(()));
                }
                AttrType::parse(&type_name).ok_or_else(|| nested.new_custom_error(()))
            })?)
        }
        _ => return Err(parser.new_custom_error(())),
    };
    if attr_type.is_some() {
        parser.expect_comma()?;
    }
    let fallback = serialize_attr_fallback(parser)?;
    Ok((name.to_string(), attr_type, Some(fallback)))
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
    Time,
    Frequency,
    Resolution,
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
            "time" => Some(Self::Time),
            "frequency" => Some(Self::Frequency),
            "resolution" => Some(Self::Resolution),
            _ => None,
        }
    }
}

const LENGTH_UNITS: &[&str] = &[
    "px", "cm", "mm", "q", "in", "pt", "pc", "em", "rem", "ex", "rex", "ch", "rch", "cap", "rcap",
    "ic", "ric", "lh", "rlh", "vw", "vh", "vi", "vb", "vmin", "vmax", "svw", "svh", "svi", "svb",
    "svmin", "svmax", "lvw", "lvh", "lvi", "lvb", "lvmin", "lvmax", "dvw", "dvh", "dvi", "dvb",
    "dvmin", "dvmax",
];
const ANGLE_UNITS: &[&str] = &["deg", "grad", "rad", "turn"];
const TIME_UNITS: &[&str] = &["s", "ms"];
const FREQUENCY_UNITS: &[&str] = &["hz", "khz"];
const RESOLUTION_UNITS: &[&str] = &["dpi", "dpcm", "dppx", "x"];

fn parse_typed_dimension(value: &str, units: &[&str]) -> Option<String> {
    let (number, unit) = ctml_css::parse_css_dimension(value)?;
    units
        .contains(&unit.as_str())
        .then(|| format_css_dimension(number, &unit))
}

fn format_css_dimension(value: f32, unit: &str) -> String {
    format!("{value}{unit}")
}

fn serialize_attr_fallback<'i, 't>(
    parser: &mut Parser<'i, 't>,
) -> Result<String, cssparser::ParseError<'i, ()>> {
    let mut output = String::new();
    while let Ok(token) = parser.next_including_whitespace() {
        if let Token::QuotedString(value) = token {
            output.push_str(value);
            continue;
        }
        let block_end = match token {
            Token::Function(_) | Token::ParenthesisBlock => Some(')'),
            Token::SquareBracketBlock => Some(']'),
            Token::CurlyBracketBlock => Some('}'),
            _ => None,
        };
        token
            .to_css(&mut output)
            .expect("writing CSS tokens into a String cannot fail");
        if let Some(block_end) = block_end {
            output.push_str(&parser.parse_nested_block(serialize_attr_fallback)?);
            output.push(block_end);
        }
    }
    Ok(output)
}

fn parse_counter_arguments<'i, 't>(
    parser: &mut Parser<'i, 't>,
) -> Result<(String, Option<String>), cssparser::ParseError<'i, ()>> {
    let name = parser.expect_ident_cloned()?;
    parser.skip_whitespace();
    let style = if parser.is_exhausted() {
        None
    } else {
        parser.expect_comma()?;
        let style = parser.expect_ident_cloned()?;
        parser.skip_whitespace();
        Some(style.to_ascii_lowercase())
    };
    if !parser.is_exhausted() {
        return Err(parser.new_custom_error(()));
    }
    Ok((name.to_string(), style))
}

fn parse_counters_arguments<'i, 't>(
    parser: &mut Parser<'i, 't>,
) -> Result<(String, String, Option<String>), cssparser::ParseError<'i, ()>> {
    let name = parser.expect_ident_cloned()?;
    parser.expect_comma()?;
    let separator = match parser.next()?.clone() {
        Token::QuotedString(value) => value.to_string(),
        _ => return Err(parser.new_custom_error(())),
    };
    parser.skip_whitespace();
    let style = if parser.is_exhausted() {
        None
    } else {
        parser.expect_comma()?;
        let style = parser.expect_ident_cloned()?;
        parser.skip_whitespace();
        Some(style.to_ascii_lowercase())
    };
    if !parser.is_exhausted() {
        return Err(parser.new_custom_error(()));
    }
    Ok((name.to_string(), separator, style))
}

fn quote_at(quotes: &[(String, String)], depth: usize) -> Option<(&str, &str)> {
    quotes
        .get(depth.min(quotes.len().saturating_sub(1)))
        .map(|(open, close)| (open.as_str(), close.as_str()))
}
