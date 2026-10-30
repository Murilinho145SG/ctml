use crate::style::TextStyle;

pub(super) fn apply_display(style: &mut TextStyle, value: &str) {
    let value = value
        .split_ascii_whitespace()
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    let value = value.iter().map(String::as_str).collect::<Vec<_>>();
    match value.as_slice() {
        ["none"] => {
            style.display_none = true;
            style.display_block = None;
            style.display_list_item = Some(false);
        }
        ["block"] | ["block", "flow"] | ["block", "flow-root"] => {
            style.display_none = false;
            style.display_block = Some(true);
            style.display_list_item = Some(false);
        }
        ["flow-root"] => {
            style.display_none = false;
            style.display_block = Some(true);
            style.display_list_item = Some(false);
        }
        ["inline"] | ["inline", "flow"] => {
            style.display_none = false;
            style.display_block = Some(false);
            style.display_list_item = Some(false);
        }
        ["inline-block"] | ["inline", "flow-root"] => {
            style.display_none = false;
            style.display_block = Some(false);
            style.display_list_item = Some(false);
        }
        ["contents"] => {
            style.display_none = false;
            style.display_block = Some(false);
            style.display_list_item = Some(false);
        }
        ["list-item"] | ["block", "flow", "list-item"] => {
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
