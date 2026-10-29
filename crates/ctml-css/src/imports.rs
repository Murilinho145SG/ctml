use std::{
    fs,
    ops::Range,
    path::{Component, Path, PathBuf},
};

use crate::{Stylesheet, comments::strip_utf8_bom_and_comments, parse_stylesheet};

/// Loads a local stylesheet and expands quoted, relative `@import` rules in place.
///
/// Imports must stay inside `root`; network URLs, media qualifiers, and escaped
/// import paths are rejected. Returned dependencies are canonical paths in the
/// order in which the loader first reads them.
pub fn parse_stylesheet_file(
    path: impl AsRef<Path>,
    root: impl AsRef<Path>,
) -> Result<(Stylesheet, Vec<PathBuf>), String> {
    let root = root
        .as_ref()
        .canonicalize()
        .map_err(|error| format!("cannot resolve stylesheet root: {error}"))?;
    let mut dependencies = Vec::new();
    let mut stack = Vec::new();
    let source = expand_file(path.as_ref(), &root, &mut stack, &mut dependencies)?;
    let stylesheet = parse_stylesheet(&source)
        .map_err(|error| format!("{}: {error}", path.as_ref().display()))?;
    Ok((stylesheet, dependencies))
}

fn expand_file(
    path: &Path,
    root: &Path,
    stack: &mut Vec<PathBuf>,
    dependencies: &mut Vec<PathBuf>,
) -> Result<String, String> {
    let path = path
        .canonicalize()
        .map_err(|error| format!("cannot read stylesheet {}: {error}", path.display()))?;
    if !path.starts_with(root) {
        return Err(format!(
            "stylesheet import {} escapes its configured root",
            path.display()
        ));
    }
    if stack.contains(&path) {
        let cycle = stack
            .iter()
            .chain(std::iter::once(&path))
            .map(|item| item.display().to_string())
            .collect::<Vec<_>>()
            .join(" -> ");
        return Err(format!("stylesheet import cycle: {cycle}"));
    }

    let source = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read stylesheet {}: {error}", path.display()))?;
    let source = strip_utf8_bom_and_comments(&source)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if !dependencies.contains(&path) {
        dependencies.push(path.clone());
    }
    stack.push(path.clone());

    let imports = find_imports(&source).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut expanded = String::with_capacity(source.len());
    let mut copied_until = 0;
    for import in imports {
        expanded.push_str(&source[copied_until..import.range.start]);
        let relative = validate_import_path(&import.path)?;
        let imported_path = path.parent().unwrap_or(root).join(relative);
        expanded.push_str(&expand_file(&imported_path, root, stack, dependencies)?);
        expanded.push('\n');
        copied_until = import.range.end;
    }
    expanded.push_str(&source[copied_until..]);
    stack.pop();
    Ok(expanded)
}

struct Import {
    range: Range<usize>,
    path: String,
}

fn find_imports(source: &str) -> Result<Vec<Import>, String> {
    let mut imports = Vec::new();
    let mut index = 0;
    let mut block_depth = 0usize;
    let mut saw_rule = false;
    while index < source.len() {
        if source[index..].starts_with("/*") {
            index = skip_comment(source, index)?;
        } else if let Some(end) = skip_string(source, index)? {
            index = end;
        } else if block_depth == 0
            && source[index..]
                .get(.."@import".len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("@import"))
            && source[index + "@import".len()..]
                .chars()
                .next()
                .is_none_or(|character| {
                    character.is_whitespace()
                        || matches!(character, '\'' | '"')
                        || source[index + "@import".len()..]
                            .get(..4)
                            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("url("))
                })
        {
            if saw_rule {
                return Err("@import rules must precede qualified rules".to_owned());
            }
            let end = find_import_end(source, index + "@import".len())?;
            let path = parse_import_path(&source[index + "@import".len()..end])?;
            imports.push(Import {
                range: index..end + 1,
                path,
            });
            index = end + 1;
        } else {
            let character = source[index..]
                .chars()
                .next()
                .expect("valid character boundary");
            match character {
                '{' => block_depth += 1,
                '}' => block_depth = block_depth.saturating_sub(1),
                character if block_depth == 0 && !character.is_whitespace() => saw_rule = true,
                _ => {}
            }
            index += character.len_utf8();
        }
    }
    Ok(imports)
}

fn skip_comment(source: &str, start: usize) -> Result<usize, String> {
    source[start + 2..]
        .find("*/")
        .map(|end| start + 2 + end + 2)
        .ok_or_else(|| "unterminated CSS comment".to_owned())
}

fn skip_string(source: &str, start: usize) -> Result<Option<usize>, String> {
    let Some(quote @ ('\'' | '"')) = source[start..].chars().next() else {
        return Ok(None);
    };
    let mut index = start + quote.len_utf8();
    while index < source.len() {
        let character = source[index..]
            .chars()
            .next()
            .expect("valid character boundary");
        if character == '\\' {
            index += character.len_utf8();
            if index < source.len() {
                index += source[index..]
                    .chars()
                    .next()
                    .expect("valid character boundary")
                    .len_utf8();
            }
        } else if character == quote {
            return Ok(Some(index + character.len_utf8()));
        } else {
            index += character.len_utf8();
        }
    }
    Err("unterminated CSS string".to_owned())
}

fn find_import_end(source: &str, start: usize) -> Result<usize, String> {
    let mut index = start;
    while index < source.len() {
        if source[index..].starts_with("/*") {
            index = skip_comment(source, index)?;
        } else if let Some(end) = skip_string(source, index)? {
            index = end;
        } else {
            let character = source[index..]
                .chars()
                .next()
                .expect("valid character boundary");
            if character == ';' {
                return Ok(index);
            }
            index += character.len_utf8();
        }
    }
    Err("@import rule is missing a semicolon".to_owned())
}

fn parse_import_path(prelude: &str) -> Result<String, String> {
    let prelude = prelude.trim();
    let (prelude, is_url) = if prelude
        .get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("url("))
        && prelude.ends_with(')')
    {
        (prelude[4..prelude.len() - 1].trim(), true)
    } else {
        (prelude, false)
    };
    let (prelude, quoted) = prelude
        .strip_prefix('"')
        .and_then(|path| path.strip_suffix('"'))
        .map(|path| (path, true))
        .or_else(|| {
            prelude
                .strip_prefix('\'')
                .and_then(|path| path.strip_suffix('\''))
                .map(|path| (path, true))
        })
        .unwrap_or((prelude, false));
    if !is_url && !quoted {
        return Err("@import requires a quoted local path or url()".to_owned());
    }
    if !quoted && prelude.contains(char::is_whitespace) {
        return Err("@import path cannot contain unquoted whitespace".to_owned());
    }
    if prelude.contains(['"', '\'', '(', ')']) {
        return Err("@import requires one local path without media qualifiers".to_owned());
    }
    if prelude.contains('\\') {
        return Err("escaped @import paths are not supported".to_owned());
    }
    Ok(prelude.to_owned())
}

fn validate_import_path(path: &str) -> Result<PathBuf, String> {
    let has_scheme = path.split_once(':').is_some_and(|(scheme, _)| {
        let mut characters = scheme.chars();
        characters
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic())
            && characters.all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
            })
    });
    let path = PathBuf::from(path);
    if path.as_os_str().is_empty()
        || has_scheme
        || path.is_absolute()
        || path.starts_with("//")
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err("@import path must be relative and stay inside the stylesheet root".to_owned());
    }
    Ok(path)
}
