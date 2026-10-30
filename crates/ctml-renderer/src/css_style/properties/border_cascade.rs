use crate::style::{BorderLineStyle, TextStyle};

pub(super) fn inherit_css_property(
    style: &mut TextStyle,
    inherited: &TextStyle,
    property: &str,
) -> bool {
    match property {
        "border" => copy_border(style, inherited, 0..4, false),
        "border-top" | "border-right" | "border-bottom" | "border-left" => copy_border(
            style,
            inherited,
            border_physical_side(property)..border_physical_side(property) + 1,
            false,
        ),
        "border-inline" => copy_border(style, inherited, 0..2, true),
        "border-block" => copy_border(style, inherited, 2..4, true),
        "border-inline-start" | "border-inline-end" | "border-block-start" | "border-block-end" => {
            copy_border(
                style,
                inherited,
                border_logical_side(property)..border_logical_side(property) + 1,
                true,
            )
        }
        "border-width" => {
            style.border_width.clone_from(&inherited.border_width);
            style.physical_border_width_orders =
                [style.current_declaration_order.saturating_add(1); 4];
        }
        "border-style" => {
            style.border_style = inherited.border_style;
            style.physical_border_style_orders =
                [style.current_declaration_order.saturating_add(1); 4];
        }
        "border-inline-width" | "border-block-width" => {
            let start = usize::from(property == "border-block-width") * 2;
            style.logical_border_width[start..start + 2]
                .clone_from_slice(&inherited.logical_border_width[start..start + 2]);
            style.logical_border_width_orders[start..start + 2]
                .fill(style.current_declaration_order.saturating_add(1));
        }
        "border-inline-style" | "border-block-style" => {
            let start = usize::from(property == "border-block-style") * 2;
            style.logical_border_style[start..start + 2]
                .clone_from_slice(&inherited.logical_border_style[start..start + 2]);
            style.logical_border_style_orders[start..start + 2]
                .fill(style.current_declaration_order.saturating_add(1));
        }
        "border-color" => {
            style.border_color = inherited.border_color;
            style.physical_border_color_orders =
                [style.current_declaration_order.saturating_add(1); 4];
        }
        "border-inline-color" | "border-block-color" => {
            let start = usize::from(property == "border-block-color") * 2;
            style.logical_border_color[start..start + 2]
                .clone_from_slice(&inherited.logical_border_color[start..start + 2]);
            style.logical_border_color_orders[start..start + 2]
                .fill(style.current_declaration_order.saturating_add(1));
        }
        _ => {
            if let Some((logical, side)) = border_color_side_property(property) {
                if logical {
                    style.logical_border_color[side] = inherited.logical_border_color[side];
                    style.logical_border_color_orders[side] =
                        style.current_declaration_order.saturating_add(1);
                } else {
                    style.border_color[side] = inherited.border_color[side];
                    style.physical_border_color_orders[side] =
                        style.current_declaration_order.saturating_add(1);
                }
                return true;
            }
            let Some((logical, side, width)) = border_side_property(property) else {
                return false;
            };
            if logical {
                if width {
                    style.logical_border_width[side] = inherited.logical_border_width[side].clone();
                    style.logical_border_width_orders[side] =
                        style.current_declaration_order.saturating_add(1);
                } else {
                    style.logical_border_style[side] = inherited.logical_border_style[side];
                    style.logical_border_style_orders[side] =
                        style.current_declaration_order.saturating_add(1);
                }
            } else if width {
                style.border_width[side] = inherited.border_width[side].clone();
                style.physical_border_width_orders[side] =
                    style.current_declaration_order.saturating_add(1);
            } else {
                style.border_style[side] = inherited.border_style[side];
                style.physical_border_style_orders[side] =
                    style.current_declaration_order.saturating_add(1);
            }
        }
    }
    true
}

