use std::{
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

use cssparser::{Parser, ParserInput, Token};

const MAX_LOCAL_RESOURCE_BYTES: u64 = 64 * 1024 * 1024;

/// A loaded file-backed CSS resource and its canonical build-dependency path.
pub struct LocalResource {
    /// Canonical path contained by the configured resource root.
    pub path: PathBuf,
    /// Binary resource contents.
    pub bytes: Vec<u8>,
}

/// Extracts the URL path from a CSS `url()` value, including quoted and escaped forms.
pub fn parse_css_url(value: &str) -> Option<String> {
    if !has_closed_url_function(value) {
        return None;
    }
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    let token = parser.next().ok()?;
    let url = match token {
        Token::UnquotedUrl(url) => url.to_string(),
        Token::Function(name) if name.eq_ignore_ascii_case("url") => parser
            .parse_nested_block(|input| -> Result<String, cssparser::ParseError<'_, ()>> {
                input.skip_whitespace();
                let url = input.expect_string()?.to_string();
                input.skip_whitespace();
                if !input.is_exhausted() {
                    return Err(input.new_custom_error(()));
                }
                Ok(url)
            })
            .ok()?,
        _ => return None,
    };
    parser.is_exhausted().then_some(url)
}

fn has_closed_url_function(value: &str) -> bool {
    let mut quote = None;
    let mut escaped = false;
    let mut depth = 0;
    let mut found_open = false;
    for (index, byte) in value.bytes().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' {
            escaped = true;
            continue;
        }
        if let Some(delimiter) = quote {
            if byte == delimiter {
                quote = None;
            }
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => {
                found_open = true;
                depth += 1;
            }
            b')' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    return found_open && value[index + 1..].trim().is_empty();
                }
            }
            _ => {}
        }
    }
    false
}

/// Parses and loads a local CSS `url()` value relative to its stylesheet.
pub fn load_css_resource(
    css_url: &str,
    stylesheet_path: impl AsRef<Path>,
    root: impl AsRef<Path>,
) -> Result<LocalResource, String> {
    let url_path = parse_css_url(css_url).ok_or_else(|| "invalid CSS url() value".to_owned())?;
    load_local_resource(&url_path, stylesheet_path, root)
}

/// Loads a local resource relative to a stylesheet, constrained to its configured root.
///
/// Network and data URLs, URL fragments and queries, path escapes, and files larger
/// than 64 MiB are rejected. The canonical path can be recorded as a build dependency.
pub fn load_local_resource(
    url_path: &str,
    stylesheet_path: impl AsRef<Path>,
    root: impl AsRef<Path>,
) -> Result<LocalResource, String> {
    let root = root
        .as_ref()
        .canonicalize()
        .map_err(|error| format!("cannot resolve resource root: {error}"))?;
    let stylesheet_path = stylesheet_path
        .as_ref()
        .canonicalize()
        .map_err(|error| format!("cannot resolve stylesheet path: {error}"))?;
    if !stylesheet_path.starts_with(&root) {
        return Err("stylesheet path is outside the configured resource root".to_owned());
    }
    let decoded_path = decode_url_path(url_path)?;
    let relative = validate_resource_path(&decoded_path)?;
    let resource_path = stylesheet_path
        .parent()
        .unwrap_or(&root)
        .join(relative)
        .canonicalize()
        .map_err(|error| format!("cannot resolve local resource: {error}"))?;
    if !resource_path.starts_with(&root) {
        return Err("resource path escapes the configured root".to_owned());
    }
    if !resource_path.is_file() {
        return Err("resource path does not name a regular file".to_owned());
    }
    let file = fs::File::open(&resource_path)
        .map_err(|error| format!("cannot open local resource: {error}"))?;
    let mut bytes = Vec::new();
    file.take(MAX_LOCAL_RESOURCE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read local resource: {error}"))?;
    if bytes.len() as u64 > MAX_LOCAL_RESOURCE_BYTES {
        return Err("local resource exceeds the 64 MiB size limit".to_owned());
    }
    Ok(LocalResource {
        path: resource_path,
        bytes,
    })
}

fn decode_url_path(url_path: &str) -> Result<String, String> {
    let bytes = url_path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let Some(hex) = bytes.get(index + 1..index + 3) else {
                return Err("resource URL contains an incomplete percent escape".to_owned());
            };
            let high = (hex[0] as char).to_digit(16);
            let low = (hex[1] as char).to_digit(16);
            let (Some(high), Some(low)) = (high, low) else {
                return Err("resource URL contains an invalid percent escape".to_owned());
            };
            decoded.push((high * 16 + low) as u8);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).map_err(|_| "resource URL is not valid UTF-8".to_owned())
}

fn validate_resource_path(path: &str) -> Result<PathBuf, String> {
    let has_scheme = path.split_once(':').is_some_and(|(scheme, _)| {
        let mut characters = scheme.chars();
        characters
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic())
            && characters.all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
            })
    });
    let path_buf = PathBuf::from(path);
    if path.is_empty()
        || path.contains(['?', '#', '\\', '\0'])
        || has_scheme
        || path_buf.is_absolute()
        || path.starts_with("//")
        || path_buf.components().any(|component| {
            !matches!(
                component,
                Component::Normal(_) | Component::CurDir | Component::ParentDir
            )
        })
    {
        return Err(
            "resource URL must be a relative local path without a query or fragment".to_owned(),
        );
    }
    Ok(path_buf)
}
