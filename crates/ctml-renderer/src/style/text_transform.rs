#[path = "text_transform_mappings.rs"]
mod mappings;

use super::{TextStyle, TextTransform, WhiteSpace};

pub(crate) fn transform_character(
    character: char,
    style: &TextStyle,
    capitalize_next: bool,
) -> String {
    let cased = match style.text_transform {
        TextTransform::Uppercase => character.to_uppercase().collect(),
        TextTransform::Lowercase => character.to_lowercase().collect(),
        TextTransform::Capitalize if capitalize_next => character.to_uppercase().collect(),
        TextTransform::None | TextTransform::Capitalize => character.to_string(),
    };
    let preserve_space = matches!(
        style.white_space,
        WhiteSpace::Pre | WhiteSpace::PreWrap | WhiteSpace::BreakSpaces
    );
    let mut transformed = String::new();
    for character in cased.chars() {
        let full_width = if style.text_transform_full_width && (character != ' ' || preserve_space)
        {
            mappings::full_width(character)
        } else {
            None
        };
        if let Some(full_width) = full_width {
            transformed.extend(full_width.chars().map(|character| {
                if style.text_transform_full_size_kana {
                    mappings::full_size_kana(character).unwrap_or(character)
                } else {
                    character
                }
            }));
        } else {
            transformed.push(if style.text_transform_full_size_kana {
                mappings::full_size_kana(character).unwrap_or(character)
            } else {
                character
            });
        }
    }
    transformed
}
