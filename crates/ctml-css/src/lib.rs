use cssparser::{Delimiter, Parser, ParserInput, ToCss, Token, serialize_identifier};

mod color;
mod comments;
mod css_string;
mod font_features;
mod font_variations;
mod imports;
mod length;
mod math;
mod numeric;
mod resources;
mod selector;

pub use color::{parse_color, parse_color_with_opacity};
pub use css_string::parse_css_string;
pub use font_features::parse_font_feature_settings;
pub use font_variations::parse_font_variation_settings;
pub use imports::parse_stylesheet_file;
pub use length::{
    FontRelativeUnits, ViewportSize, parse_font_relative_length,
    parse_font_relative_length_with_viewport, parse_length, parse_length_with_font_metrics,
    parse_length_with_line_height, parse_nonnegative_font_relative_length,
    parse_nonnegative_font_relative_length_with_viewport, parse_nonnegative_length,
    parse_nonnegative_length_with_line_height, parse_signed_font_relative_length,
    parse_signed_font_relative_length_with_viewport, parse_signed_length_percentage,
    parse_signed_length_percentage_with_line_height,
};
pub use math::{parse_math_expression, parse_math_number};
pub use numeric::{parse_css_dimension, parse_css_number, parse_css_percentage};
pub use resources::{
    CssFontSource, LocalResource, load_css_resource, load_local_resource, parse_css_url,
    parse_font_face_sources,
};
pub use selector::{
    ElementIdentity, SelectorPosition, SelectorSibling, SelectorTreeAncestor, SelectorTreeElement,
    selector_matches, selector_matches_with_attributes, selector_matches_with_context,
    selector_matches_with_selector_tree_context, selector_matches_with_tree_context,
    selector_specificity,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tests/color_missing.rs"]
mod color_missing_tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stylesheet {
    pub rules: Vec<CssRule>,
    pub font_faces: Vec<CssFontFace>,
    /// Layer names in normal cascade order, from lowest to highest priority.
    pub layers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CssFontFace {
    pub family: String,
    pub source: String,
    pub weight: Option<String>,
    pub style: Option<String>,
    pub stretch: Option<String>,
    pub weight_range: Option<(u16, u16)>,
    pub style_range: Option<(String, String)>,
    /// Source stylesheet, populated by `parse_stylesheet_file`.
    pub stylesheet_path: Option<std::path::PathBuf>,
}

fn parse_font_weight_range(value: &str) -> Option<(u16, u16)> {
    let mut parts = value.split_ascii_whitespace();
    let start = parts.next()?.parse::<u16>().ok()?;
    let end = parts.next()?.parse::<u16>().ok()?;
    if parts.next().is_some() || !(1..=1000).contains(&start) || !(1..=1000).contains(&end) {
        return None;
    }
    (start <= end).then_some((start, end))
}

fn parse_oblique_angle_range(value: &str) -> Option<(String, String)> {
    let mut parts = value.split_ascii_whitespace();
    if !parts.next()?.eq_ignore_ascii_case("oblique") {
        return None;
    }
    let parse_angle = |part: &str| {
        if part == "0" {
            return Some(0.0);
        }
        let part = part.to_ascii_lowercase();
        let (number, unit) = ["deg", "grad", "rad", "turn"]
            .into_iter()
            .find_map(|unit| part.strip_suffix(unit).map(|number| (number, unit)))?;
        let number = number.parse::<f32>().ok()?;
        let degrees = match unit {
            "deg" => number,
            "grad" => number * 0.9,
            "rad" => number.to_degrees(),
            "turn" => number * 360.0,
            _ => return None,
        };
        (degrees.is_finite() && (-90.0..=90.0).contains(&degrees)).then_some(degrees)
    };
    let start = parts.next()?;
    let end = parts.next()?;
    let start_degrees = parse_angle(start)?;
    let end_degrees = parse_angle(end)?;
    if parts.next().is_some() || start_degrees > end_degrees {
        return None;
    }
    Some((format!("{start_degrees}deg"), format!("{end_degrees}deg")))
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
    Ok(Stylesheet {
        rules,
        font_faces: state.font_faces,
        layers,
    })
}

#[derive(Default)]
struct LayerParseState {
    roots: Vec<LayerNode>,
    rules: Vec<(Option<String>, CssRule)>,
    anonymous_layer_count: usize,
    font_faces: Vec<CssFontFace>,
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
            let full_path = canonical_layer_path(&path);
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
        canonical_layer_path(&path)
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
                let path = current_layer
                    .iter()
                    .cloned()
                    .chain(std::iter::once(
                        path.rsplit('.').next().unwrap_or_default().to_owned(),
                    ))
                    .collect::<Vec<_>>();
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
        if prelude.eq_ignore_ascii_case("@font-face") && !statement {
            input.expect_curly_bracket_block()?;
            let declarations = input.parse_nested_block(parse_declaration_block)?;
            let family = declarations
                .iter()
                .find(|item| item.property == "font-family")
                .map(|item| {
                    crate::parse_css_string(&item.value)
                        .unwrap_or_else(|| item.value.trim().to_owned())
                })
                .filter(|family| !family.is_empty());
            let source = declarations
                .iter()
                .find(|item| item.property == "src")
                .map(|item| item.value.clone());
            if let (Some(family), Some(source)) = (family, source) {
                state.font_faces.push(CssFontFace {
                    family,
                    source,
                    weight: declarations
                        .iter()
                        .find(|item| item.property == "font-weight")
                        .map(|item| item.value.clone()),
                    style: declarations
                        .iter()
                        .find(|item| item.property == "font-style")
                        .map(|item| item.value.clone()),
                    stretch: declarations
                        .iter()
                        .find(|item| item.property == "font-stretch")
                        .map(|item| item.value.clone()),
                    weight_range: declarations
                        .iter()
                        .find(|item| item.property == "font-weight")
                        .and_then(|item| parse_font_weight_range(&item.value)),
                    style_range: declarations
                        .iter()
                        .find(|item| item.property == "font-style")
                        .and_then(|item| parse_oblique_angle_range(&item.value)),
                    stylesheet_path: None,
                });
            }
            continue;
        }
        if prelude.starts_with('@') {
            if !statement && input.expect_curly_bracket_block().is_ok() {
                input.parse_nested_block(|nested| {
                    while nested.next_including_whitespace().is_ok() {}
                    Ok(())
                })?;
            } else if !statement && !input.is_exhausted() {
                let _ = input.next_including_whitespace();
            }
            continue;
        }
        if statement || input.is_exhausted() {
            continue;
        }
        if input.expect_curly_bracket_block().is_err() {
            let _ = input.next_including_whitespace();
            continue;
        }
        let selectors = selector::split_selector_list(&prelude)
            .into_iter()
            .map(str::trim)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if selectors.is_empty()
            || selectors.iter().any(|selector| {
                selector.is_empty() || selector::selector_compounds(selector).is_none()
            })
        {
            input.parse_nested_block(|nested| {
                while nested.next_including_whitespace().is_ok() {}
                Ok(())
            })?;
            continue;
        }
        let declarations = input.parse_nested_block(parse_declaration_block)?;
        let layer_name = (!current_layer.is_empty()).then(|| canonical_layer_path(current_layer));
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
    let mut source = ParserInput::new(header);
    let mut input = Parser::new(&mut source);
    let mut names = Vec::new();
    while !input.is_exhausted() {
        input.skip_whitespace();
        if input.is_exhausted() {
            break;
        }
        let mut parts = vec![input.expect_ident().map_err(|_| ())?.to_string()];
        loop {
            input.skip_whitespace();
            if input.try_parse(|input| input.expect_delim('.')).is_err() {
                break;
            }
            input.skip_whitespace();
            parts.push(input.expect_ident().map_err(|_| ())?.to_string());
        }
        names.push(parts);
        input.skip_whitespace();
        if input.is_exhausted() {
            break;
        }
        input.expect_comma().map_err(|_| ())?;
        input.skip_whitespace();
        if input.is_exhausted() {
            return Err(());
        }
    }
    if names.is_empty() { Err(()) } else { Ok(names) }
}

pub(crate) fn canonical_layer_path(parts: &[impl AsRef<str>]) -> String {
    parts
        .iter()
        .map(|part| {
            let part = part.as_ref();
            if part.starts_with("<anonymous-layer-") {
                part.to_owned()
            } else {
                let mut serialized = String::new();
                serialize_identifier(part, &mut serialized)
                    .expect("writing a CSS identifier into a String cannot fail");
                serialized
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

pub(crate) fn split_layer_path(path: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut escaped = false;
    for (index, character) in path.char_indices() {
        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '.' {
            parts.push(path[start..index].to_owned());
            start = index + character.len_utf8();
        }
    }
    parts.push(path[start..].to_owned());
    parts
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
            let (value, valid) = serialize_declaration_value(declaration)?;
            Ok((property, value.trim().to_owned(), valid))
        });
        if let Ok((property, value, valid)) = parsed
            && valid
            && (!value.is_empty() || property.starts_with("--"))
        {
            declarations.push(CssDeclaration { property, value });
        }
    }
    Ok(declarations)
}

fn serialize_declaration_value<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<(String, bool), cssparser::ParseError<'i, ()>> {
    let mut value = String::new();
    let mut valid = true;
    while let Ok(token) = input.next_including_whitespace() {
        if matches!(token, Token::BadString(_) | Token::BadUrl(_)) {
            valid = false;
        }
        let block_end = match token {
            Token::Function(_) | Token::ParenthesisBlock => Some(')'),
            Token::SquareBracketBlock => Some(']'),
            Token::CurlyBracketBlock => Some('}'),
            _ => None,
        };
        token
            .to_css(&mut value)
            .expect("writing CSS tokens into a String cannot fail");
        if let Some(block_end) = block_end {
            let (contents, contents_valid) =
                input.parse_nested_block(serialize_declaration_value)?;
            value.push_str(&contents);
            value.push(block_end);
            valid &= contents_valid;
        }
    }
    Ok((value, valid))
}
