use cosmic_text::{
    FontSystem,
    fontdb::{Family as DatabaseFamily, Query as FontQuery},
};

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
    let font = font_system.get_font(id, style.weight)?;
    let metrics = font.as_swash().metrics(&[]);
    normal_line_height_from_metrics(
        metrics.ascent,
        metrics.descent,
        metrics.leading,
        style.font_size,
        metrics.units_per_em,
    )
}

fn normal_line_height_from_metrics(
    ascent: f32,
    descent: f32,
    leading: f32,
    font_size: f32,
    units_per_em: u16,
) -> Option<f32> {
    (units_per_em != 0).then(|| {
        let scale = font_size / f32::from(units_per_em);
        (ascent * scale).round() + (descent * scale).round() + (leading * scale).round()
    })
}

#[cfg(test)]
mod tests {
    use super::normal_line_height_from_metrics;

    #[test]
    fn scales_positive_font_metrics_without_a_fixed_line_height_floor() {
        let line_height = normal_line_height_from_metrics(1854.0, 434.0, 67.0, 18.0, 2048)
            .expect("font metrics should have a nonzero units-per-em value");

        assert_eq!(line_height, 21.0);
        assert!(line_height < 18.0 * 1.2);
    }

    #[test]
    fn rounds_each_font_metric_before_adding_line_height() {
        let line_height = normal_line_height_from_metrics(1740.0, 375.0, 30.0, 12.0, 2048)
            .expect("font metrics should have a nonzero units-per-em value");

        assert_eq!(line_height, 12.0);
    }
}
