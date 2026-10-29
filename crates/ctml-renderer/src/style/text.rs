use super::{TextRun, TextStyle, TextTransform};

pub fn append_text(
    text: &str,
    style: &TextStyle,
    element_path: &[usize],
    output: &mut Vec<TextRun>,
    last_was_space: &mut bool,
    capitalize_next: &mut bool,
) {
    for character in text.chars() {
        if character.is_whitespace() {
            if !*last_was_space {
                push_character(' ', style, element_path, output);
            }
            *last_was_space = true;
            *capitalize_next = true;
        } else {
            match style.text_transform {
                TextTransform::Uppercase => {
                    for transformed in character.to_uppercase() {
                        push_variant_character(transformed, character, style, element_path, output);
                    }
                }
                TextTransform::Lowercase => {
                    for transformed in character.to_lowercase() {
                        push_variant_character(transformed, character, style, element_path, output);
                    }
                }
                TextTransform::Capitalize if *capitalize_next => {
                    for transformed in character.to_uppercase() {
                        push_variant_character(transformed, character, style, element_path, output);
                    }
                }
                _ => push_variant_character(character, character, style, element_path, output),
            }
            *last_was_space = false;
            *capitalize_next = !character.is_alphanumeric();
        }
    }
}

fn push_variant_character(
    character: char,
    source_character: char,
    style: &TextStyle,
    element_path: &[usize],
    output: &mut Vec<TextRun>,
) {
    push_character_with_source(character, source_character, style, element_path, output);
}

pub fn push_line_break(
    output: &mut Vec<TextRun>,
    style: &TextStyle,
    element_path: &[usize],
    last_was_space: &mut bool,
) {
    if let Some(last) = output.last_mut() {
        while last.text.ends_with(' ') {
            last.text.pop();
        }
        if !last.text.ends_with('\n') {
            push_character('\n', style, element_path, output);
        }
    }
    *last_was_space = true;
}

pub fn push_character(
    character: char,
    style: &TextStyle,
    element_path: &[usize],
    output: &mut Vec<TextRun>,
) {
    push_character_with_source(character, character, style, element_path, output);
}

fn push_character_with_source(
    character: char,
    source_character: char,
    style: &TextStyle,
    element_path: &[usize],
    output: &mut Vec<TextRun>,
) {
    if let Some(last) = output.last_mut()
        && last.style == *style
        && last.element_path == element_path
    {
        last.text.push(character);
        if last.source_first_non_whitespace.is_none() && !source_character.is_whitespace() {
            last.source_first_non_whitespace = Some(source_character);
        }
    } else {
        output.push(TextRun {
            text: character.to_string(),
            source_first_non_whitespace: (!source_character.is_whitespace())
                .then_some(source_character),
            style: style.clone(),
            element_path: element_path.to_vec(),
            selection_background: None,
            selection_foreground: None,
            first_line_block: None,
            first_line_foreground: None,
            first_line_background: None,
            first_line_decoration: None,
            first_line_font_style: None,
            first_letter_block: None,
            first_letter_foreground: None,
            first_letter_background: None,
            first_letter_decoration: None,
            first_letter_font_style: None,
        });
    }
}
