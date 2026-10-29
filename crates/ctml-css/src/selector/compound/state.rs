use crate::selector::{ElementIdentity, list::split_selector_list};

pub(super) fn link_pseudo_matches(
    name: &str,
    element: &ElementIdentity,
    attributes: &[(String, Option<String>)],
) -> bool {
    if name == "visited" {
        return false;
    }
    matches!(name, "link" | "any-link")
        && element.tag.eq_ignore_ascii_case("a")
        && attributes
            .iter()
            .any(|(attribute, _)| attribute.eq_ignore_ascii_case("href"))
}

pub(super) fn language_pseudo_matches(
    argument: &str,
    attributes: &[(String, Option<String>)],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
) -> bool {
    let language = inherited_attribute("lang", attributes, ancestor_attributes)
        .or_else(|| inherited_attribute("xml:lang", attributes, ancestor_attributes));
    let Some(Some(language)) = language else {
        return false;
    };
    split_selector_list(argument).into_iter().any(|range| {
        let range = unquote(range.trim());
        if range == "*" {
            return !language.is_empty();
        }
        !range.is_empty()
            && (language.eq_ignore_ascii_case(range)
                || language
                    .get(..range.len())
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(range))
                    && language.as_bytes().get(range.len()) == Some(&b'-'))
    })
}

pub(super) fn direction_pseudo_matches(
    argument: &str,
    attributes: &[(String, Option<String>)],
    ancestor_attributes: &[Vec<(String, Option<String>)>],
) -> bool {
    let requested = unquote(argument.trim());
    if !requested.eq_ignore_ascii_case("ltr") && !requested.eq_ignore_ascii_case("rtl") {
        return false;
    }
    match inherited_attribute("dir", attributes, ancestor_attributes) {
        Some(Some(direction))
            if direction.eq_ignore_ascii_case("ltr") || direction.eq_ignore_ascii_case("rtl") =>
        {
            direction.eq_ignore_ascii_case(requested)
        }
        Some(_) => false,
        None => requested.eq_ignore_ascii_case("ltr"),
    }
}

fn inherited_attribute<'a>(
    name: &str,
    attributes: &'a [(String, Option<String>)],
    ancestor_attributes: &'a [Vec<(String, Option<String>)>],
) -> Option<Option<&'a str>> {
    attributes
        .iter()
        .find(|(attribute, _)| attribute.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_deref())
        .or_else(|| {
            ancestor_attributes.iter().rev().find_map(|attributes| {
                attributes
                    .iter()
                    .find(|(attribute, _)| attribute.eq_ignore_ascii_case(name))
                    .map(|(_, value)| value.as_deref())
            })
        })
}

fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
}
