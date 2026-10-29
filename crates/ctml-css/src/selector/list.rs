pub(super) fn split_selector_list(source: &str) -> Vec<&str> {
    let mut selectors = Vec::new();
    let mut start = 0;
    let mut bracket_depth = 0;
    let mut paren_depth = 0;
    let mut quote = None;
    for (index, character) in source.char_indices() {
        if quote.is_some() {
            if quote == Some(character) {
                quote = None;
            }
            continue;
        }
        if character == '"' || character == '\'' {
            quote = Some(character);
        } else if character == '[' {
            bracket_depth += 1;
        } else if character == ']' {
            bracket_depth -= 1;
        } else if bracket_depth == 0 && character == '(' {
            paren_depth += 1;
        } else if bracket_depth == 0 && character == ')' {
            paren_depth -= 1;
        } else if bracket_depth == 0 && paren_depth == 0 && character == ',' {
            selectors.push(&source[start..index]);
            start = index + 1;
        }
    }
    selectors.push(&source[start..]);
    selectors
}
