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
    let width_exact = x_max - x_min;
    let width = width_exact as u32;
    let tail_coverage = ((width_exact - width as f32) * 255.0).round() as u8;
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
            tail_coverage,
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
        draw_line(
            renderer,
            x_min as i32,
            y,
            width,
            tail_coverage,
            thickness,
            style,
            color,
        );
    }

    if decoration.overline {
        let color = decoration
            .overline_color_opt
            .or(span.color_opt)
            .unwrap_or(default_color);
        let y = (run.line_y - data.ascent * font_size).max(run.line_top) as i32 + 1;
        draw_line(
            renderer,
            x_min as i32,
            y,
            width,
            tail_coverage,
            thickness,
            style,
            color,
        );
    }
}

fn draw_line<R: Renderer>(
    renderer: &mut R,
    x: i32,
    y: i32,
    width: u32,
    tail_coverage: u8,
    thickness: u32,
    style: TextDecorationStyle,
    color: Color,
) {
    match style {
        TextDecorationStyle::Solid => {
            rectangle(renderer, x, y, width, thickness, color);
            draw_tail(renderer, x, y, width, thickness, tail_coverage, color);
        }
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
            draw_tail(renderer, x, y, width, thickness, tail_coverage, color);
            draw_tail(
                renderer,
                x,
                y + (2 * thickness) as i32,
                width,
                thickness,
                tail_coverage,
                color,
            );
        }
        TextDecorationStyle::Dashed => {
            let dash = thickness.saturating_mul(3).max(1);
            let step = dash.saturating_add(thickness.saturating_mul(2)).max(1);
            let edge_color = with_alpha(color, 64);
            let mut offset = 0;
            while offset + thickness < width {
                let length = dash.min(width - offset);
                rectangle(renderer, x + offset as i32, y, length, thickness, color);
                rectangle(renderer, x + offset as i32, y - 1, length, 1, edge_color);
                rectangle(
                    renderer,
                    x + offset as i32,
                    y + thickness as i32,
                    length,
                    1,
                    edge_color,
                );
                offset += step;
            }
        }
        TextDecorationStyle::Dotted => {
            let step = thickness.saturating_mul(2).max(1);
            let edge_color = with_alpha(color, 51);
            let mut offset = 0;
            while offset < width {
                let dot_x = x + offset as i32;
                rectangle(renderer, dot_x, y, 1, thickness, color);
                rectangle(renderer, dot_x, y - 1, 1, 1, edge_color);
                rectangle(renderer, dot_x, y + thickness as i32, 1, 1, edge_color);
                offset += step;
            }
        }
        TextDecorationStyle::Wavy => {
            const SAMPLE_GRID: u32 = 8;
            let period = 7.0;
            let amplitude = 1.36;
            let half_thickness = 1.0;
            for offset in 0..width {
                for row in y - 1..=y + 6 {
                    let mut covered_samples = 0;
                    for sample_x in 0..SAMPLE_GRID {
                        let x_position =
                            offset as f32 + (sample_x as f32 + 0.5) / SAMPLE_GRID as f32;
                        let phase = (x_position - 3.5) * std::f32::consts::TAU / period;
                        let center_y = y as f32 + 2.52 - amplitude * phase.sin();
                        for sample_y in 0..SAMPLE_GRID {
                            let y_position =
                                row as f32 + (sample_y as f32 + 0.5) / SAMPLE_GRID as f32;
                            if (y_position - center_y).abs() <= half_thickness {
                                covered_samples += 1;
                            }
                        }
                    }
                    if covered_samples > 0 {
                        let alpha = ((covered_samples * 255 + 32) / 64) as u8;
                        rectangle(
                            renderer,
                            x + offset as i32,
                            row,
                            1,
                            1,
                            with_alpha(color, alpha),
                        );
                    }
                }
            }
        }
    }
}

fn with_alpha(color: Color, coverage: u8) -> Color {
    let (red, green, blue, alpha) = color.as_rgba_tuple();
    Color::rgba(
        red,
        green,
        blue,
        ((alpha as u16 * coverage as u16 + 127) / 255) as u8,
    )
}

fn draw_tail<R: Renderer>(
    renderer: &mut R,
    x: i32,
    y: i32,
    width: u32,
    thickness: u32,
    coverage: u8,
    color: Color,
) {
    if coverage > 0 {
        rectangle(
            renderer,
            x + width as i32,
            y,
            1,
            thickness,
            with_alpha(color, coverage),
        );
    }
}

fn rectangle<R: Renderer>(renderer: &mut R, x: i32, y: i32, width: u32, height: u32, color: Color) {
    renderer.rectangle(x, y, width, height, color);
}
