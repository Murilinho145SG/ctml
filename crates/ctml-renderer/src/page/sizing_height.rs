use ctml_css::{FontRelativeUnits, ViewportSize, parse_signed_font_relative_length_with_viewport};

use crate::style::{BorderLineStyle, LayoutSize, WritingMode};

use super::sizing::{block_size_constraints, resolve_width};
use super::sizing_padding::{resolve_borders, resolve_margin, resolve_padding};

pub(super) fn resolve_content_block_height(
    height: Option<&str>,
    min_height: Option<&str>,
    max_height: Option<&str>,
    content_height: f32,
    available_height: f32,
    font_size: f32,
    line_height: f32,
    font_relative_units: FontRelativeUnits,
    viewport_size: ViewportSize,
) -> f32 {
    resolve_used_height(
        height,
        min_height,
        max_height,
        content_height,
        available_height,
        font_size,
        line_height,
        font_relative_units,
        viewport_size,
    )
}

pub(super) fn resolve_layout_height_with_ancestors(
    height: Option<&str>,
    min_height: Option<&str>,
    max_height: Option<&str>,
    available_height: f32,
    available_width: f32,
    ancestors: &[LayoutSize],
    page_writing_mode: WritingMode,
    border_box: bool,
    aspect_ratio: Option<f32>,
    border_width: &[Option<String>; 4],
    border_style: &[BorderLineStyle; 4],
    padding: &[Option<String>; 4],
    margin: &[Option<String>; 4],
    font_size: f32,
    line_height: f32,
    font_relative_units: FontRelativeUnits,
    viewport_size: ViewportSize,
) -> f32 {
    let mut containing_height = available_height;
    let mut containing_width = available_width;
    for ancestor in ancestors.iter().take(ancestors.len().saturating_sub(1)) {
        let next_width = super::sizing::resolve_layout_width(
            std::slice::from_ref(ancestor),
            containing_width,
            page_writing_mode,
        );
        let (height, min_height, max_height) = block_size_constraints(ancestor, page_writing_mode);
        if height.is_none()
            && min_height.is_none()
            && max_height.is_none()
            && ancestor.aspect_ratio.is_none()
        {
            containing_width = next_width;
            continue;
        }
        containing_height = resolve_block_box_extent(
            ancestor,
            0.0,
            containing_height,
            containing_width,
            page_writing_mode,
        );
        containing_width = next_width;
    }
    let padding = resolve_padding_values(
        padding,
        containing_width,
        font_size,
        line_height,
        font_relative_units,
        viewport_size,
    );
    let borders = resolve_border_values(
        border_width,
        border_style,
        containing_width,
        font_size,
        line_height,
        font_relative_units,
        viewport_size,
    );
    let block_decoration = block_padding_extent(padding, page_writing_mode)
        + block_border_extent(borders, page_writing_mode);
    let inline_decoration = if page_writing_mode == WritingMode::HorizontalTb {
        padding[1] + padding[3] + borders[1] + borders[3]
    } else {
        padding[0] + padding[2] + borders[0] + borders[2]
    };
    let aspect_height = if height.is_none() && page_writing_mode == WritingMode::HorizontalTb {
        aspect_ratio.map(|ratio| {
            let inline_size =
                super::sizing::resolve_layout_width(ancestors, available_width, page_writing_mode)
                    + if border_box { inline_decoration } else { 0.0 };
            (inline_size / ratio - if border_box { block_decoration } else { 0.0 }).max(0.0)
        })
    } else {
        None
    };
    resolve_block_size(
        height,
        min_height,
        max_height,
        aspect_height.unwrap_or_default(),
        containing_height,
        block_decoration,
        border_box,
        font_size,
        line_height,
        font_relative_units,
        viewport_size,
    ) + block_margin_extent(
        resolve_margin_values(
            margin,
            containing_width,
            font_size,
            line_height,
            font_relative_units,
            viewport_size,
        ),
        page_writing_mode,
    )
}

