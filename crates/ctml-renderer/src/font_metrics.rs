use cosmic_text::{
    FontSystem,
    fontdb::{Family as DatabaseFamily, Query as FontQuery},
};
use skia_safe::{Font, FontHinting, FontMgr};

use crate::style::TextStyle;

pub(crate) fn normal_line_height(
    font_system: &mut FontSystem,
    style: &TextStyle,
    family: &str,
) -> Option<f32> {
    let family = match family.to_ascii_lowercase().as_str() {
        "serif" => DatabaseFamily::Serif,
        "sans-serif" => DatabaseFamily::SansSerif,
        "monospace" => DatabaseFamily::Monospace,
        "cursive" => DatabaseFamily::Cursive,
        "fantasy" => DatabaseFamily::Fantasy,
        _ => DatabaseFamily::Name(family),
    };
    let families = [family];
    let id = font_system.db().query(&FontQuery {
        families: &families,
        weight: style.weight,
        stretch: style.stretch,
        style: match style.font_style {
            cosmic_text::Style::Normal => cosmic_text::fontdb::Style::Normal,
            cosmic_text::Style::Italic => cosmic_text::fontdb::Style::Italic,
            cosmic_text::Style::Oblique => cosmic_text::fontdb::Style::Oblique,
        },
    })?;
    let font_manager = FontMgr::new();
    let typeface = font_system
        .db()
        .with_face_data(id, |data, index| {
            font_manager.new_from_bytes(data, Some(index))
        })
        .flatten()?;
    let mut font = Font::from_typeface(typeface, style.font_size);
    font.set_hinting(FontHinting::None);
    font.set_force_auto_hinting(true);
    font.set_subpixel(true);
    font.set_linear_metrics(true);
    let (_, metrics) = font.metrics();
    Some(normal_line_height_from_metrics(
        -metrics.ascent,
        metrics.descent,
        metrics.leading,
        false,
        true,
    ))
}

fn normal_line_height_from_metrics(
    ascent: f32,
    descent: f32,
    leading: f32,
    subpixel_ascent_descent: bool,
    subpixel_positioning: bool,
) -> f32 {
    let (ascent, descent) = if subpixel_ascent_descent && (ascent < 3.0 || ascent + descent < 2.0) {
        (ascent, descent)
    } else {
        let mut ascent = ascent.round();
        let descent_metric = descent;
        let mut descent = descent.round();
        if descent < descent_metric && subpixel_positioning && ascent >= 1.0 {
            ascent -= 1.0;
            descent += 1.0;
        }
        (ascent, descent)
    };
    ascent.round() + descent.round() + leading.round()
}

#[cfg(test)]
mod tests {
    use super::normal_line_height_from_metrics;

    #[test]
    fn rounds_metrics_without_a_fixed_line_height_floor() {
        let line_height = normal_line_height_from_metrics(16.3, 3.8, 0.6, false, true);

        assert_eq!(line_height, 21.0);
        assert!(line_height < 18.0 * 1.2);
    }

    #[test]
    fn rounds_each_font_metric_before_adding_line_height() {
        let line_height = normal_line_height_from_metrics(10.2, 2.2, 0.2, false, true);

        assert_eq!(line_height, 12.0);
    }
}
