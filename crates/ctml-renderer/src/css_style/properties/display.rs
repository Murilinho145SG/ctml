use crate::style::TextStyle;

pub(super) fn apply_display(style: &mut TextStyle, value: &str) {
    match value.to_ascii_lowercase().as_str() {
        "none" => {
            style.display_none = true;
            style.display_block = None;
        }
        "block" => {
            style.display_none = false;
            style.display_block = Some(true);
        }
        "inline" => {
            style.display_none = false;
            style.display_block = Some(false);
        }
        "contents" => {
            style.display_none = false;
            style.display_block = Some(false);
        }
        _ => {}
    }
}

pub(super) fn inherit_display(style: &mut TextStyle, inherited: &TextStyle) {
    style.display_none = inherited.display_none;
    style.display_block = inherited.display_block;
}

pub(super) fn reset_display(style: &mut TextStyle) {
    style.display_none = false;
    style.display_block = Some(false);
}

pub(super) fn revert_display(style: &mut TextStyle) {
    style.display_none = false;
    style.display_block = None;
}
