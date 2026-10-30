use crate::style::TextStyle;

pub(super) fn apply_text_indent(style: &mut TextStyle, value: &str) {
    let mut length = Vec::new();
    let mut hanging = false;
    let mut each_line = false;
    for component in value.split_ascii_whitespace() {
        if component.eq_ignore_ascii_case("hanging") {
            if hanging {
                return;
            }
            hanging = true;
        } else if component.eq_ignore_ascii_case("each-line") {
            if each_line {
                return;
            }
            each_line = true;
        } else {
            length.push(component);
        }
    }
    if length.is_empty() {
        return;
    }
    let length = length.join(" ");
    if length.ends_with('%') {
        let Some(percent) = ctml_css::parse_css_percentage(&length) else {
            return;
        };
        if !percent.is_finite() {
            return;
        }
        style.text_indent = 0.0;
        style.text_indent_percent = percent;
    } else if let Some(indent) = ctml_css::parse_signed_font_relative_length_with_viewport(
        &length,
        style.font_size,
        style.line_height,
        style.font_relative_units,
        style.viewport_size,
    ) {
        style.text_indent = indent;
        style.text_indent_percent = 0.0;
    } else {
        return;
    }
    style.text_indent_hanging = hanging;
    style.text_indent_each_line = each_line;
}