pub(super) fn resolve_block_box_extent(
    size: &LayoutSize,
    content_height: f32,
    available_height: f32,
    containing_width: f32,
    page_writing_mode: WritingMode,
) -> f32 {
    let (height, min_height, max_height) = block_size_constraints(size, page_writing_mode);
    let content_height = if size.size_contained {
        size.contain_intrinsic_height
            .as_deref()
            .and_then(|value| {
                resolve_width(
                    value,
                    containing_width,
                    size.font_size,
                    size.line_height,
                    size.font_relative_units,
                    size.viewport_size,
                )
            })
            .unwrap_or_default()
    } else {
        content_height
    };
    let padding = resolve_padding(size, containing_width);
    let borders = resolve_borders(size, containing_width);
    let margin = resolve_margin(size, containing_width);
    let padding_start = if page_writing_mode == WritingMode::HorizontalTb
        && size.writing_mode == WritingMode::HorizontalTb
    {
        padding[0]
    } else {
        0.0
    };
    let margin_start = margin_block_start(margin, page_writing_mode);
    let content_margin_start = if margin_start < 0.0 || content_height <= f32::EPSILON {
        margin_start
    } else {
        0.0
    };
    let aspect_height = if height.is_none()
        && page_writing_mode == WritingMode::HorizontalTb
        && size.writing_mode == WritingMode::HorizontalTb
    {
        size.aspect_ratio.map(|ratio| {
            let inline_content = super::sizing::resolve_layout_width(
                std::slice::from_ref(size),
                containing_width,
                page_writing_mode,
            );
            let inline_decoration = padding[1] + padding[3] + borders[1] + borders[3];
            let block_decoration = padding[0] + padding[2] + borders[0] + borders[2];
            let inline_size = inline_content
                + if size.border_box {
                    inline_decoration
                } else {
                    0.0
                };
            (inline_size / ratio
                - if size.border_box {
                    block_decoration
                } else {
                    0.0
                })
            .max((content_height - padding_start).max(0.0))
        })
    } else {
        None
    };
    resolve_block_size(
        height,
        min_height,
        max_height,
        aspect_height.unwrap_or((content_height - padding_start).max(0.0)),
        available_height,
        block_padding_extent(padding, page_writing_mode)
            + block_border_extent(borders, page_writing_mode),
        size.border_box,
        size.font_size,
        size.line_height,
        size.font_relative_units,
        size.viewport_size,
    ) + content_margin_start
        + margin_block_end(margin, page_writing_mode)
}

fn resolve_margin_values(
    margins: &[Option<String>; 4],
    containing_width: f32,
    font_size: f32,
    line_height: f32,
    units: FontRelativeUnits,
    viewport: ViewportSize,
) -> [f32; 4] {
    std::array::from_fn(|index| {
        margins[index]
            .as_deref()
            .and_then(|value| {
                value
                    .strip_suffix('%')
                    .and_then(|percent| percent.trim().parse::<f32>().ok())
                    .map(|percent| containing_width * percent / 100.0)
                    .filter(|margin| margin.is_finite())
                    .or_else(|| {
                        parse_signed_font_relative_length_with_viewport(
                            value,
                            font_size,
                            line_height,
                            units,
                            viewport,
                        )
                    })
            })
            .unwrap_or_default()
    })
}

fn margin_block_start(margin: [f32; 4], page_writing_mode: WritingMode) -> f32 {
    if page_writing_mode == WritingMode::HorizontalTb {
        margin[0]
    } else if page_writing_mode == WritingMode::VerticalRl {
        margin[1]
    } else {
        margin[3]
    }
}

fn margin_block_end(margin: [f32; 4], page_writing_mode: WritingMode) -> f32 {
    let side = if page_writing_mode == WritingMode::HorizontalTb {
        2
    } else if page_writing_mode == WritingMode::VerticalRl {
        3
    } else {
        1
    };
    margin[side]
}

fn block_margin_extent(margin: [f32; 4], page_writing_mode: WritingMode) -> f32 {
    if page_writing_mode == WritingMode::HorizontalTb {
        margin[0] + margin[2]
    } else {
        margin[1] + margin[3]
    }
}

