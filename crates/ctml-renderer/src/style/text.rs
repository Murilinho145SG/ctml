use super::{TextRun, TextStyle, TextTransform, WhiteSpace};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

pub fn append_text(
    text: &str,
    style: &TextStyle,
    element_path: &[usize],
    output: &mut Vec<TextRun>,
    last_was_space: &mut bool,
    capitalize_next: &mut bool,
) {
    let preserve_spaces = matches!(
        style.white_space,
        WhiteSpace::Pre | WhiteSpace::PreWrap | WhiteSpace::BreakSpaces
    );
    let preserve_breaks = matches!(
        style.white_space,
        WhiteSpace::Pre | WhiteSpace::PreWrap | WhiteSpace::PreLine | WhiteSpace::BreakSpaces
    );
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        if is_segment_break(character) && preserve_breaks {
            if character == '\r' && characters.peek() == Some(&'\n') {
                characters.next();
            }
            if style.white_space == WhiteSpace::PreLine {
                trim_trailing_collapsible(output);
            }
            push_character('\n', style, element_path, output);
            *last_was_space = true;
            *capitalize_next = true;
        } else if character.is_whitespace() {
            if preserve_spaces {
                push_preserved_space(character, style, element_path, output);
            } else if !*last_was_space {
                push_space(style, element_path, output);
            }
            *last_was_space = true;
            *capitalize_next = true;
        } else {
            push_transformed_character(character, style, element_path, output, *capitalize_next);
            *last_was_space = false;
            *capitalize_next = !character.is_alphanumeric();
        }
    }
}

fn is_segment_break(character: char) -> bool {
    matches!(character, '\n' | '\r' | '\u{000c}')
}

fn trim_trailing_collapsible(output: &mut [TextRun]) {
    if let Some(last) = output.last_mut() {
        if matches!(
            last.style.white_space,
            WhiteSpace::Pre | WhiteSpace::PreWrap | WhiteSpace::BreakSpaces
        ) {
            return;
        }
        while last.text.ends_with(' ') {
            last.text.pop();
        }
    }
}

fn push_transformed_character(
    character: char,
    style: &TextStyle,
    element_path: &[usize],
    output: &mut Vec<TextRun>,
    capitalize_next: bool,
) {
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
        TextTransform::Capitalize if capitalize_next => {
            for transformed in character.to_uppercase() {
                push_variant_character(transformed, character, style, element_path, output);
            }
        }
        _ => push_variant_character(character, character, style, element_path, output),
    }
}

fn push_space(style: &TextStyle, element_path: &[usize], output: &mut Vec<TextRun>) {
    push_space_character(' ', style, element_path, output);
}

fn push_preserved_space(
    character: char,
    style: &TextStyle,
    element_path: &[usize],
    output: &mut Vec<TextRun>,
) {
    push_space_character(character, style, element_path, output);
}

fn push_space_character(
    character: char,
    style: &TextStyle,
    element_path: &[usize],
    output: &mut Vec<TextRun>,
) {
    let mut space_style = style.clone();
    space_style.word_spacing_space = true;
    let word_spacing = style.word_spacing_px
        + style.word_spacing_percent.unwrap_or_default() * style.font_size / 100.0;
    if word_spacing != 0.0 {
        space_style.letter_spacing = if style.letter_spacing_px {
            style.letter_spacing / style.font_size.max(1.0)
        } else {
            style.letter_spacing
        } + word_spacing / style.font_size.max(1.0);
        space_style.letter_spacing_px = false;
    }
    push_character(character, &space_style, element_path, output);
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
        let preserve_spaces = matches!(
            style.white_space,
            WhiteSpace::Pre | WhiteSpace::PreWrap | WhiteSpace::BreakSpaces
        );
        while !preserve_spaces && last.text.ends_with(' ') {
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
        if last.source_first_letter_character.is_none()
            && is_first_letter_character(source_character)
        {
            last.source_first_letter_character = Some(source_character);
        }
    } else {
        output.push(TextRun {
            text: character.to_string(),
            source_first_letter_character: is_first_letter_character(source_character)
                .then_some(source_character),
            style: style.clone(),
            element_path: element_path.to_vec(),
            selection_background: None,
            selection_foreground: None,
            first_line_block: None,
            first_line_foreground: None,
            first_line_background: None,
            first_line_decoration: None,
            first_line_opacity: None,
            first_line_font_style: None,
            first_letter_block: None,
            first_letter_foreground: None,
            first_letter_background: None,
            first_letter_decoration: None,
            first_letter_opacity: None,
            first_letter_font_style: None,
        });
    }
}

fn is_first_letter_character(character: char) -> bool {
    matches!(
        character.general_category(),
        GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter
            | GeneralCategory::DecimalNumber
            | GeneralCategory::LetterNumber
            | GeneralCategory::OtherNumber
            | GeneralCategory::MathSymbol
            | GeneralCategory::CurrencySymbol
            | GeneralCategory::ModifierSymbol
            | GeneralCategory::OtherSymbol
    )
}
