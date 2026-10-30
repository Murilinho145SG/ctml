use skia_safe::{
    Canvas, Color, Color4f, Font, Paint, PaintStyle, Path, PathBuilder, PathFillType, Point, Rect,
    TextBlob,
};

use crate::page::PAGE_PADDING;
use crate::style::{
    TextEmphasisPosition, TextEmphasisSide, TextEmphasisStyle, TextStyle, WritingMode,
};

pub(super) fn draw_text_shadows(canvas: &Canvas, blob: &TextBlob, style: &TextStyle, opacity: f32) {
    for shadow in style.text_shadows.iter().rev() {
        let color = shadow.color;
        let color = Color4f::new(
            f32::from(color[0]) / 255.0,
            f32::from(color[1]) / 255.0,
            f32::from(color[2]) / 255.0,
            f32::from(scale_alpha(color[3], opacity)) / 255.0,
        );
        let Some(filter) = skia_safe::image_filters::drop_shadow_only(
            (shadow.offset_x, shadow.offset_y),
            (shadow.blur, shadow.blur),
            color,
            None,
            None,
            None,
        ) else {
            continue;
        };
        let mut paint = Paint::default();
        paint.set_color(Color::WHITE);
        paint.set_image_filter(filter);
        canvas.draw_text_blob(blob, Point::default(), &paint);
    }
}

pub(super) fn draw_text_emphasis(
    canvas: &Canvas,
    line: &cosmic_text::LayoutRun<'_>,
    glyphs: &[cosmic_text::LayoutGlyph],
    style: &TextStyle,
    writing_mode: WritingMode,
    font: &Font,
    opacity: f32,
) {
    if style.text_emphasis_style == TextEmphasisStyle::None {
        return;
    }
    let color = style.text_emphasis_color.unwrap_or(style.color);
    let mut paint = Paint::default();
    paint.set_color(Color::from_argb(
        scale_alpha(color[3], opacity),
        color[0],
        color[1],
        color[2],
    ));
    let mut seen = std::collections::HashSet::new();
    for glyph in glyphs {
        if !seen.insert((glyph.start, glyph.end))
            || line
                .text
                .get(glyph.start..glyph.end)
                .is_none_or(|text| text.chars().all(char::is_whitespace))
        {
            continue;
        }
        let radius = (glyph.font_size * 0.14).clamp(1.0, 2.5);
        let center = Point::new(
            glyph.x + glyph.w * 0.5 + PAGE_PADDING,
            match emphasis_position(style, writing_mode) {
                TextEmphasisPosition::Over => line.line_top + radius + PAGE_PADDING,
                TextEmphasisPosition::Under => {
                    line.line_top + line.line_height - radius - 1.0 + PAGE_PADDING
                }
            },
        );
        draw_emphasis_mark(
            canvas,
            center,
            radius,
            style.text_emphasis_style.clone(),
            font,
            &mut paint,
        );
    }
}

fn emphasis_position(style: &TextStyle, writing_mode: WritingMode) -> TextEmphasisPosition {
    if style.writing_mode != writing_mode || writing_mode == WritingMode::HorizontalTb {
        return style.text_emphasis_position;
    }
    match (writing_mode, style.text_emphasis_side) {
        (WritingMode::VerticalRl, Some(TextEmphasisSide::Left))
        | (WritingMode::VerticalLr, Some(TextEmphasisSide::Right)) => TextEmphasisPosition::Under,
        (WritingMode::VerticalRl, Some(TextEmphasisSide::Right))
        | (WritingMode::VerticalLr, Some(TextEmphasisSide::Left)) => TextEmphasisPosition::Over,
        _ => style.text_emphasis_position,
    }
}

pub(super) fn scale_alpha(alpha: u8, opacity: f32) -> u8 {
    (f32::from(alpha) * opacity).round().clamp(0.0, 255.0) as u8
}

