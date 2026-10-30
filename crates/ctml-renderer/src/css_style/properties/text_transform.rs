use crate::style::{TextStyle, TextTransform};

pub(super) fn apply_text_transform_property(style: &mut TextStyle, value: &str) {
    if let Some((case, full_width, full_size_kana)) = parse_text_transform(value) {
        style.text_transform = case;
        style.text_transform_full_width = full_width;
        style.text_transform_full_size_kana = full_size_kana;
    }
}

fn parse_text_transform(value: &str) -> Option<(TextTransform, bool, bool)> {
    let mut case = TextTransform::None;
    let mut case_seen = false;
    let mut none_seen = false;
    let mut full_width = false;
    let mut full_size_kana = false;
    let mut has_value = false;
    for token in value.split_ascii_whitespace() {
        has_value = true;
        match token.to_ascii_lowercase().as_str() {
            "none" if !none_seen => none_seen = true,
            "capitalize" if !case_seen => {
                case = TextTransform::Capitalize;
                case_seen = true;
            }
            "uppercase" if !case_seen => {
                case = TextTransform::Uppercase;
                case_seen = true;
            }
            "lowercase" if !case_seen => {
                case = TextTransform::Lowercase;
                case_seen = true;
            }
            "full-width" if !full_width => full_width = true,
            "full-size-kana" if !full_size_kana => full_size_kana = true,
            _ => return None,
        }
    }
    (has_value && (!none_seen || (!case_seen && !full_width && !full_size_kana))).then_some((
        case,
        full_width,
        full_size_kana,
    ))
}
