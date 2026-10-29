use cssparser::{Delimiter, Parser, ParserInput};

mod color;
mod comments;
mod imports;
mod length;
mod selector;

pub use color::{parse_color, parse_color_with_opacity};
pub use imports::parse_stylesheet_file;
pub use length::{parse_length, parse_nonnegative_length, parse_signed_length_percentage};
pub use selector::{
    ElementIdentity, SelectorSibling, selector_matches, selector_matches_with_attributes,
    selector_matches_with_context, selector_matches_with_tree_context, selector_specificity,
};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stylesheet {
    pub rules: Vec<CssRule>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CssRule {
    pub selectors: Vec<String>,
    pub declarations: Vec<CssDeclaration>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CssDeclaration {
    pub property: String,
    pub value: String,
}

/// Parses a stylesheet's qualified rules and declarations.
///
/// Selectors and values are retained as source text for the native styling layer.
/// At-rules are outside the initial CTML CSS subset.
pub fn parse_stylesheet(source: &str) -> Result<Stylesheet, String> {
    parse_stylesheet_with_line_offset(source, 0)
}

pub(crate) fn parse_stylesheet_with_line_offset(
    source: &str,
    line_offset: u32,
) -> Result<Stylesheet, String> {
    let source = comments::strip_utf8_bom_and_comments(source)?;
    let mut source = ParserInput::new(&source);
    let mut input = Parser::new(&mut source);
    let mut rules = Vec::new();

    while !input.is_exhausted() {
        input.skip_whitespace();
        if input.is_exhausted() {
            break;
        }

        let selector_text = input
            .parse_until_before::<_, _, ()>(Delimiter::CurlyBracketBlock, |prelude| {
                let start = prelude.position();
                while prelude.next_including_whitespace().is_ok() {}
                Ok(prelude.slice(start..prelude.position()).trim().to_owned())
            })
            .map_err(|error| {
                located_error(
                    error.location.line + line_offset,
                    error.location.column,
                    error,
                )
            })?;
        input.expect_curly_bracket_block().map_err(|error| {
            located_error(
                error.location.line + line_offset,
                error.location.column,
                error,
            )
        })?;
        let declarations = input
            .parse_nested_block(parse_declaration_block)
            .map_err(|error| {
                located_error(
                    error.location.line + line_offset,
                    error.location.column,
                    error,
                )
            })?;

        let selectors = split_selector_list(&selector_text)
            .into_iter()
            .map(|selector| selector.trim())
            .filter(|selector| !selector.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if selectors.is_empty() {
            return Err("CSS rule is missing a selector".to_owned());
        }
        rules.push(CssRule {
            selectors,
            declarations,
        });
    }

    Ok(Stylesheet { rules })
}

fn located_error(line: u32, column: u32, error: impl std::fmt::Debug) -> String {
    format!("line {}, column {column}: {error:?}", line + 1)
}

/// Parses a CSS declaration list, such as the value of an element's `style` attribute.
/// Invalid declarations are discarded while later declarations remain available.
pub fn parse_declarations(source: &str) -> Result<Vec<CssDeclaration>, String> {
    let source = comments::strip_utf8_bom_and_comments(source)?;
    let mut source = ParserInput::new(&source);
    let mut input = Parser::new(&mut source);
    parse_declaration_block(&mut input)
        .map_err(|error| located_error(error.location.line, error.location.column, error))
}

fn split_selector_list(source: &str) -> Vec<&str> {
    let mut selectors = Vec::new();
    let mut start = 0;
    let mut bracket_depth = 0;
    let mut paren_depth = 0;
    let mut quote = None;
    for (index, character) in source.char_indices() {
        if bracket_depth > 0 {
            match (quote, character) {
                (Some(current), character) if current == character => quote = None,
                (None, '"' | '\'') => quote = Some(character),
                (None, '[') => bracket_depth += 1,
                (None, ']') => bracket_depth -= 1,
                _ => {}
            }
        } else if character == '[' {
            bracket_depth = 1;
        } else if character == '(' {
            paren_depth += 1;
        } else if character == ')' {
            paren_depth -= 1;
        } else if character == ',' && paren_depth == 0 {
            selectors.push(&source[start..index]);
            start = index + character.len_utf8();
        }
    }
    selectors.push(&source[start..]);
    selectors
}

fn parse_declaration_block<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<Vec<CssDeclaration>, cssparser::ParseError<'i, ()>> {
    let mut declarations = Vec::new();
    while !input.is_exhausted() {
        input.skip_whitespace();
        if input.is_exhausted() {
            break;
        }
        let parsed = input.parse_until_after::<_, _, ()>(Delimiter::Semicolon, |declaration| {
            let property = declaration.expect_ident()?.to_ascii_lowercase();
            declaration.expect_colon()?;
            declaration.skip_whitespace();
            let value_start = declaration.position();
            while declaration.next_including_whitespace().is_ok() {}
            let value = declaration
                .slice(value_start..declaration.position())
                .trim()
                .to_owned();
            Ok((property, value))
        });
        if let Ok((property, value)) = parsed
            && !value.is_empty()
        {
            declarations.push(CssDeclaration { property, value });
        }
    }
    Ok(declarations)
}
