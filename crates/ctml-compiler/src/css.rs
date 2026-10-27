use cssparser::{Delimiter, Parser, ParserInput};

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
    let mut source = ParserInput::new(source);
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
            .map_err(|error| format!("{error:?}"))?;
        input
            .expect_curly_bracket_block()
            .map_err(|error| format!("{error:?}"))?;
        let declarations = input
            .parse_nested_block(parse_declarations)
            .map_err(|error| format!("{error:?}"))?;

        let selectors = selector_text
            .split(',')
            .map(str::trim)
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

fn parse_declarations<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<Vec<CssDeclaration>, cssparser::ParseError<'i, ()>> {
    let mut declarations = Vec::new();
    while !input.is_exhausted() {
        input.skip_whitespace();
        if input.is_exhausted() {
            break;
        }
        let raw = input.parse_until_after(Delimiter::Semicolon, |declaration| {
            let start = declaration.position();
            while declaration.next_including_whitespace().is_ok() {}
            Ok(declaration
                .slice(start..declaration.position())
                .trim()
                .to_owned())
        })?;
        let Some((property, value)) = raw.split_once(':') else {
            continue;
        };
        let property = property.trim();
        let value = value.trim();
        if !property.is_empty() && !value.is_empty() {
            declarations.push(CssDeclaration {
                property: property.to_ascii_lowercase(),
                value: value.to_owned(),
            });
        }
    }
    Ok(declarations)
}
