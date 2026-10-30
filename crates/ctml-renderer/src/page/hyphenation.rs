use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

use hyphenation::{Hyphenator, Language, Load, Standard};

use crate::style::{Hyphens, TextRun};

static DICTIONARIES: OnceLock<Mutex<HashMap<Language, Option<Arc<Standard>>>>> = OnceLock::new();

pub(super) fn apply_auto_hyphenation(runs: &mut [TextRun]) {
    for run in runs {
        if run.style.hyphens != Hyphens::Auto {
            continue;
        }
        let Some(language) = run.style.language.as_deref().and_then(resolve_language) else {
            continue;
        };
        let Some(dictionary) = dictionary(language) else {
            continue;
        };
        run.text = hyphenate_text(&run.text, &dictionary);
    }
}

fn resolve_language(language: &str) -> Option<Language> {
    let normalized = language.trim().replace('_', "-").to_ascii_lowercase();
    Language::try_from_code(&normalized).or_else(|| {
        let primary = normalized.split('-').next()?;
        match primary {
            "de" => Some(Language::German1996),
            "en" => Some(Language::EnglishUS),
            _ => Language::try_from_code(primary),
        }
    })
}

fn dictionary(language: Language) -> Option<Arc<Standard>> {
    let dictionaries = DICTIONARIES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut dictionaries = dictionaries.lock().ok()?;
    Some(
        dictionaries
            .entry(language)
            .or_insert_with(|| Standard::from_embedded(language).ok().map(Arc::new))
            .clone()?,
    )
}

fn hyphenate_text(text: &str, dictionary: &Standard) -> String {
    let mut output = String::with_capacity(text.len());
    let mut word_start = None;
    let mut has_soft_hyphen = false;

    for (index, character) in text.char_indices() {
        if character.is_alphabetic() || character == '\u{00ad}' {
            word_start.get_or_insert(index);
            has_soft_hyphen |= character == '\u{00ad}';
            continue;
        }

        if let Some(start) = word_start.take() {
            append_word(
                &mut output,
                &text[start..index],
                dictionary,
                has_soft_hyphen,
            );
            has_soft_hyphen = false;
        }
        output.push(character);
    }

    if let Some(start) = word_start {
        append_word(&mut output, &text[start..], dictionary, has_soft_hyphen);
    }
    output
}

fn append_word(output: &mut String, word: &str, dictionary: &Standard, has_soft_hyphen: bool) {
    if has_soft_hyphen {
        output.push_str(word);
        return;
    }

    let breaks = dictionary.hyphenate(word).breaks;
    let mut previous = 0;
    for index in breaks
        .into_iter()
        .filter(|index| word.is_char_boundary(*index))
    {
        output.push_str(&word[previous..index]);
        output.push('\u{00ad}');
        previous = index;
    }
    output.push_str(&word[previous..]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_dictionary_breaks_only_for_resolvable_languages() {
        let dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
        assert_eq!(
            hyphenate_text("hyphenation", &dictionary),
            "hy\u{00ad}phen\u{00ad}a\u{00ad}tion"
        );
        assert_eq!(
            hyphenate_text("hello world", &dictionary)
                .matches('\u{00ad}')
                .count(),
            0
        );
    }

    #[test]
    fn preserves_manual_breaks_without_adding_dictionary_breaks() {
        let dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
        assert_eq!(
            hyphenate_text("hy\u{00ad}phenation", &dictionary),
            "hy\u{00ad}phenation"
        );
    }

    #[test]
    fn resolves_language_tags_and_common_language_defaults() {
        assert_eq!(resolve_language("en-US"), Some(Language::EnglishUS));
        assert_eq!(resolve_language("pt-BR"), Some(Language::Portuguese));
        assert_eq!(resolve_language("zz-ZZ"), None);
    }
}