fn resolve_padding_values(
    padding: &[Option<String>; 4],
    containing_width: f32,
    font_size: f32,
    line_height: f32,
    units: FontRelativeUnits,
    viewport: ViewportSize,
) -> [f32; 4] {
    std::array::from_fn(|index| {
        padding[index]
            .as_deref()
            .and_then(|value| {
                resolve_width(
                    value,
                    containing_width,
                    font_size,
                    line_height,
                    units,
                    viewport,
                )
            })
            .unwrap_or_default()
    })
}

fn block_padding_extent(padding: [f32; 4], page_writing_mode: WritingMode) -> f32 {
    if page_writing_mode == WritingMode::HorizontalTb {
        padding[0] + padding[2]
    } else {
        padding[1] + padding[3]
    }
}

fn resolve_border_values(
    widths: &[Option<String>; 4],
    styles: &[BorderLineStyle; 4],
    containing_width: f32,
    font_size: f32,
    line_height: f32,
    units: FontRelativeUnits,
    viewport: ViewportSize,
) -> [f32; 4] {
    std::array::from_fn(|index| {
        if styles[index] == BorderLineStyle::None {
            return 0.0;
        }
        let Some(value) = widths[index].as_deref() else {
            return 3.0;
        };
        match value {
            "thin" => 1.0,
            "medium" => 3.0,
            "thick" => 5.0,
            _ => resolve_width(
                value,
                containing_width,
                font_size,
                line_height,
                units,
                viewport,
            )
            .unwrap_or_default(),
        }
    })
}

fn block_border_extent(border: [f32; 4], page_writing_mode: WritingMode) -> f32 {
    if page_writing_mode == WritingMode::HorizontalTb {
        border[0] + border[2]
    } else {
        border[1] + border[3]
    }
}

fn resolve_block_size(
    height: Option<&str>,
    min_height: Option<&str>,
    max_height: Option<&str>,
    content_height: f32,
    available_height: f32,
    padding: f32,
    border_box: bool,
    font_size: f32,
    line_height: f32,
    font_relative_units: FontRelativeUnits,
    viewport_size: ViewportSize,
) -> f32 {
    let resolve_border_box = |value: Option<&str>| {
        value.and_then(|value| {
            resolve_width(
                value,
                available_height,
                font_size,
                line_height,
                font_relative_units,
                viewport_size,
            )
        })
    };
    let (height, min_height, max_height, padding) = if border_box {
        let to_content = |value: Option<&str>| {
            resolve_border_box(value).map(|value| format!("{}px", (value - padding).max(0.0)))
        };
        (
            to_content(height),
            to_content(min_height),
            to_content(max_height),
            padding,
        )
    } else {
        (
            height.map(str::to_owned),
            min_height.map(str::to_owned),
            max_height.map(str::to_owned),
            padding,
        )
    };
    resolve_content_block_height(
        height.as_deref(),
        min_height.as_deref(),
        max_height.as_deref(),
        content_height,
        available_height,
        font_size,
        line_height,
        font_relative_units,
        viewport_size,
    ) + padding
}

fn resolve_used_height(
    height: Option<&str>,
    min_height: Option<&str>,
    max_height: Option<&str>,
    auto_height: f32,
    available_height: f32,
    font_size: f32,
    line_height: f32,
    font_relative_units: FontRelativeUnits,
    viewport_size: ViewportSize,
) -> f32 {
    let resolve = |value: &str| {
        resolve_width(
            value,
            available_height,
            font_size,
            line_height,
            font_relative_units,
            viewport_size,
        )
    };
    let min_height = min_height.and_then(|value| resolve(value));
    let max_height = max_height.and_then(|value| resolve(value));
    let specified_height = height
        .and_then(|value| resolve(value))
        .unwrap_or(auto_height);
    match (min_height, max_height) {
        (Some(min), Some(max)) if min > max => specified_height.max(min),
        (Some(min), Some(max)) => specified_height.max(min).min(max),
        (Some(min), None) => specified_height.max(min),
        (None, Some(max)) => specified_height.min(max),
        (None, None) => specified_height,
    }
}
