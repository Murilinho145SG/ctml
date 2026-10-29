use cosmic_text::{Color, DecorationSpan, LayoutRun, Renderer, UnderlineStyle};

use crate::style::TextDecorationStyle;

pub(super) fn render_patterned_decorations<R: Renderer>(
    renderer: &mut R,
    run: &LayoutRun<'_>,
    default_color: Color,
) {
    for span in run.decorations {
        render_span(renderer, run, span, default_color);
    }
}

fn render_span<R: Renderer>(
    renderer: &mut R,
    run: &LayoutRun<'_>,
    span: &DecorationSpan,
    default_color: Color,
) {
    let glyphs = &run.glyphs[span.glyph_range.clone()];
    if glyphs.is_empty() {
        return;
    }

    let mut x_min = f32::INFINITY;
    let mut x_max = f32::NEG_INFINITY;
    for glyph in glyphs {
        x_min = x_min.min(glyph.x);
        x_max = x_max.max(glyph.x + glyph.w);
    }
    let width = (x_max - x_min) as u32;
    if width == 0 {
        return;
    }

    let style = match glyphs[0].metadata {
        1 => TextDecorationStyle::Double,
        2 => TextDecorationStyle::Dotted,
        3 => TextDecorationStyle::Dashed,
        4 => TextDecorationStyle::Wavy,
        _ => TextDecorationStyle::Solid,
    };
    let data = &span.data;
    let decoration = &data.text_decoration;
    let font_size = span.font_size;
    let thickness = (data.underline_metrics.thickness * font_size)
        .max(1.0)
        .round() as u32;

    if decoration.underline != UnderlineStyle::None {
        let color = decoration
            .underline_color_opt
            .or(span.color_opt)
            .unwrap_or(default_color);
        let y = (run.line_y - data.underline_metrics.offset * font_size) as i32 + 1;
        let line_style = if decoration.underline == UnderlineStyle::Double {
            TextDecorationStyle::Double
        } else {
            style
        };
        draw_line(
            renderer,
            x_min as i32,
            y,
            width,
            thickness,
            line_style,
            color,
        );
    }

    if decoration.strikethrough {
        let color = decoration
            .strikethrough_color_opt
            .or(span.color_opt)
            .unwrap_or(default_color);
        let thickness = (data.strikethrough_metrics.thickness * font_size)
            .max(1.0)
            .round() as u32;
        let y = (run.line_y - data.strikethrough_metrics.offset * font_size) as i32 + 1;
        draw_line(renderer, x_min as i32, y, width, thickness, style, color);
    }

    if decoration.overline {
        let color = decoration
            .overline_color_opt
            .or(span.color_opt)
            .unwrap_or(default_color);
        let y = (run.line_y - data.ascent * font_size).max(run.line_top) as i32 + 1;
        draw_line(renderer, x_min as i32, y, width, thickness, style, color);
    }
}

fn draw_line<R: Renderer>(
    renderer: &mut R,
    x: i32,
    y: i32,
    width: u32,
    thickness: u32,
    style: TextDecorationStyle,
    color: Color,
) {
    match style {
        TextDecorationStyle::Solid => rectangle(renderer, x, y, width, thickness, color),
        TextDecorationStyle::Double => {
            rectangle(renderer, x, y, width, thickness, color);
            rectangle(
                renderer,
                x,
                y + (2 * thickness) as i32,
                width,
                thickness,
                color,
            );
        }
        TextDecorationStyle::Dashed => {
            let dash = thickness.saturating_mul(3).max(1);
            let step = dash.saturating_add(thickness.saturating_mul(2)).max(1);
            let mut offset = 0;
            while offset < width {
                rectangle(
                    renderer,
                    x + offset as i32,
                    y,
                    dash.min(width - offset),
                    thickness,
                    color,
                );
                offset += step;
            }
        }
        TextDecorationStyle::Dotted => {
            let radius = thickness as f32 / 2.0;
            let step = thickness.saturating_mul(2).max(1);
            let center_y = y as f32 + radius;
            let mut center_x = x as f32 + radius;
            let end = x + width as i32;
            while center_x < end as f32 {
                for dy in 0..thickness as i32 {
                    for dx in 0..thickness as i32 {
                        let px = center_x.floor() as i32 + dx - radius.floor() as i32;
                        let py = center_y.floor() as i32 + dy - radius.floor() as i32;
                        let distance_x = px as f32 + 0.5 - center_x;
                        let distance_y = py as f32 + 0.5 - center_y;
                        if distance_x * distance_x + distance_y * distance_y <= radius * radius {
                            rectangle(renderer, px, py, 1, 1, color);
                        }
                    }
                }
                center_x += step as f32;
            }
        }
        TextDecorationStyle::Wavy => {
            let period = (thickness as f32 * 4.0).max(4.0);
            let amplitude = (thickness as f32).max(1.0);
            for offset in 0..width {
                let phase = offset as f32 * std::f32::consts::TAU / period;
                let center_y = y as f32 + amplitude * phase.sin();
                let top = (center_y - thickness as f32 / 2.0).round() as i32;
                rectangle(renderer, x + offset as i32, top, 1, thickness, color);
            }
        }
    }
}

fn rectangle<R: Renderer>(renderer: &mut R, x: i32, y: i32, width: u32, height: u32, color: Color) {
    renderer.rectangle(x, y, width, height, color);
}
