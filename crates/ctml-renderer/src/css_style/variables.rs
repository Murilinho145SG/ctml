use std::collections::{HashMap, HashSet};

const MAX_SUBSTITUTED_VALUE_LENGTH: usize = 1_000_000;
const MAX_VARIABLE_DEPTH: usize = 64;

pub(super) fn resolve_custom_properties(
    raw_properties: &HashMap<String, String>,
) -> HashMap<String, String> {
    let dependencies = raw_properties
        .iter()
        .map(|(name, value)| {
            let references = variable_references(value)
                .into_iter()
                .filter(|reference| raw_properties.contains_key(reference))
                .collect();
            (name.clone(), references)
        })
        .collect::<HashMap<String, Vec<String>>>();
    let cyclic = cyclic_properties(&dependencies);
    let mut resolved = HashMap::new();
    for name in raw_properties.keys() {
        let mut stack = Vec::new();
        if let Some(value) = resolve_variable(name, raw_properties, &cyclic, &mut stack) {
            resolved.insert(name.clone(), value);
        }
    }
    resolved
}

pub(super) fn substitute_vars(
    value: &str,
    custom_properties: &HashMap<String, String>,
) -> Option<String> {
    substitute_value(value, custom_properties, &HashSet::new(), &mut Vec::new())
}

pub(super) fn contains_var_function(value: &str) -> bool {
    next_var_call(value, 0).is_some()
}

fn resolve_variable(
    name: &str,
    raw_properties: &HashMap<String, String>,
    cyclic: &HashSet<String>,
    stack: &mut Vec<String>,
) -> Option<String> {
    if cyclic.contains(name)
        || stack.len() >= MAX_VARIABLE_DEPTH
        || stack.iter().any(|item| item == name)
    {
        return None;
    }
    let value = raw_properties.get(name)?;
    stack.push(name.to_owned());
    let resolved = substitute_value(value, raw_properties, cyclic, stack);
    stack.pop();
    resolved
}

fn substitute_value(
    value: &str,
    custom_properties: &HashMap<String, String>,
    cyclic: &HashSet<String>,
    stack: &mut Vec<String>,
) -> Option<String> {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0;
    while cursor < value.len() {
        if let Some((start, end, name, fallback)) = next_var_call(value, cursor) {
            output.push_str(&value[cursor..start]);
            let replacement =
                resolve_variable(name, custom_properties, cyclic, stack).or_else(|| {
                    fallback.and_then(|fallback| {
                        substitute_value(fallback, custom_properties, cyclic, stack)
                    })
                })?;
            output.push_str(&replacement);
            if output.len() > MAX_SUBSTITUTED_VALUE_LENGTH {
                return None;
            }
            cursor = end;
        } else {
            let character = value[cursor..].chars().next()?;
            output.push(character);
            cursor += character.len_utf8();
            if output.len() > MAX_SUBSTITUTED_VALUE_LENGTH {
                return None;
            }
        }
    }
    Some(output)
}

fn variable_references(value: &str) -> Vec<String> {
    let mut references = Vec::new();
    let mut cursor = 0;
    while let Some((start, end, name, fallback)) = next_var_call(value, cursor) {
        references.push(name.to_owned());
        if let Some(fallback) = fallback {
            references.extend(variable_references(fallback));
        }
        cursor = end;
        if cursor <= start {
            break;
        }
    }
    references
}

fn next_var_call(value: &str, cursor: usize) -> Option<(usize, usize, &str, Option<&str>)> {
    let mut quote = None;
    let mut escaped = false;
    for character in value[..cursor].chars() {
        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if quote == Some(character) {
            quote = None;
        } else if quote.is_none() && matches!(character, '\'' | '"') {
            quote = Some(character);
        }
    }
    let mut index = cursor;
    while index < value.len() {
        let character = value[index..].chars().next()?;
        if escaped {
            escaped = false;
            index += character.len_utf8();
            continue;
        }
        if character == '\\' {
            escaped = true;
            index += character.len_utf8();
            continue;
        }
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            }
            index += character.len_utf8();
            continue;
        }
        if character == '\'' || character == '"' {
            quote = Some(character);
            index += character.len_utf8();
            continue;
        }
        if value[index..]
            .get(..4)
            .is_some_and(|token| token.eq_ignore_ascii_case("var("))
            && (index == 0
                || !value[..index]
                    .chars()
                    .next_back()
                    .is_some_and(is_identifier_character))
        {
            let open = index + 3;
            let close = matching_parenthesis(value, open)?;
            let contents = &value[open + 1..close];
            let comma = top_level_comma(contents);
            let name = contents[..comma.unwrap_or(contents.len())].trim();
            if !name.starts_with("--") || name.len() == 2 {
                return None;
            }
            let fallback = comma.map(|comma| contents[comma + 1..].trim());
            return Some((index, close + 1, name, fallback));
        }
        index += character.len_utf8();
    }
    None
}

