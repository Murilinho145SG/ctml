use crate::style::{BoxDecorationBreak, TextStyle};

pub(super) fn apply_box_decoration_break(style: &mut TextStyle, value: &str) {
    match value.trim().to_ascii_lowercase().as_str() {
        "slice" => style.box_decoration_break = BoxDecorationBreak::Slice,
        "clone" => style.box_decoration_break = BoxDecorationBreak::Clone,
        _ => {}
    }
}

pub(super) fn inherit_css_property(
    style: &mut TextStyle,
    inherited: &TextStyle,
    property: &str,
) -> bool {
    if property == "box-decoration-break" {
        style.box_decoration_break = inherited.box_decoration_break;
        true
    } else {
        false
    }
}

pub(super) fn initial_css_property(style: &mut TextStyle, property: &str) -> bool {
    if property == "box-decoration-break" {
        style.box_decoration_break = BoxDecorationBreak::Slice;
        true
    } else {
        false
    }
}
