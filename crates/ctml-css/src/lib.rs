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
    ElementIdentity, SelectorPosition, SelectorSibling, SelectorTreeAncestor, SelectorTreeElement,
    selector_matches, selector_matches_with_attributes, selector_matches_with_context,
    selector_matches_with_selector_tree_context, selector_matches_with_tree_context,
    selector_specificity,
};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stylesheet {
    pub rules: Vec<CssRule>,
    /// Layer names in normal cascade order, from lowest to highest priority.
    pub layers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CssRule {
    pub selectors: Vec<String>,
    pub declarations: Vec<CssDeclaration>,
    /// Index into [`Stylesheet::layers`], or `None` for unlayered rules.
    pub layer: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CssDeclaration {
    pub property: String,
    pub value: String,
}

/// Parses qualified rules, declarations, and named cascade layers.
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
    let mut state = LayerParseState::default();
    parse_rule_list(&mut input, &[], &mut state).map_err(|error| {
        located_error(
            error.location.line + line_offset,
            error.location.column,
            error,
        )
    })?;
    let layers = state.ordered_layers();
    let rules = state
        .rules
        .into_iter()
        .map(|(layer_name, mut rule)| {
            rule.layer = layer_name
                .as_ref()
                .and_then(|name| layers.iter().position(|layer| layer == name));
            rule
        })
        .collect();
    Ok(Stylesheet { rules, layers })
}

#[derive(Default)]
struct LayerParseState {
    roots: Vec<LayerNode>,
    rules: Vec<(Option<String>, CssRule)>,
    anonymous_layer_count: usize,
}

#[derive(Default)]
struct LayerNode {
    name: String,
    path: String,
    children: Vec<LayerNode>,
}

impl LayerParseState {
    fn register_anonymous(&mut self, current_layer: &[String]) -> String {
        let name = format!("<anonymous-layer-{}>", self.anonymous_layer_count);
        self.anonymous_layer_count += 1;
        let mut path = current_layer.to_vec();
        path.push(name);
        self.register(&path)
    }

    fn register(&mut self, parts: &[String]) -> String {
        let mut siblings = &mut self.roots;
        let mut path = Vec::new();
        for part in parts {
            path.push(part.as_str());
            let full_path = path.join(".");
            let index = siblings.iter().position(|layer| layer.name == *part);
            let index = index.unwrap_or_else(|| {
                siblings.push(LayerNode {
                    name: part.clone(),
                    path: full_path.clone(),
                    children: Vec::new(),
                });
                siblings.len() - 1
            });
            siblings = &mut siblings[index].children;
        }
        path.join(".")
    }

    fn ordered_layers(&self) -> Vec<String> {
        fn append(layers: &[LayerNode], output: &mut Vec<String>) {
            for layer in layers {
                append(&layer.children, output);
                output.push(layer.path.clone());
            }
        }
        let mut output = Vec::new();
        append(&self.roots, &mut output);
        output
    }
}

fn parse_rule_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    current_layer: &[String],
    state: &mut LayerParseState,
) -> Result<(), cssparser::ParseError<'i, ()>> {
    while !input.is_exhausted() {
        input.skip_whitespace();
        if input.is_exhausted() {
            break;
        }
        let prelude = input.parse_until_before::<_, _, ()>(
            Delimiter::CurlyBracketBlock | Delimiter::Semicolon,
            |prelude| {
                let start = prelude.position();
                while prelude.next_including_whitespace().is_ok() {}
                Ok(prelude.slice(start..prelude.position()).trim().to_owned())
            },
        )?;
        let statement = input.try_parse(|input| input.expect_semicolon()).is_ok();
        if let Some(layer_header) = layer_at_rule(&prelude) {
            if layer_header.is_empty() {
                if statement {
                    return Err(input.new_custom_error(()));
                }
                let path = state.register_anonymous(current_layer);
                let path = path.split('.').map(str::to_owned).collect::<Vec<_>>();
                input.expect_curly_bracket_block()?;
                input.parse_nested_block(|nested| parse_rule_list(nested, &path, state))?;
                continue;
            }
            let names = parse_layer_names(layer_header).map_err(|_| input.new_custom_error(()))?;
            if statement {
                for name in names {
                    let path = current_layer
                        .iter()
                        .cloned()
                        .chain(name)
                        .collect::<Vec<_>>();
                    state.register(&path);
                }
            } else {
                if names.len() != 1 {
                    return Err(input.new_custom_error(()));
                }
                let path = current_layer
                    .iter()
                    .cloned()
                    .chain(names.into_iter().next().unwrap())
                    .collect::<Vec<_>>();
                state.register(&path);
                input.expect_curly_bracket_block()?;
                input.parse_nested_block(|nested| parse_rule_list(nested, &path, state))?;
            }
            continue;
        }
        if prelude.starts_with('@') || statement {
            return Err(input.new_custom_error(()));
        }
        let selectors = split_selector_list(&prelude)
            .into_iter()
            .map(str::trim)
            .filter(|selector| !selector.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if selectors.is_empty() {
            return Err(input.new_custom_error(()));
        }
        input.expect_curly_bracket_block()?;
        let declarations = input.parse_nested_block(parse_declaration_block)?;
        let layer_name = (!current_layer.is_empty()).then(|| current_layer.join("."));
        state.rules.push((
            layer_name,
            CssRule {
                selectors,
                declarations,
                layer: None,
            },
        ));
    }
    Ok(())
}

fn layer_at_rule(prelude: &str) -> Option<&str> {
    let name = prelude.get(..6)?;
    if !name.eq_ignore_ascii_case("@layer") {
        return None;
    }
    let rest = &prelude[6..];
    (rest.is_empty() || rest.starts_with(char::is_whitespace)).then_some(rest.trim())
}

fn parse_layer_names(header: &str) -> Result<Vec<Vec<String>>, ()> {
    if header.is_empty() {
        return Err(());
    }
    header
        .split(',')
        .map(|name| {
            let parts = name.trim().split('.').map(str::trim).collect::<Vec<_>>();
            if parts.is_empty() || parts.iter().any(|part| !is_layer_identifier(part)) {
                return Err(());
            }
            Ok(parts.into_iter().map(str::to_owned).collect())
        })
        .collect()
}

fn is_layer_identifier(identifier: &str) -> bool {
    let characters = identifier.chars().collect::<Vec<_>>();
    let Some(first) = characters.first() else {
        return false;
    };
    let starts_with_identifier = if *first == '-' {
        characters
            .get(1)
            .is_some_and(|character| *character == '-' || is_layer_name_start(*character))
    } else {
        is_layer_name_start(*first)
    };
    starts_with_identifier
        && characters.iter().skip(1).all(|character| {
            *character == '_'
                || *character == '-'
                || character.is_alphanumeric()
                || !character.is_ascii()
        })
}

fn is_layer_name_start(character: char) -> bool {
    character == '_' || character.is_alphabetic() || !character.is_ascii()
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
            let property = declaration.expect_ident()?;
            let property = if property.starts_with("--") {
                property.to_string()
            } else {
                property.to_ascii_lowercase()
            };
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
            && (!value.is_empty() || property.starts_with("--"))
        {
            declarations.push(CssDeclaration { property, value });
        }
    }
    Ok(declarations)
}