pub(super) fn initial_css_property(style: &mut TextStyle, property: &str) -> bool {
    match property {
        "border" => reset_border(style, 0..4, false),
        "border-top" | "border-right" | "border-bottom" | "border-left" => {
            let side = border_physical_side(property);
            reset_border(style, side..side + 1, false);
        }
        "border-inline" => reset_border(style, 0..2, true),
        "border-block" => reset_border(style, 2..4, true),
        "border-inline-start" | "border-inline-end" | "border-block-start" | "border-block-end" => {
            let side = border_logical_side(property);
            reset_border(style, side..side + 1, true);
        }
        "border-width" => {
            style.border_width = std::array::from_fn(|_| None);
            style.physical_border_width_orders =
                [style.current_declaration_order.saturating_add(1); 4];
        }
        "border-style" => {
            style.border_style = [BorderLineStyle::None; 4];
            style.physical_border_style_orders =
                [style.current_declaration_order.saturating_add(1); 4];
        }
        "border-inline-width" | "border-block-width" => {
            let start = usize::from(property == "border-block-width") * 2;
            style.logical_border_width[start..start + 2].fill(Some("medium".to_owned()));
            style.logical_border_width_orders[start..start + 2]
                .fill(style.current_declaration_order.saturating_add(1));
        }
        "border-inline-style" | "border-block-style" => {
            let start = usize::from(property == "border-block-style") * 2;
            style.logical_border_style[start..start + 2].fill(Some(BorderLineStyle::None));
            style.logical_border_style_orders[start..start + 2]
                .fill(style.current_declaration_order.saturating_add(1));
        }
        "border-color" => {
            style.border_color = std::array::from_fn(|_| None);
            style.physical_border_color_orders =
                [style.current_declaration_order.saturating_add(1); 4];
        }
        "border-inline-color" | "border-block-color" => {
            let start = usize::from(property == "border-block-color") * 2;
            style.logical_border_color[start..start + 2].fill(None);
            style.logical_border_color_orders[start..start + 2]
                .fill(style.current_declaration_order.saturating_add(1));
        }
        _ => {
            if let Some((logical, side)) = border_color_side_property(property) {
                if logical {
                    style.logical_border_color[side] = None;
                    style.logical_border_color_orders[side] =
                        style.current_declaration_order.saturating_add(1);
                } else {
                    style.border_color[side] = None;
                    style.physical_border_color_orders[side] =
                        style.current_declaration_order.saturating_add(1);
                }
                return true;
            }
            let Some((logical, side, width)) = border_side_property(property) else {
                return false;
            };
            if logical {
                if width {
                    style.logical_border_width[side] = Some("medium".to_owned());
                    style.logical_border_width_orders[side] =
                        style.current_declaration_order.saturating_add(1);
                } else {
                    style.logical_border_style[side] = Some(BorderLineStyle::None);
                    style.logical_border_style_orders[side] =
                        style.current_declaration_order.saturating_add(1);
                }
            } else if width {
                style.border_width[side] = None;
                style.physical_border_width_orders[side] =
                    style.current_declaration_order.saturating_add(1);
            } else {
                style.border_style[side] = BorderLineStyle::None;
                style.physical_border_style_orders[side] =
                    style.current_declaration_order.saturating_add(1);
            }
        }
    }
    true
}

fn border_side_property(property: &str) -> Option<(bool, usize, bool)> {
    let (side_name, width) = if let Some(side) = property.strip_suffix("-width") {
        (side, true)
    } else if let Some(side) = property.strip_suffix("-style") {
        (side, false)
    } else {
        return None;
    };
    let side_name = side_name.strip_prefix("border-")?;
    let (logical, side) = match side_name {
        "top" => (false, 0),
        "right" => (false, 1),
        "bottom" => (false, 2),
        "left" => (false, 3),
        "inline-start" => (true, 0),
        "inline-end" => (true, 1),
        "block-start" => (true, 2),
        "block-end" => (true, 3),
        _ => return None,
    };
    Some((logical, side, width))
}

fn border_color_side_property(property: &str) -> Option<(bool, usize)> {
    let side_name = property.strip_suffix("-color")?.strip_prefix("border-")?;
    Some(match side_name {
        "top" => (false, 0),
        "right" => (false, 1),
        "bottom" => (false, 2),
        "left" => (false, 3),
        "inline-start" => (true, 0),
        "inline-end" => (true, 1),
        "block-start" => (true, 2),
        "block-end" => (true, 3),
        _ => return None,
    })
}

fn border_physical_side(property: &str) -> usize {
    match property {
        "border-top" => 0,
        "border-right" => 1,
        "border-bottom" => 2,
        _ => 3,
    }
}

fn border_logical_side(property: &str) -> usize {
    match property {
        "border-inline-start" => 0,
        "border-inline-end" => 1,
        "border-block-start" => 2,
        _ => 3,
    }
}

fn copy_border(
    style: &mut TextStyle,
    inherited: &TextStyle,
    range: std::ops::Range<usize>,
    logical: bool,
) {
    let order = style.current_declaration_order.saturating_add(1);
    for side in range {
        if logical {
            style.logical_border_width[side] = inherited.logical_border_width[side].clone();
            style.logical_border_style[side] = inherited.logical_border_style[side];
            style.logical_border_color[side] = inherited.logical_border_color[side];
            style.logical_border_width_orders[side] = order;
            style.logical_border_style_orders[side] = order;
            style.logical_border_color_orders[side] = order;
        } else {
            style.border_width[side] = inherited.border_width[side].clone();
            style.border_style[side] = inherited.border_style[side];
            style.border_color[side] = inherited.border_color[side];
            style.physical_border_width_orders[side] = order;
            style.physical_border_style_orders[side] = order;
            style.physical_border_color_orders[side] = order;
        }
    }
}

fn reset_border(style: &mut TextStyle, range: std::ops::Range<usize>, logical: bool) {
    let order = style.current_declaration_order.saturating_add(1);
    for side in range {
        if logical {
            style.logical_border_width[side] = Some("medium".to_owned());
            style.logical_border_style[side] = Some(BorderLineStyle::None);
            style.logical_border_color[side] = None;
            style.logical_border_width_orders[side] = order;
            style.logical_border_style_orders[side] = order;
            style.logical_border_color_orders[side] = order;
        } else {
            style.border_width[side] = None;
            style.border_style[side] = BorderLineStyle::None;
            style.border_color[side] = None;
            style.physical_border_width_orders[side] = order;
            style.physical_border_style_orders[side] = order;
            style.physical_border_color_orders[side] = order;
        }
    }
}
