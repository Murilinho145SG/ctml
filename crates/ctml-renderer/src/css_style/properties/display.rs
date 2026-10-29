use crate::style::TextStyle;

pub(super) fn apply_display(style: &mut TextStyle, value: &str) {
    match value.to_ascii_lowercase().as_str() {
        "none" => {
            style.display_none = true;
            style.display_block = None;
            style.display_list_item = Some(false);
        }
        "block" => {
            style.display_none = false;
            style.display_block = Some(true);
            style.display_list_item = Some(false);
        }
        "flow-root" => {
            style.display_none = false;
            style.display_block = Some(true);
            style.display_list_item = Some(false);
        }
        "inline" => {
            style.display_none = false;
            style.display_block = Some(false);
            style.display_list_item = Some(false);
        }
        "inline-block" => {
            style.display_none = false;
            style.display_block = Some(false);
            style.display_list_item = Some(false);
        }
        "contents" => {
            style.display_none = false;
            style.display_block = Some(false);
            style.display_list_item = Some(false);
        }
        "list-item" => {
            style.display_none = false;
            style.display_block = Some(true);
            style.display_list_item = Some(true);
        }
        _ => {}
    }
}

pub(super) fn inherit_display(style: &mut TextStyle, inherited: &TextStyle) {
    style.display_none = inherited.display_none;
    style.display_block = inherited.display_block;
    style.display_list_item = inherited.display_list_item;
}

pub(super) fn reset_display(style: &mut TextStyle) {
    style.display_none = false;
    style.display_block = Some(false);
    style.display_list_item = Some(false);
}

pub(super) fn revert_display(style: &mut TextStyle) {
    style.display_none = false;
    style.display_block = None;
    style.display_list_item = None;
}