fn matching_parenthesis(value: &str, open: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut index = open + 1;
    while index < value.len() {
        let character = value[index..].chars().next()?;
        if character == '\'' || character == '"' {
            index = skip_quoted(value, index, character)?;
            continue;
        }
        if character == '\\' {
            index += character.len_utf8();
            if index < value.len() {
                index += value[index..].chars().next()?.len_utf8();
            }
            continue;
        }
        match character {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += character.len_utf8();
    }
    None
}

fn top_level_comma(value: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = 0;
    while index < value.len() {
        let character = value[index..].chars().next()?;
        if character == '\'' || character == '"' {
            index = skip_quoted(value, index, character)?;
            continue;
        }
        if character == '\\' {
            index += character.len_utf8();
            if index < value.len() {
                index += value[index..].chars().next()?.len_utf8();
            }
            continue;
        }
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return Some(index),
            _ => {}
        }
        index += character.len_utf8();
    }
    None
}

fn skip_quoted(value: &str, start: usize, quote: char) -> Option<usize> {
    let mut index = start + quote.len_utf8();
    while index < value.len() {
        let character = value[index..].chars().next()?;
        if character == '\\' {
            index += character.len_utf8();
            if index < value.len() {
                index += value[index..].chars().next()?.len_utf8();
            }
        } else {
            index += character.len_utf8();
            if character == quote {
                return Some(index);
            }
        }
    }
    Some(value.len())
}

fn is_identifier_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_' || character == '-' || character == '\\'
}

fn cyclic_properties(dependencies: &HashMap<String, Vec<String>>) -> HashSet<String> {
    let mut states = HashMap::new();
    let mut stack = Vec::new();
    let mut cyclic = HashSet::new();
    for name in dependencies.keys() {
        visit_property(name, dependencies, &mut states, &mut stack, &mut cyclic);
    }
    cyclic
}

fn visit_property(
    name: &str,
    dependencies: &HashMap<String, Vec<String>>,
    states: &mut HashMap<String, bool>,
    stack: &mut Vec<String>,
    cyclic: &mut HashSet<String>,
) {
    if stack.len() >= MAX_VARIABLE_DEPTH {
        states.insert(name.to_owned(), false);
        cyclic.insert(name.to_owned());
        return;
    }
    states.insert(name.to_owned(), true);
    stack.push(name.to_owned());
    for dependency in dependencies.get(name).into_iter().flatten() {
        match states.get(dependency) {
            Some(true) => {
                if let Some(start) = stack.iter().position(|item| item == dependency) {
                    cyclic.extend(stack[start..].iter().cloned());
                }
            }
            Some(false) => {}
            None => visit_property(dependency, dependencies, states, stack, cyclic),
        }
    }
    stack.pop();
    states.insert(name.to_owned(), false);
}

#[cfg(test)]
mod tests {
    use super::{resolve_custom_properties, substitute_vars};
    use std::collections::HashMap;

    #[test]
    fn substitutes_nested_fallbacks_and_resolves_custom_property_dependencies() {
        let properties = HashMap::from([
            ("--base".to_owned(), "#336699".to_owned()),
            ("--accent".to_owned(), "var(--base)".to_owned()),
            (
                "--label".to_owned(),
                "var(--missing, var(--accent))".to_owned(),
            ),
        ]);
        let properties = resolve_custom_properties(&properties);

        assert_eq!(properties["--accent"], "#336699");
        assert_eq!(
            substitute_vars("var(--label)", &properties).as_deref(),
            Some("#336699")
        );
        assert_eq!(
            substitute_vars("var(--missing, rgb(1, 2, 3))", &properties).as_deref(),
            Some("rgb(1, 2, 3)")
        );
    }

    #[test]
    fn invalidates_dependency_cycles_and_uses_fallbacks() {
        let properties = HashMap::from([
            ("--first".to_owned(), "var(--second, red)".to_owned()),
            ("--second".to_owned(), "var(--first, blue)".to_owned()),
            ("--self".to_owned(), "var(--self)".to_owned()),
        ]);
        let properties = resolve_custom_properties(&properties);

        assert!(!properties.contains_key("--first"));
        assert!(!properties.contains_key("--second"));
        assert!(!properties.contains_key("--self"));
        assert_eq!(
            substitute_vars("var(--first, green)", &properties).as_deref(),
            Some("green")
        );
    }

    #[test]
    fn ignores_var_text_inside_strings_and_identifiers() {
        let properties = HashMap::from([("--value".to_owned(), "blue".to_owned())]);
        assert_eq!(
            substitute_vars("content: \"var(--value)\"; myvar(--value)", &properties).as_deref(),
            Some("content: \"var(--value)\"; myvar(--value)")
        );
    }
}