fn draw_emphasis_mark(
    canvas: &Canvas,
    center: Point,
    radius: f32,
    style: TextEmphasisStyle,
    font: &Font,
    paint: &mut Paint,
) {
    let open = matches!(
        &style,
        TextEmphasisStyle::OpenDot
            | TextEmphasisStyle::OpenCircle
            | TextEmphasisStyle::OpenDoubleCircle
            | TextEmphasisStyle::OpenTriangle
            | TextEmphasisStyle::OpenSesame
    );
    paint.set_style(if open {
        PaintStyle::Stroke
    } else {
        PaintStyle::Fill
    });
    paint.set_stroke_width((radius * 0.4).max(0.7));
    match style {
        TextEmphasisStyle::FilledDot | TextEmphasisStyle::OpenDot => {
            canvas.draw_circle(center, radius * 0.58, paint);
        }
        TextEmphasisStyle::FilledCircle | TextEmphasisStyle::OpenCircle => {
            canvas.draw_circle(center, radius, paint);
        }
        TextEmphasisStyle::FilledDoubleCircle => {
            let mut rings = PathBuilder::new();
            rings.set_fill_type(PathFillType::EvenOdd);
            rings.add_circle(center, radius, None);
            rings.add_circle(center, radius * 0.55, None);
            let rings = Path::from(rings);
            canvas.draw_path(&rings, paint);
            canvas.draw_circle(center, radius * 0.2, paint);
        }
        TextEmphasisStyle::OpenDoubleCircle => {
            paint.set_style(PaintStyle::Stroke);
            canvas.draw_circle(center, radius, paint);
            canvas.draw_circle(center, radius * 0.52, paint);
        }
        TextEmphasisStyle::FilledTriangle | TextEmphasisStyle::OpenTriangle => {
            let path = Path::polygon(
                &[
                    Point::new(center.x, center.y - radius),
                    Point::new(center.x + radius, center.y + radius * 0.8),
                    Point::new(center.x - radius, center.y + radius * 0.8),
                ],
                true,
                None,
                None,
            );
            canvas.draw_path(&path, paint);
        }
        TextEmphasisStyle::FilledSesame | TextEmphasisStyle::OpenSesame => {
            let bounds = Rect::from_xywh(
                center.x - radius * 0.65,
                center.y - radius,
                radius * 1.3,
                radius * 2.0,
            );
            canvas.draw_oval(bounds, paint);
        }
        TextEmphasisStyle::Custom(mark) => {
            let mut mark_font = font.clone();
            mark_font.set_size((font.size() * 0.5).max(1.0));
            let (width, bounds) = mark_font.measure_text(&mark, Some(paint));
            canvas.draw_str(
                mark,
                (
                    center.x - width * 0.5,
                    center.y - (bounds.top() + bounds.bottom()) * 0.5,
                ),
                &mark_font,
                paint,
            );
        }
        TextEmphasisStyle::None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::emphasis_position;
    use crate::style::{TextEmphasisPosition, TextEmphasisSide, TextStyle, WritingMode};

    #[test]
    fn vertical_emphasis_side_maps_to_the_requested_physical_side() {
        let mut style = TextStyle {
            writing_mode: WritingMode::VerticalRl,
            text_emphasis_side: Some(TextEmphasisSide::Left),
            ..TextStyle::default()
        };
        assert_eq!(
            emphasis_position(&style, WritingMode::VerticalRl),
            TextEmphasisPosition::Under
        );

        style.text_emphasis_side = Some(TextEmphasisSide::Right);
        assert_eq!(
            emphasis_position(&style, WritingMode::VerticalRl),
            TextEmphasisPosition::Over
        );

        style.writing_mode = WritingMode::VerticalLr;
        style.text_emphasis_side = Some(TextEmphasisSide::Left);
        assert_eq!(
            emphasis_position(&style, WritingMode::VerticalLr),
            TextEmphasisPosition::Over
        );

        style.text_emphasis_side = Some(TextEmphasisSide::Right);
        assert_eq!(
            emphasis_position(&style, WritingMode::VerticalLr),
            TextEmphasisPosition::Under
        );
    }
}
