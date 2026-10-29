use cosmic_text::FontSystem;

pub(super) fn resolve_family_list(family_list: &str, font_system: &FontSystem) -> String {
    let candidates = split_family_list(family_list);
    let first = candidates
        .first()
        .cloned()
        .unwrap_or_else(|| "sans-serif".to_owned());

    for family in candidates {
        let lower = family.to_ascii_lowercase();
        if is_generic_family(&lower) {
            return lower;
        }
        if font_system.db().faces().any(|face| {
            face.families
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case(&family))
        }) {
            return family;
        }
        #[cfg(target_os = "linux")]
        if linux_font_alias(&lower)
            && let Some(alias) = crate::fontconfig::matched_family(&family)
        {
            return alias;
        }
    }

    #[cfg(target_os = "linux")]
    if let Some(alias) = crate::fontconfig::matched_family("Times New Roman") {
        return alias;
    }

    first
}

fn split_family_list(value: &str) -> Vec<String> {
    let mut families = Vec::new();
    let mut start = 0;
    let mut quote = None;
    let mut escaped = false;

    for (index, character) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
        } else if character == ',' {
            push_family(&mut families, &value[start..index]);
            start = index + character.len_utf8();
        }
    }
    push_family(&mut families, &value[start..]);
    families
}

fn push_family(families: &mut Vec<String>, family: &str) {
    let family = family.trim().trim_matches(['\'', '"']).trim();
    if !family.is_empty() {
        families.push(family.to_owned());
    }
}

fn is_generic_family(family: &str) -> bool {
    matches!(
        family,
        "serif" | "sans-serif" | "monospace" | "cursive" | "fantasy"
    )
}

#[cfg(target_os = "linux")]
fn linux_font_alias(family: &str) -> bool {
    matches!(
        family,
        "arial" | "helvetica" | "times new roman" | "times" | "courier new" | "courier"
    )
}

#[cfg(test)]
mod tests {
    use cosmic_text::FontSystem;

    use super::{resolve_family_list, split_family_list};

    #[test]
    fn splits_quoted_and_unquoted_family_names() {
        assert_eq!(
            split_family_list("\"Missing, Display\", serif, Arial"),
            ["Missing, Display", "serif", "Arial"]
        );
    }

    #[test]
    fn chooses_a_generic_family_after_an_unavailable_name() {
        let font_system = FontSystem::new();
        assert_eq!(
            resolve_family_list("\"Missing Font\", serif", &font_system),
            "serif"
        );
    }

    #[test]
    fn chooses_the_first_installed_named_family() {
        let font_system = FontSystem::new();
        let Some(installed_family) = font_system
            .db()
            .faces()
            .find_map(|face| face.families.first().map(|(name, _)| name.clone()))
        else {
            return;
        };
        let family_list = format!("\"Missing Font\", \"{installed_family}\", serif");

        assert_eq!(
            resolve_family_list(&family_list, &font_system),
            installed_family
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn uses_chromiums_standard_serif_for_an_unavailable_named_family() {
        let font_system = FontSystem::new();
        let expected = crate::fontconfig::matched_family("Times New Roman")
            .expect("Fontconfig should resolve Chromium's standard serif family");

        assert_eq!(resolve_family_list("Inter", &font_system), expected);
    }
}
