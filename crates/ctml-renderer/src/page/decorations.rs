use cosmic_text::{Color, DecorationSpan, Renderer, UnderlineStyle};
use std::collections::HashMap;

use super::CursorSpan;

use crate::style::{TextDecorationSkipInk, TextDecorationStyle};

use super::{
    overflow_clip::{self, ClipRect},
    pixel::PixelRenderer,
};

pub(super) fn render_patterned_decorations<'a>(
    renderer: &mut PixelRenderer<'a>,
    run: &super::PageLayoutRun<'_>,
    default_color: Color,
    auto_ink_mask: &'a [u8],
    all_ink_mask: &'a [u8],
    cursor_spans: &[CursorSpan],
    overflow_clips: &HashMap<usize, ClipRect>,
) {
    for span in run.decorations {
        let Some(glyph) = run.glyphs.get(span.glyph_range.start) else {
            continue;
        };
        if cursor_spans.iter().any(|span| {
            span.line == run.line_i
                && span.start <= glyph.start
                && glyph.start < span.end
                && span.visibility_hidden
        }) {
            continue;
        }
        let metadata = run
            .glyphs
            .get(span.glyph_range.start)
            .map_or(0, |glyph| glyph.metadata);
        let skip_ink = (metadata >> 4) & 0b11;
        renderer.set_skip_ink_mask(match skip_ink {
            value if value == TextDecorationSkipInk::None as usize => None,
            value if value == TextDecorationSkipInk::All as usize => Some(all_ink_mask),
            _ => Some(auto_ink_mask),
        });
        let Some(glyph) = run.glyphs.get(span.glyph_range.start) else {
            continue;
        };
        let path = cursor_spans
            .iter()
            .find(|cursor| {
                cursor.line == run.line_i && cursor.start <= glyph.start && glyph.start < cursor.end
            })
            .map_or(&[][..], |cursor| cursor.element_path.as_slice());
        renderer.set_clip_rect(overflow_clip::clip_for_path(path, overflow_clips));
        render_span(renderer, run, span, default_color);
    }
}

