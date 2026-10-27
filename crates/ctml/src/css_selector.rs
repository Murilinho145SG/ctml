#[derive(Clone)]
pub(super) struct ElementIdentity {
    pub(super) tag: String,
    pub(super) class: Option<String>,
    pub(super) id: Option<String>,
}

pub(super) fn selector_matches(
    selector: &str,
    current: &ElementIdentity,
    ancestors: &[ElementIdentity],
) -> bool {
    let Some(compounds) = selector_compounds(selector) else {
        return false;
    };
    let Some((last, _)) = compounds.last() else {
        return false;
    };
    if !compound_matches(last, current) {
        return false;
    }
    let mut ancestor_index = ancestors.len();
    for index in (0..compounds.len() - 1).rev() {
        let (compound, _) = &compounds[index];
        match compounds[index + 1].1 {
            SelectorRelation::Child => {
                let Some(parent_index) = ancestor_index.checked_sub(1) else {
                    return false;
                };
                if !compound_matches(compound, &ancestors[parent_index]) {
                    return false;
                }
                ancestor_index = parent_index;
            }
            SelectorRelation::Descendant => {
                let Some(found) = ancestors[..ancestor_index]
                    .iter()
                    .rposition(|ancestor| compound_matches(compound, ancestor))
                else {
                    return false;
                };
                ancestor_index = found;
            }
        }
    }
    true
}

#[derive(Clone, Copy)]
enum SelectorRelation {
    Descendant,
    Child,
}

fn selector_compounds(selector: &str) -> Option<Vec<(String, SelectorRelation)>> {
    let mut compounds = Vec::new();
    let mut start = None;
    let mut relation = SelectorRelation::Descendant;
    for (index, byte) in selector.bytes().enumerate() {
        if byte.is_ascii_whitespace() || byte == b'>' {
            if let Some(compound_start) = start.take() {
                compounds.push((selector[compound_start..index].to_owned(), relation));
                relation = SelectorRelation::Descendant;
            } else if byte == b'>'
                && (compounds.is_empty() || matches!(relation, SelectorRelation::Child))
            {
                return None;
            }
            if byte == b'>' {
                relation = SelectorRelation::Child;
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(compound_start) = start {
        compounds.push((selector[compound_start..].to_owned(), relation));
        Some(compounds)
    } else {
        None
    }
}

fn compound_matches(compound: &str, element: &ElementIdentity) -> bool {
    if compound == "*" {
        return true;
    }
    let mut cursor = 0;
    let bytes = compound.as_bytes();
    if bytes.first() == Some(&b'*') {
        cursor = 1;
    } else if bytes
        .first()
        .is_some_and(|byte| *byte != b'.' && *byte != b'#')
    {
        let start = cursor;
        while cursor < bytes.len() && bytes[cursor] != b'.' && bytes[cursor] != b'#' {
            cursor += 1;
        }
        if !compound[start..cursor].eq_ignore_ascii_case(&element.tag) {
            return false;
        }
    }
    while cursor < bytes.len() {
        let marker = bytes[cursor];
        if marker != b'.' && marker != b'#' {
            return false;
        }
        cursor += 1;
        let start = cursor;
        while cursor < bytes.len() && bytes[cursor] != b'.' && bytes[cursor] != b'#' {
            cursor += 1;
        }
        if start == cursor {
            return false;
        }
        let name = &compound[start..cursor];
        if marker == b'#' && element.id.as_deref() != Some(name) {
            return false;
        }
        if marker == b'.'
            && !element
                .class
                .as_deref()
                .is_some_and(|classes| classes.split_ascii_whitespace().any(|class| class == name))
        {
            return false;
        }
    }
    true
}

pub(super) fn selector_specificity(selector: &str) -> (usize, usize, usize) {
    let Some(compounds) = selector_compounds(selector) else {
        return (0, 0, 0);
    };
    let ids = compounds
        .iter()
        .flat_map(|(compound, _)| compound.bytes())
        .filter(|byte| *byte == b'#')
        .count();
    let classes = compounds
        .iter()
        .flat_map(|(compound, _)| compound.bytes())
        .filter(|byte| *byte == b'.')
        .count();
    let tags = compounds
        .iter()
        .filter(|(compound, _)| !compound.starts_with(['.', '#', '*']))
        .count();
    (ids, classes, tags)
}