fn render_span<R: Renderer>(
    renderer: &mut R,
    run: &super::PageLayoutRun<'_>,
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

    let style = match glyphs[0].metadata & 0b111 {
        1 => TextDecorationStyle::Double,
        2 => TextDecorationStyle::Dotted,
        3 => TextDecorationStyle::Dashed,
        4 => TextDecorationStyle::Wavy,
        _ => TextDecorationStyle::Solid,
    };
    let data = &span.data;
    let decoration = &data.text_decoration;
    let line_y = run.line_y + run.offset_for_range(glyphs[0].start, glyphs[glyphs.len() - 1].end);
    let font_size = span.font_size;
    let metadata = glyphs[0].metadata;
    let has_custom_thickness = metadata & (1 << 6) != 0;
    let thickness = if has_custom_thickness {
        ((((metadata >> 8) & 0x00ff_ffff) as f32 / 256.0).round() as u32).max(1)
    } else {
        (data.underline_metrics.thickness * font_size)
            .max(1.0)
            .round() as u32
    };
    let thickness_offset = if has_custom_thickness {
        ((thickness - 1) / 2) as i32
    } else {
        0
    };

    if decoration.underline != UnderlineStyle::None {
        let color = decoration
            .underline_color_opt
            .or(span.color_opt)
            .unwrap_or(default_color);
        let has_explicit_underline_offset = metadata as u64 & (1_u64 << 32) != 0;
        let underline_offset = if has_explicit_underline_offset {
            ((((metadata as u64 >> 33) & 0xffff) as u16 as i16) as f32 / 16.0).round() as i32
        } else {
            0
        };
        let baseline_adjustment = i32::from(!has_explicit_underline_offset);
        let y = (line_y - data.underline_metrics.offset * font_size) as i32
            + baseline_adjustment
            + thickness_offset
            + underline_offset;
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
        let y = (line_y - data.strikethrough_metrics.offset * font_size) as i32 + thickness_offset;
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
        let text_top = line_y - data.ascent * font_size;
        let y = text_top as i32 - 2 + thickness_offset;
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

pub(super) fn draw_line<R: Renderer>(
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
            draw_dashed_line(renderer, x, y, width, thickness, color);
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
            const WAVELENGTH: f32 = 7.0;
            const CONTROL_POINT_DISTANCE: f32 = 3.0;
            const WAVE_OFFSET: f32 = 2.4;
            let half_thickness = 1.0;
            for offset in 0..width {
                for row in y - 1..=y + 6 {
                    let mut covered_samples = 0;
                    for sample_x in 0..SAMPLE_GRID {
                        let x_position =
                            offset as f32 + (sample_x as f32 + 0.5) / SAMPLE_GRID as f32;
                        let t = (x_position / WAVELENGTH).fract();
                        let one_minus_t = 1.0 - t;
                        let wave_y = 3.0 * one_minus_t.powi(2) * t * CONTROL_POINT_DISTANCE
                            - 3.0 * one_minus_t * t.powi(2) * CONTROL_POINT_DISTANCE;
                        let center_y = y as f32 + WAVE_OFFSET + wave_y;
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

fn draw_dashed_line<R: Renderer>(
    renderer: &mut R,
    x: i32,
    y: i32,
    width: u32,
    thickness: u32,
    color: Color,
) {
    let dash = thickness.saturating_mul(3).max(1) as f32;
    let base_gap = thickness.saturating_mul(2).max(1) as f32;
    let gap = adjusted_dash_gap(width as f32, dash, base_gap);
    let step = dash + gap;
    let edge_alpha = 65.0;
    let mut start = 0.0;
    while start < width as f32 {
        let end = (start + dash).min(width as f32);
        for pixel in start.floor() as i32..end.ceil() as i32 {
            let coverage = ((end.min(pixel as f32 + 1.0) - start.max(pixel as f32)).clamp(0.0, 1.0)
                * 255.0)
                .ceil() as u8;
            if coverage == 0 {
                continue;
            }
            let pixel_x = x + pixel;
            rectangle(
                renderer,
                pixel_x,
                y,
                1,
                thickness,
                with_alpha(color, coverage),
            );
            let edge_coverage = (edge_alpha * f32::from(coverage) / 255.0).round() as u8;
            let edge_color = with_alpha(color, edge_coverage);
            rectangle(renderer, pixel_x, y - 1, 1, 1, edge_color);
            rectangle(renderer, pixel_x, y + thickness as i32, 1, 1, edge_color);
        }
        start += step;
    }
}

fn adjusted_dash_gap(path_length: f32, dash_length: f32, gap_length: f32) -> f32 {
    let min_dashes = ((path_length + gap_length) / (dash_length + gap_length)).floor();
    if min_dashes < 2.0 {
        return gap_length;
    }
    let max_dashes = min_dashes + 1.0;
    let min_gaps = min_dashes - 1.0;
    let max_gaps = max_dashes - 1.0;
    let min_gap = (path_length - min_dashes * dash_length) / min_gaps;
    let max_gap = (path_length - max_dashes * dash_length) / max_gaps;
    if max_gap <= 0.0 || (min_gap - gap_length).abs() < (max_gap - gap_length).abs() {
        min_gap
    } else {
        max_gap
    }
}

#[cfg(test)]
mod tests {
    use super::adjusted_dash_gap;

    #[test]
    fn adjusts_dash_gaps_to_fit_the_decoration_length() {
        let gap = adjusted_dash_gap(101.0, 3.0, 2.0);
        assert!((gap - 1.9).abs() < f32::EPSILON);

        let gap = adjusted_dash_gap(114.0, 3.0, 2.0);
        assert!((gap - (45.0 / 22.0)).abs() < f32::EPSILON);
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
